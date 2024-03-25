//! Module defining iterators on table rows.

use std::{
  fs::File,
  io::{BufRead, BufReader, Write},
  path::Path,
};

use base64::{engine::general_purpose, read::DecoderReader};
use memchr::memmem::Finder;
use once_cell::sync::Lazy;
use quick_xml::{events::Event, Reader};

use crate::{
  data::{
    binary::Binary, binary2::Binary2, stream::Stream, tabledata::TableData, TableOrBinOrBin2,
  },
  error::VOTableError,
  impls::{
    b64::read::{
      B64Cleaner, BulkBinaryRowDeserializer, OwnedB64Cleaner, OwnedBulkBinaryRowDeserializer,
    },
    mem::VoidTableDataContent,
    Schema, VOTableValue,
  },
  iter::elems::{
    Binary2RowValueIterator, BinaryRowValueIterator, DataTableRowValueIterator, RowValueIterator,
  },
  resource::{Resource, ResourceOrTable, ResourceSubElem},
  table::{Table, TableElem},
  utils::{discard_comment, discard_event, is_empty},
  votable::{VOTable, VOTableWrapper},
  VOTableElement,
};

pub mod elems;
pub mod strings;

static TR_END_FINDER: Lazy<Finder<'static>> = Lazy::new(|| Finder::new("</TR>"));
static STREAM_END_FINDER: Lazy<Finder<'static>> = Lazy::new(|| Finder::new("</STREAM>"));
static TABLEDATA_END_FINDER: Lazy<Finder<'static>> = Lazy::new(|| Finder::new("</TABLEDATA>"));

/// Iterate over the raw rows (i.e. everything inside the `<TR>`/`</TR>` tags).
/// We assume the `<TABLEDATA>` tag has already been consumed and this iterator will consume
/// the `</TABLEDATA>` tag.
pub struct TabledataRowIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
  has_next: bool,
}

impl<'a, R: BufRead> TabledataRowIterator<'a, R> {
  /// We assume here that the reader has already consumed the `<TABLEDATA>` tag.
  pub fn new(reader: &'a mut Reader<R>, reader_buff: &'a mut Vec<u8>) -> Self {
    Self {
      reader,
      reader_buff,
      has_next: true,
    }
  }
}

impl<'a, R: BufRead> Iterator for TabledataRowIterator<'a, R> {
  type Item = Result<Vec<u8>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    next_tabledata_row(self.reader, self.reader_buff, &mut self.has_next)
  }
}

/// Iterate over the raw rows (i.e. everything inside the `<TR>`/`</TR>` tags).
/// We assume the `<TABLEDATA>` tag has already been consumed and this iterator will consume
/// the `</TABLEDATA>` tag.
pub struct OwnedTabledataRowIterator<R: BufRead> {
  pub reader: Reader<R>,
  pub reader_buff: Vec<u8>,
  pub votable: VOTable<VoidTableDataContent>,
  pub has_next: bool,
}

impl<R: BufRead> OwnedTabledataRowIterator<R> {
  pub fn skip_remaining_data(&mut self) -> Result<(), VOTableError> {
    self
      .reader
      .read_to_end(
        TableData::<VoidTableDataContent>::TAG_BYTES,
        &mut self.reader_buff,
      )
      .map_err(VOTableError::Read)
  }

  pub fn read_to_end(self) -> Result<VOTable<VoidTableDataContent>, VOTableError> {
    let Self {
      mut reader,
      mut reader_buff,
      mut votable,
      has_next: _,
    } = self;
    votable
      .read_from_data_end_to_end(&mut reader, &mut reader_buff)
      .map(|()| votable)
  }
}

impl<R: BufRead> Iterator for OwnedTabledataRowIterator<R> {
  type Item = Result<Vec<u8>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    next_tabledata_row(&mut self.reader, &mut self.reader_buff, &mut self.has_next)
  }
}

fn next_tabledata_row<T: BufRead>(
  reader: &mut Reader<T>,
  reader_buff: &mut Vec<u8>,
  has_next: &mut bool,
) -> Option<Result<Vec<u8>, VOTableError>> {
  if *has_next {
    reader_buff.clear();
    loop {
      let event = reader.read_event(reader_buff);
      match event {
        Ok(Event::Start(ref e)) if e.name() == b"TR" => {
          let mut raw_row: Vec<u8> = Vec::with_capacity(256);
          return Some(
            read_until_found(TR_END_FINDER.as_ref(), reader, &mut raw_row).map(move |_| raw_row),
          );
        }
        Ok(Event::End(ref e)) if e.name() == TableData::<VoidTableDataContent>::TAG_BYTES => {
          *has_next = false;
          return None;
        }
        Ok(Event::Eof) => return Some(Err(VOTableError::PrematureEOF("reading rows"))),
        Ok(Event::Text(ref e)) if is_empty(e) => {}
        Ok(Event::Comment(ref e)) => {
          discard_comment(e, reader, TableData::<VoidTableDataContent>::TAG)
        }
        Ok(event) => discard_event(event, TableData::<VoidTableDataContent>::TAG),
        Err(e) => return Some(Err(VOTableError::Read(e))),
      }
    }
  } else {
    None
  }
}

/// Same as `read_until` but taking a `memchr::memmem::Finder` for better performances when
/// a same `needle` has to be used several times.
fn read_until_found<T: BufRead>(
  finder: Finder<'_>,
  reader: &mut Reader<T>,
  buf: &mut Vec<u8>,
) -> Result<usize, VOTableError> {
  let needle = finder.needle();
  let l = needle.len();
  let r = reader.get_mut();
  let mut ending_pattern: Option<(&[u8], &[u8])> = None;
  let mut read = 0;
  loop {
    let (done, used) = {
      let available = match r.fill_buf() {
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
        Err(e) => return Err(VOTableError::Io(e)),
      };
      if let Some((start, end)) = ending_pattern {
        if available.starts_with(end) {
          r.consume(end.len());
          read += end.len();
          return Ok(read);
        } else {
          // not the right pattern, starting part to be added!!
          buf.extend_from_slice(start);
          ending_pattern = None;
        }
      }
      match finder.find(available) {
        Some(i) => {
          buf.extend_from_slice(&available[..i]);
          (true, i + l)
        }
        None => {
          let len = available.len();
          for sub in 1..l {
            if available.ends_with(&needle[0..sub]) {
              /*println!(
                "{} -- {}",
                from_utf8(&needle[0..sub]).unwrap(),
                from_utf8(&needle[sub..l]).unwrap()
              );*/
              ending_pattern = Some((&needle[0..sub], &needle[sub..l]));
              buf.extend_from_slice(&available[..len - sub]);
              break;
            }
          }
          if ending_pattern.is_none() {
            buf.extend_from_slice(available);
          }
          (false, len)
        }
      }
    };
    r.consume(used);
    read += used;
    if done || used == 0 {
      return Ok(read);
    }
  }
}

fn copy_until_found<R, W>(
  finder: Finder<'_>,
  reader: &mut R,
  writer: &mut W,
) -> Result<usize, VOTableError>
where
  R: BufRead,
  W: Write,
{
  let needle = finder.needle();
  let l = needle.len();
  let r = reader;
  let mut ending_pattern: Option<(&[u8], &[u8])> = None;
  let mut read = 0;
  loop {
    let (done, used) = {
      let available = match r.fill_buf() {
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
        Err(e) => return Err(VOTableError::Io(e)),
      };
      if let Some((start, end)) = ending_pattern {
        if available.starts_with(end) {
          r.consume(end.len());
          read += end.len();
          return Ok(read);
        } else {
          // not the right pattern, starting part to be added!!
          writer.write_all(start).map_err(VOTableError::Io)?;
          ending_pattern = None;
        }
      }
      match finder.find(available) {
        Some(i) => {
          writer
            .write_all(&available[..i])
            .map_err(VOTableError::Io)?;
          (true, i + l)
        }
        None => {
          let len = available.len();
          for sub in 1..l {
            if available.ends_with(&needle[0..sub]) {
              ending_pattern = Some((&needle[0..sub], &needle[sub..l]));
              writer
                .write_all(&available[..len - sub])
                .map_err(VOTableError::Io)?;
              break;
            }
          }
          if ending_pattern.is_none() {
            writer.write_all(available).map_err(VOTableError::Io)?;
          }
          (false, len)
        }
      }
    };
    r.consume(used);
    read += used;
    if done || used == 0 {
      return Ok(read);
    }
  }
}

/// Iterate over the raw rows.
/// We assume the `<BINARY>` or `<BINARY2>` tag has already been consumed and this iterator will consume
/// the `</BINARY>` or `</BINARY2>` tag.
pub struct Binary1or2RowIterator<'a, R: BufRead> {
  reader: BulkBinaryRowDeserializer<'a, R>,
}

impl<'a, R: BufRead> Binary1or2RowIterator<'a, R> {
  /// We assume here that the reader has already consumed the `<STREAM>` tag.
  pub fn new(reader: &'a mut Reader<R>, context: &[TableElem], is_binary2: bool) -> Self {
    let b64_cleaner = B64Cleaner::new(reader.get_mut());
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    // Get schema
    let schema: Vec<Schema> = context
      .iter()
      .filter_map(|table_elem| match table_elem {
        TableElem::Field(field) => Some(field.into()),
        _ => None,
      })
      .collect();
    let reader = if is_binary2 {
      BulkBinaryRowDeserializer::new_binary2(decoder, schema.as_slice())
    } else {
      BulkBinaryRowDeserializer::new_binary(decoder, schema.as_slice())
    };
    Self { reader }
  }
}

impl<'a, R: BufRead> Iterator for Binary1or2RowIterator<'a, R> {
  type Item = Result<Vec<u8>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.reader.has_data_left().unwrap_or(false) {
      let mut row = Vec::with_capacity(512);
      Some(self.reader.read_raw_row(&mut row).map(|_| {
        row.shrink_to_fit();
        row
      }))
    } else {
      None
    }
  }
}

pub struct OwnedBinary1or2RowIterator<R: BufRead> {
  pub votable: VOTable<VoidTableDataContent>,
  pub reader: OwnedBulkBinaryRowDeserializer<R>,
  pub is_binary2: bool,
}

impl<R: BufRead> OwnedBinary1or2RowIterator<R> {
  pub fn new(reader: Reader<R>, votable: VOTable<VoidTableDataContent>, is_binary2: bool) -> Self {
    let b64_cleaner = OwnedB64Cleaner::new(reader.into_inner());
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    // Get schema
    let schema: Vec<Schema> = votable
      .get_first_table()
      .unwrap() // .resources[0].tables[0]
      .elems
      .iter()
      .filter_map(|table_elem| match table_elem {
        TableElem::Field(field) => Some(field.into()),
        _ => None,
      })
      .collect();
    let reader = if is_binary2 {
      OwnedBulkBinaryRowDeserializer::new_binary2(decoder, schema.as_slice())
    } else {
      OwnedBulkBinaryRowDeserializer::new_binary(decoder, schema.as_slice())
    };
    Self {
      votable,
      reader,
      is_binary2,
    }
  }

  pub fn skip_remaining_data(mut self) -> Result<Self, VOTableError> {
    match self.reader.has_data_left() {
      Ok(true) => {
        let Self {
          votable,
          reader,
          is_binary2,
        } = self;
        reader.skip_remaining_data().map(|reader| Self {
          votable,
          reader,
          is_binary2,
        })
      }
      Ok(false) => Ok(self),
      Err(e) => Err(e),
    }
  }

  pub fn read_to_end(self) -> Result<VOTable<VoidTableDataContent>, VOTableError> {
    let Self {
      mut votable,
      reader,
      is_binary2,
    } = self;
    // TODO: partly redundant code with SimpleVOTableRowIterator...
    let mut reader = Reader::from_reader(reader.into_inner());
    reader.check_end_names(false);
    let mut reader_buff: Vec<u8> = Vec::with_capacity(512);
    reader
      .read_to_end(
        if is_binary2 {
          b"BINARY2".to_vec()
        } else {
          b"BINARY".to_vec()
        },
        &mut reader_buff,
      )
      .map_err(|e| VOTableError::Custom(format!("Reading to BINARY or BINARY2... {:?}", e)))?;
    votable
      .read_from_data_end_to_end(&mut reader, &mut reader_buff)
      .map(|()| votable)
  }
}

impl<R: BufRead> Iterator for OwnedBinary1or2RowIterator<R> {
  type Item = Result<Vec<u8>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    // TODO: ask the exact max binary size (estimated from the number of bytes of each field)?!
    if self.reader.has_data_left().unwrap_or(false) {
      let mut row = Vec::with_capacity(512);
      Some(self.reader.read_raw_row(&mut row).map(|_| {
        row.shrink_to_fit();
        row
      }))
    } else {
      None
    }
  }
}

/// Structure made to iterate on the raw rows of a "simple" VOTable.
/// By "simple", we mean a VOTable containing a single resource containing itself a single table.  
pub struct SimpleVOTableRowIterator<R: BufRead> {
  pub reader: Reader<R>,
  pub reader_buff: Vec<u8>,
  pub votable: VOTable<VoidTableDataContent>,
  pub data_type: TableOrBinOrBin2,
}

impl SimpleVOTableRowIterator<BufReader<File>> {
  /// Open file and starts parsing the VOTable till (inclusive):
  /// * `TABLEDATA` for the `TABLEDATA` tag
  /// * `STREAM` for `BINARY` and `BINARY2` tags
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (votable, resource, reader) =
      VOTableWrapper::<VoidTableDataContent>::manual_from_ivoa_xml_file(path, &mut reader_buff)?;
    SimpleVOTableRowIterator::<BufReader<File>>::from_votable_resource_reader(
      votable,
      resource,
      reader,
      reader_buff,
    )
  }
}

impl<R: BufRead> SimpleVOTableRowIterator<R> {
  pub fn from_reader(reader: R) -> Result<Self, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (votable, resource, reader) =
      VOTable::from_reader_till_next_resource(reader, &mut reader_buff)?;
    Self::from_votable_resource_reader(votable, resource, reader, reader_buff)
  }

  fn from_votable_resource_reader(
    mut votable: VOTable<VoidTableDataContent>,
    mut resource: Resource<VoidTableDataContent>,
    mut reader: Reader<R>,
    mut reader_buff: Vec<u8>,
  ) -> Result<Self, VOTableError> {
    let mut sub_elem = resource
      .read_till_next_table_by_ref(&mut reader, &mut reader_buff)
      .and_then(|opt_sub_elem| {
        opt_sub_elem.ok_or_else(|| {
          VOTableError::Custom(String::from("No table found in the VOTable resource!"))
        })
      })?;
    match &mut sub_elem {
      ResourceSubElem {
        links: _,
        resource_or_table: ResourceOrTable::<_>::Table(ref mut table),
        ..
      } => {
        if let Some(mut data) = table.read_till_data_by_ref(&mut reader, &mut reader_buff)? {
          match data.read_till_table_bin_or_bin2_or_fits_by_ref(&mut reader, &mut reader_buff)? {
            Some(TableOrBinOrBin2::TableData) => {
              table.set_data_by_ref(data);
              resource.push_sub_elem_by_ref(sub_elem);
              votable.push_resource_by_ref(resource);
              Ok(SimpleVOTableRowIterator {
                reader,
                reader_buff,
                votable,
                data_type: TableOrBinOrBin2::TableData,
              })
            }
            Some(TableOrBinOrBin2::Binary) => {
              let stream = Stream::open_stream(&mut reader, &mut reader_buff)?;
              let binary = Binary::from_stream(stream);
              data.set_binary_by_ref(binary);
              table.set_data_by_ref(data);
              resource.push_sub_elem_by_ref(sub_elem);
              votable.push_resource_by_ref(resource);
              Ok(SimpleVOTableRowIterator {
                reader,
                reader_buff,
                votable,
                data_type: TableOrBinOrBin2::Binary,
              })
            }
            Some(TableOrBinOrBin2::Binary2) => {
              let stream = Stream::open_stream(&mut reader, &mut reader_buff)?;
              let binary2 = Binary2::from_stream(stream);
              data.set_binary2_by_ref(binary2);
              table.set_data_by_ref(data);
              resource.push_sub_elem_by_ref(sub_elem);
              votable.push_resource_by_ref(resource);
              Ok(SimpleVOTableRowIterator {
                reader,
                reader_buff,
                votable,
                data_type: TableOrBinOrBin2::Binary2,
              })
            }
            Some(TableOrBinOrBin2::Fits(_)) => Err(VOTableError::Custom(String::from(
              "FITS data not supported",
            ))),
            None => Err(VOTableError::Custom(String::from(
              "No data found in the first VOtable table",
            ))),
          }
        } else {
          Err(VOTableError::Custom(String::from(
            "No data found in the first VOTable table",
          )))
        }
      }
      _ => Err(VOTableError::Custom(String::from("Not a table?!"))),
    }
  }

  pub fn data_type(&self) -> &TableOrBinOrBin2 {
    &self.data_type
  }

  pub fn votable(&self) -> &VOTable<VoidTableDataContent> {
    &self.votable
  }

  /// An external code have to take charge of the parsing o the data part of the VOTable till:
  /// * `</TABLEDATA>` for `<TABLEDATA>`
  /// * `</BINARY>` for `<BINARY>`
  /// * `</BINARY2>` for `<BINARY2>`
  pub fn borrow_mut_reader_and_buff(&mut self) -> (&mut Reader<R>, &mut Vec<u8>) {
    (&mut self.reader, &mut self.reader_buff)
  }

  /// This method returns an iterator over each row in which each row is a of `Vec<VOTableValue`.
  /// It is generic and is valid for either TableData, Bianry or Binary2.
  /// WARNING: use either this method *or* one of the `to_onwed` method
  /// (since they will consume data rows).
  pub fn to_row_value_iter(&mut self) -> RowValueIterator<'_, R> {
    let table = self.votable.get_first_table_mut().unwrap();
    let schema: Vec<Schema> = table
      .elems
      .iter()
      .filter_map(|table_elem| match table_elem {
        TableElem::Field(field) => Some(field.into()),
        _ => None,
      })
      .collect();
    match &self.data_type {
      TableOrBinOrBin2::TableData => RowValueIterator::TableData(DataTableRowValueIterator::new(
        &mut self.reader,
        &mut self.reader_buff,
        table,
        schema,
      )),
      TableOrBinOrBin2::Binary => {
        RowValueIterator::BinaryTable(BinaryRowValueIterator::new(&mut self.reader, table, schema))
      }
      TableOrBinOrBin2::Binary2 => RowValueIterator::Binary2Table(Binary2RowValueIterator::new(
        &mut self.reader,
        table,
        schema,
      )),
      _ => unreachable!(),
    }
  }

  /// Before calling this method, you **must** ensure that `self.data_type()` returns `TableOrBinOrBin2::TableData`
  pub fn to_owned_tabledata_row_iterator(self) -> OwnedTabledataRowIterator<R> {
    assert!(matches!(self.data_type, TableOrBinOrBin2::TableData));
    OwnedTabledataRowIterator {
      reader: self.reader,
      reader_buff: self.reader_buff,
      votable: self.votable,
      has_next: true,
    }
  }

  /// Before calling this method, you **must** ensure that `self.data_type()` returns `TableOrBinOrBin2::Binary`
  pub fn to_owned_binary_row_iterator(self) -> OwnedBinary1or2RowIterator<R> {
    assert!(matches!(self.data_type, TableOrBinOrBin2::Binary));
    OwnedBinary1or2RowIterator::new(self.reader, self.votable, false)
  }

  /// Before calling this method, you **must** ensure that `self.data_type()` returns `TableOrBinOrBin2::Binary2`
  pub fn to_owned_binary2_row_iterator(self) -> OwnedBinary1or2RowIterator<R> {
    assert!(matches!(self.data_type, TableOrBinOrBin2::Binary2));
    OwnedBinary1or2RowIterator::new(self.reader, self.votable, true)
  }

  /// You can call this method only if you have not yet consumed:
  /// * `</TABLEDATA>` in the case of `<TABLEDATA>`
  /// * `</STREAM>` **and** `</BINARY>` in the case of `<BINARY>`
  /// * `</STREAM>` **and** `</BINARY2>` in the case of `<BINARY2>`
  pub fn skip_remaining_data(&mut self) -> Result<(), VOTableError> {
    match self.data_type {
      TableOrBinOrBin2::TableData => self
        .reader
        .read_to_end(
          TableData::<VoidTableDataContent>::TAG_BYTES,
          &mut self.reader_buff,
        )
        .map_err(VOTableError::Read),
      TableOrBinOrBin2::Binary => self
        .reader
        .read_to_end(
          Stream::<VoidTableDataContent>::TAG_BYTES,
          &mut self.reader_buff,
        )
        .map_err(VOTableError::Read)
        .and_then(|_| {
          self
            .reader
            .read_to_end(
              Binary::<VoidTableDataContent>::TAG_BYTES,
              &mut self.reader_buff,
            )
            .map_err(VOTableError::Read)
        }),
      TableOrBinOrBin2::Binary2 => self
        .reader
        .read_to_end(
          Stream::<VoidTableDataContent>::TAG_BYTES,
          &mut self.reader_buff,
        )
        .map_err(VOTableError::Read)
        .and_then(|_| {
          self
            .reader
            .read_to_end(
              Binary2::<VoidTableDataContent>::TAG_BYTES,
              &mut self.reader_buff,
            )
            .map_err(VOTableError::Read)
        }),
      _ => unreachable!(),
    }
  }

  /// You can call this method only if you have not yet consumed:
  /// * `</TABLEDATA>` in the case of `<TABLEDATA>`
  /// * `</STREAM>` **and** `</BINARY>` in the case of `<BINARY>`
  /// * `</STREAM>` **and** `</BINARY2>` in the case of `<BINARY2>`
  pub fn copy_remaining_data<W: Write>(&mut self, mut write: W) -> Result<(), VOTableError> {
    match self.data_type {
      TableOrBinOrBin2::TableData => copy_until_found(
        TABLEDATA_END_FINDER.as_ref(),
        self.reader.get_mut(),
        &mut write,
      ),
      TableOrBinOrBin2::Binary => copy_until_found(
        STREAM_END_FINDER.as_ref(),
        self.reader.get_mut(),
        &mut write,
      ),
      TableOrBinOrBin2::Binary2 => copy_until_found(
        STREAM_END_FINDER.as_ref(),
        self.reader.get_mut(),
        &mut write,
      ),
      _ => unreachable!(),
    }
    .map(|_| ())
  }

  pub fn end_of_it(self) -> VOTable<VoidTableDataContent> {
    self.votable
  }

  pub fn read_to_end(self) -> Result<VOTable<VoidTableDataContent>, VOTableError> {
    let Self {
      mut reader,
      mut reader_buff,
      mut votable,
      data_type: _,
    } = self;
    votable
      .read_from_data_end_to_end(&mut reader, &mut reader_buff)
      .map(|()| votable)
  }
}

/// Iterates over a table rows.
pub trait TableIter: Iterator<Item = Result<Vec<VOTableValue>, VOTableError>> {
  /// Returns the table metadata.
  fn table(&mut self) -> &mut Table<VoidTableDataContent>;
  /// Read to the end of the table, skipping all remaining data rows.
  fn read_to_end(self) -> Result<(), VOTableError>;
}

/// Returns an Iterator on the tables a VOTable contains.
/// For each table, an iterator on the table rows is provided.
/// The iteration on a table rows must be complete before iterating to the the new table.
/// TODO:
/// * to use this iterator like `SimpleVOTableRowIterator`, we **must** implement
/// methods starting reading again after the last table.
pub struct VOTableIterator<R: BufRead> {
  reader: Reader<R>,
  reader_buff: Vec<u8>,
  votable: VOTable<VoidTableDataContent>,
  resource_stack: Vec<Resource<VoidTableDataContent>>,
  resource_sub_elems_stack: Vec<ResourceSubElem<VoidTableDataContent>>,
}

impl VOTableIterator<BufReader<File>> {
  pub fn from_file<P: AsRef<Path>>(
    path: P,
  ) -> Result<VOTableIterator<BufReader<File>>, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (votable, resource, reader) =
      VOTableWrapper::<VoidTableDataContent>::manual_from_ivoa_xml_file(path, &mut reader_buff)?;
    let mut resource_stack = Vec::with_capacity(4);
    resource_stack.push(resource);
    Ok(VOTableIterator::<BufReader<File>> {
      reader,
      reader_buff,
      votable,
      resource_stack,
      resource_sub_elems_stack: Vec::with_capacity(10),
    })
  }

  pub fn end_of_it(self) -> VOTable<VoidTableDataContent> {
    self.votable
  }
}

impl<R: BufRead> VOTableIterator<R> {
  pub fn from_reader(reader: R) -> Result<Self, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (votable, resource, reader) =
      VOTable::from_reader_till_next_resource(reader, &mut reader_buff)?;
    let mut resource_stack = Vec::with_capacity(4);
    resource_stack.push(resource);
    Ok(VOTableIterator::<R> {
      reader,
      reader_buff,
      votable,
      resource_stack,
      resource_sub_elems_stack: Vec::with_capacity(10),
    })
  }

  pub fn read_all_skipping_data(mut self) -> Result<VOTable<VoidTableDataContent>, VOTableError> {
    while let Some(table_it) = self.next_table_row_value_iter()? {
      table_it.read_to_end()?;
    }
    assert!(self.resource_sub_elems_stack.is_empty());
    assert!(self.resource_stack.is_empty());
    Ok(self.votable)
  }

  pub fn next_table_row_value_iter(
    &mut self,
  ) -> Result<Option<RowValueIterator<'_, R>>, VOTableError> {
    loop {
      if let Some(mut sub_resource) = self.resource_sub_elems_stack.pop() {
        match &mut sub_resource.resource_or_table {
          ResourceOrTable::<_>::Resource(r) => {
            match r
              .read_till_next_resource_or_table_by_ref(&mut self.reader, &mut self.reader_buff)?
            {
              Some(s) => {
                self.resource_sub_elems_stack.push(sub_resource);
                self.resource_sub_elems_stack.push(s);
              }
              None => {
                if let Some(last) = self.resource_sub_elems_stack.last_mut() {
                  last.push_sub_elem_by_ref(sub_resource)?;
                } else if let Some(last) = self.resource_stack.last_mut() {
                  last.push_sub_elem_by_ref(sub_resource);
                } else {
                  return Err(VOTableError::Custom(String::from(
                    "No more RESOURCE in the stack :o/",
                  )));
                }
              }
            }
          }
          ResourceOrTable::<_>::Table(table) => {
            if let Some(mut data) =
              table.read_till_data_by_ref(&mut self.reader, &mut self.reader_buff)?
            {
              match data.read_till_table_bin_or_bin2_or_fits_by_ref(
                &mut self.reader,
                &mut self.reader_buff,
              )? {
                Some(TableOrBinOrBin2::TableData) => {
                  table.set_data_by_ref(data);

                  let schema: Vec<Schema> = table
                    .elems
                    .iter()
                    .filter_map(|table_elem| match table_elem {
                      TableElem::Field(field) => Some(field.into()),
                      _ => None,
                    })
                    .collect();

                  if let Some(last) = self.resource_sub_elems_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource)?;
                  } else if let Some(last) = self.resource_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource);
                  } else {
                    return Err(VOTableError::Custom(String::from(
                      "No more RESOURCE in the stack :o/",
                    )));
                  }

                  let row_it = DataTableRowValueIterator::new(
                    &mut self.reader,
                    &mut self.reader_buff,
                    self
                      .resource_stack
                      .last_mut()
                      .unwrap()
                      .get_last_table_mut()
                      .unwrap(),
                    schema,
                  );
                  return Ok(Some(RowValueIterator::TableData(row_it)));
                }
                Some(TableOrBinOrBin2::Binary) => {
                  let stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  let binary = Binary::from_stream(stream);
                  data.set_binary_by_ref(binary);
                  table.set_data_by_ref(data);

                  let schema: Vec<Schema> = table
                    .elems
                    .iter()
                    .filter_map(|table_elem| match table_elem {
                      TableElem::Field(field) => Some(field.into()),
                      _ => None,
                    })
                    .collect();

                  if let Some(last) = self.resource_sub_elems_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource)?;
                  } else if let Some(last) = self.resource_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource);
                  } else {
                    return Err(VOTableError::Custom(String::from(
                      "No more RESOURCE in the stack :o/",
                    )));
                  }

                  let row_it = BinaryRowValueIterator::new(
                    &mut self.reader,
                    self
                      .resource_stack
                      .last_mut()
                      .unwrap()
                      .get_last_table_mut()
                      .unwrap(),
                    schema,
                  );
                  return Ok(Some(RowValueIterator::BinaryTable(row_it)));
                }
                Some(TableOrBinOrBin2::Binary2) => {
                  let stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  let binary2 = Binary2::from_stream(stream);
                  data.set_binary2_by_ref(binary2);
                  table.set_data_by_ref(data);

                  let schema: Vec<Schema> = table
                    .elems
                    .iter()
                    .filter_map(|table_elem| match table_elem {
                      TableElem::Field(field) => Some(field.into()),
                      _ => None,
                    })
                    .collect();

                  if let Some(last) = self.resource_sub_elems_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource)?;
                  } else if let Some(last) = self.resource_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource);
                  } else {
                    return Err(VOTableError::Custom(String::from(
                      "No more RESOURCE in the stack :o/",
                    )));
                  }

                  let row_it = Binary2RowValueIterator::new(
                    &mut self.reader,
                    self
                      .resource_stack
                      .last_mut()
                      .unwrap()
                      .get_last_table_mut()
                      .unwrap(),
                    schema,
                  );
                  return Ok(Some(RowValueIterator::Binary2Table(row_it)));
                }
                Some(TableOrBinOrBin2::Fits(fits)) => {
                  data.set_fits_by_ref(fits);
                  table.set_data_by_ref(data);

                  if let Some(last) = self.resource_sub_elems_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource)?;
                  } else if let Some(last) = self.resource_stack.last_mut() {
                    last.push_sub_elem_by_ref(sub_resource);
                  } else {
                    return Err(VOTableError::Custom(String::from(
                      "No more RESOURCE in the stack :o/",
                    )));
                  }
                }
                None => {
                  return Err(VOTableError::Custom(String::from("Unexpected empty DATA")));
                }
              }
            } else {
              if let Some(last) = self.resource_sub_elems_stack.last_mut() {
                last.push_sub_elem_by_ref(sub_resource)?;
              } else if let Some(last) = self.resource_stack.last_mut() {
                last.push_sub_elem_by_ref(sub_resource);
              } else {
                return Err(VOTableError::Custom(String::from(
                  "No more RESOURCE in the stack :o/",
                )));
              }
            }
          }
        }
        // Check the kind of sub-resource
        // - if table, good, return
        // - if resource, try to read sub-resource
        //    - if no sub-resource, add to the prev- sub-resource (to the resource if no more thing in the stack)
      } else if let Some(mut resource) = self.resource_stack.pop() {
        // No more sub-resource in the stack, try to read sub-resource.
        match resource
          .read_till_next_resource_or_table_by_ref(&mut self.reader, &mut self.reader_buff)?
        {
          // If sub-resource found, add it to the stack
          Some(resource_sub_elem) => {
            self.resource_sub_elems_stack.push(resource_sub_elem);
            self.resource_stack.push(resource);
          }
          // Else no more element to read in the resource, add it to the votable
          None => self.votable.push_resource_by_ref(resource),
        }
      } else {
        // No more resource in the stack, try to read next resource.
        match self
          .votable
          .read_till_next_resource_by_ref(&mut self.reader, &mut self.reader_buff)?
        {
          Some(resource) => self.resource_stack.push(resource),
          None => return Ok(None),
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use serde::{de::DeserializeSeed, Deserializer};

  use crate::{
    data::TableOrBinOrBin2,
    impls::{
      b64::read::BinaryDeserializer, visitors::FixedLengthArrayVisitor, Schema, VOTableValue,
    },
    iter::{Binary1or2RowIterator, SimpleVOTableRowIterator, TabledataRowIterator},
    table::TableElem,
  };

  #[test]
  fn test_simple_votable_read_iter_tabledata() {
    println!();
    println!("-- next_table_row_value_iter dss12.vot --");
    println!();

    let mut svor = SimpleVOTableRowIterator::from_file("resources/sdss12.vot").unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::TableData));

    // svor.skip_remaining_data().unwrap();
    let raw_row_it = TabledataRowIterator::new(&mut svor.reader, &mut svor.reader_buff);
    for raw_row_res in raw_row_it {
      eprintln!(
        "ROW: {:?}",
        std::str::from_utf8(&raw_row_res.unwrap()).unwrap()
      );
    }
    let votable = svor.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert!(true)
  }

  #[test]
  fn test_simple_votable_read_iter_tabledata_owned() {
    println!();
    println!("-- next_table_row_value_iter sdss12.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::from_file("resources/sdss12.vot").unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::TableData));
    let mut raw_row_it = svor.to_owned_tabledata_row_iterator();
    let mut n_row = 0_u32;
    while let Some(raw_row_res) = raw_row_it.next() {
      eprintln!(
        "ROW: {:?}",
        std::str::from_utf8(&raw_row_res.unwrap()).unwrap()
      );
      n_row += 1;
    }
    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 50)
  }

  #[test]
  fn test_simple_votable_read_iter_binary1() {
    println!();
    println!("-- next_table_row_value_iter binary.b64 --");
    println!();

    let mut svor = SimpleVOTableRowIterator::from_file("resources/binary.b64").unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary));

    let context = svor.votable.get_first_table().unwrap().elems.as_slice();
    // svor.skip_remaining_data().unwrap();
    let raw_row_it = Binary1or2RowIterator::new(&mut svor.reader, context, false);
    let schema: Vec<Schema> = context
      .iter()
      .filter_map(|table_elem| match table_elem {
        TableElem::Field(field) => Some(field.into()),
        _ => None,
      })
      .collect();
    let schema_len = schema.len();
    for raw_row_res in raw_row_it {
      /*eprintln!(
        "ROW SIZE: {:?}",
        raw_row_res.map(|row| row.len()).unwrap_or(0)
      );*/
      eprintln!(
        "ROW: {:?}",
        raw_row_res.map(|row| {
          let mut binary_deser = BinaryDeserializer::new(Cursor::new(row));
          let mut row: Vec<VOTableValue> = Vec::with_capacity(schema_len);
          for field_schema in schema.iter() {
            let field = field_schema.deserialize(&mut binary_deser).unwrap();
            row.push(field);
          }
          row
        })
      );
    }
    let votable = svor.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());
  }

  #[test]
  fn test_simple_votable_read_iter_binary2() {
    println!();
    println!("-- next_table_row_value_iter gaia_dr3.b264 --");
    println!();

    let mut svor = SimpleVOTableRowIterator::from_file("resources/gaia_dr3.b264").unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary2));

    let context = svor.votable.resources[0]
      .get_first_table()
      .unwrap()
      .elems
      .as_slice();
    // svor.skip_remaining_data().unwrap();
    let schema: Vec<Schema> = context
      .iter()
      .filter_map(|table_elem| match table_elem {
        TableElem::Field(field) => Some(field.into()),
        _ => None,
      })
      .collect();
    let schema_len = schema.len();
    let n_bytes = (schema.len() + 7) / 8;
    let raw_row_it = Binary1or2RowIterator::new(&mut svor.reader, context, true);
    for raw_row_res in raw_row_it {
      /*eprintln!(
        "ROW SIZE: {:?}",
        raw_row_res.map(|row| row.len()).unwrap_or(0)
      );*/
      eprintln!(
        "ROW: {:?}",
        raw_row_res.map(|row| {
          let mut binary_deser = BinaryDeserializer::new(Cursor::new(row));
          let mut row: Vec<VOTableValue> = Vec::with_capacity(schema_len);
          let bytes_visitor = FixedLengthArrayVisitor::new(n_bytes);
          let null_flags: Vec<u8> = (&mut binary_deser)
            .deserialize_tuple(n_bytes, bytes_visitor)
            .unwrap();
          for (i_col, field_schema) in schema.iter().enumerate() {
            let field = field_schema.deserialize(&mut binary_deser).unwrap();
            let is_null = (null_flags[i_col >> 3] & (128_u8 >> (i_col & 7))) != 0;
            if is_null {
              row.push(VOTableValue::Null)
            } else {
              row.push(field)
            };
          }
          row
        })
      );
    }
    let votable = svor.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert!(true)
  }

  /*
  #[test]
  fn test_simple_votable_read_iter_tabledata_owned_local_fxp() {
    println!();
    println!("-- next_table_row_value_iter 1358_vlpv.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::open_file_and_read_to_data(
      "/home/pineau/Téléchargements/1358_vlpv.vot",
    )
    .unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::TableData));
    let mut raw_row_it = svor.to_owned_tabledata_row_iterator();
    let mut n_row = 0_u32;
    while let Some(raw_row_res) = raw_row_it.next() {
      let raw_row = raw_row_res.unwrap();
      let row = std::str::from_utf8(&raw_row).unwrap();
      if row.contains("TR") {
        eprintln!("ROW: {:?}", row);
      }
      n_row += 1;
    }
    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 1720588)
  }*/

  /*#[test]
  fn test_simple_votable_read_iter_binary1_owned_local_fxp() {
    println!();
    println!("-- next_table_row_value_iter 1358_vlpv.b64.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::open_file_and_read_to_data(
      "/home/pineau/Téléchargements/1358_vlpv.b64.vot",
    )
    .unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary));
    let mut raw_row_it = svor.to_owned_binary_row_iterator();
    let mut n_row = 0_u32;
    while let Some(_raw_row_res) = raw_row_it.next() {
      n_row += 1;
    }
    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 1720588)
  }

  #[test]
  fn test_simple_votable_read_iter_binary1_owned_local_fxp_t2() {
    println!();
    println!("-- next_table_row_value_iter 1358_vlpv.b64.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::open_file_and_read_to_data(
      "/home/pineau/Téléchargements/1358_vlpv.b64.vot",
    )
    .unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary));
    let mut raw_row_it = svor.to_owned_binary_row_iterator();
    let mut n_row = 0_u32;
    // Only read the first line
    if let Some(_raw_row_res) = raw_row_it.next() {
      n_row += 1;
    }
    let raw_row_it = raw_row_it.skip_remaining_data().unwrap();
    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 1)
  }*/

  /*
  #[test]
  fn test_simple_votable_read_iter_binary2_owned_local_fxp() {
    println!();
    println!("-- next_table_row_value_iter 1358_vlpv.b64v2.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::open_file_and_read_to_data(
      "/home/pineau/Téléchargements/1358_vlpv.b64v2.vot",
    )
    .unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary2));
    let mut raw_row_it = svor.to_owned_binary2_row_iterator();
    let mut n_row = 0_u32;
    while let Some(_raw_row_res) = raw_row_it.next() {
      n_row += 1;
    }

    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 1720588)
  }*/

  /*
  #[test]
  fn test_simple_votable_read_iter_binary2_owned_local_fxp_2() {
    println!();
    println!("-- next_table_row_value_iter async_20190630210155.ungzip.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::open_file_and_read_to_data(
      "/home/pineau/Téléchargements/async_20190630210155.ungzip.vot",
    )
    .unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary2));
    let mut raw_row_it = svor.to_owned_binary2_row_iterator();
    let mut n_row = 0_u32;
    while let Some(_raw_row_res) = raw_row_it.next() {
      n_row += 1;
    }

    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 3000000);
  }

  #[test]
  fn test_simple_votable_read_iter_binary2_owned_local_fxp_2_t2() {
    println!();
    println!("-- next_table_row_value_iter async_20190630210155.ungzip.vot --");
    println!();

    let svor = SimpleVOTableRowIterator::open_file_and_read_to_data(
      "/home/pineau/Téléchargements/async_20190630210155.ungzip.vot",
    )
    .unwrap();
    assert!(matches!(svor.data_type(), &TableOrBinOrBin2::Binary2));
    let mut raw_row_it = svor.to_owned_binary2_row_iterator();
    let mut n_row = 0_u32;
    if let Some(_raw_row_res) = raw_row_it.next() {
      n_row += 1;
    }

    let raw_row_it = raw_row_it.skip_remaining_data().unwrap();
    let votable = raw_row_it.read_to_end().unwrap();
    println!("VOTable: {}", votable.wrap().to_toml_string(true).unwrap());

    assert_eq!(n_row, 1);
  }*/
}
