use base64::engine::general_purpose;
use base64::read::DecoderReader;
use std::{
  fs::File,
  io::{BufRead, BufReader},
  path::Path,
};

use memchr::memmem::Finder;
use once_cell::sync::Lazy;
use quick_xml::{events::Event, Reader};

use crate::{
  data::{
    binary::Binary, binary2::Binary2, stream::Stream, tabledata::TableData, TableOrBinOrBin2,
  },
  error::VOTableError,
  impls::{
    b64::read::{B64Cleaner, BulkBinaryRowDeserializer},
    mem::VoidTableDataContent,
    Schema, VOTableValue,
  },
  iter::elems::{Binary2RowValueIterator, BinaryRowValueIterator},
  resource::{Resource, ResourceOrTable},
  table::{Table, TableElem},
  votable::{VOTable, VOTableWrapper},
  QuickXmlReadWrite,
};

pub mod elems;
pub mod strings;

use elems::DataTableRowValueIterator;
// use strings::RowStringIterator;

static TR_END_FINDER: Lazy<Finder<'static>> = Lazy::new(|| Finder::new("</TR>"));

/// Iterate over the raw rows (i.e. everything inside the `<TR>`/`</TR>` tags).
/// We assume the `<TABLEDATA>` tag has already been consumed and this iterator will consume
/// the `</TABLEDATA>` tag.
pub struct TabledataRowIterator<'a> {
  reader: &'a mut Reader<BufReader<File>>,
  reader_buff: &'a mut Vec<u8>,
}

impl<'a> TabledataRowIterator<'a> {
  /// We assume here that the reader has already consumed the `<TABLEDATA>` tag.
  pub fn new(reader: &'a mut Reader<BufReader<File>>, reader_buff: &'a mut Vec<u8>) -> Self {
    Self {
      reader,
      reader_buff,
    }
  }

  /// Put the content of the reader in the given buffer until the given 'needle' is reached.
  /// The `needle` is not written in the buffer and is removed from the reader.
  ///
  /// If successful, this function will return the total number of bytes read.
  ///
  /// # Info
  /// Adapted from [the Rust std.io code](https://doc.rust-lang.org/src/std/io/mod.rs.html#2151-2153)
  /// to look for an array and not just a byte.
  pub fn read_until(&mut self, needle: &[u8], buf: &mut Vec<u8>) -> Result<usize, VOTableError> {
    self.read_until_with_finder(&Finder::new(needle), buf)
  }

  /// Same as `read_until` but taking a `memchr::memmem::Finder` for better performances when
  /// a same `needle` has to be used several times.
  pub fn read_until_with_finder(
    &mut self,
    needle_finder: &Finder,
    buf: &mut Vec<u8>,
  ) -> Result<usize, VOTableError> {
    let r = self.reader.get_mut();
    let mut read = 0;
    loop {
      let (done, used) = {
        let available = match r.fill_buf() {
          Ok(n) => n,
          Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
          Err(e) => return Err(VOTableError::Io(e)),
        };
        match needle_finder.find(available) {
          Some(i) => {
            buf.extend_from_slice(&available[..i]);
            (true, i + 1)
          }
          None => {
            buf.extend_from_slice(available);
            (false, available.len())
          }
        }
      };
      r.consume(used + needle_finder.needle().len());
      read += used;
      if done || used == 0 {
        return Ok(read);
      }
    }
  }

  // skip_until ?
}

impl<'a> Iterator for TabledataRowIterator<'a> {
  type Item = Result<Vec<u8>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff);
      match event {
        Ok(Event::Start(ref e)) if e.name() == b"TR" => {
          let mut raw_row: Vec<u8> = Vec::with_capacity(228);
          return Some(
            self
              .read_until_with_finder(&TR_END_FINDER, &mut raw_row)
              .map(move |_| raw_row),
          );
        }
        Ok(Event::End(ref e)) if e.name() == TableData::<VoidTableDataContent>::TAG_BYTES => {
          return None
        }
        Err(e) => return Some(Err(VOTableError::Read(e))),
        Ok(Event::Eof) => return Some(Err(VOTableError::PrematureEOF("reading rows"))),
        Ok(Event::Text(ref e))
          if unsafe { std::str::from_utf8_unchecked(e.escaped()) }
            .trim()
            .is_empty() =>
        {
          continue
        }
        _ => eprintln!("Discarded event reading rows: {:?}", event),
      }
      self.reader_buff.clear();
    }
  }
}

/// Iterate over the raw rows (i.e. everything inside the `<TR>`/`</TR>` tags).
/// We assume the `<TABLEDATA>` tag has already been consumed and this iterator will consume
/// the `</TABLEDATA>` tag.
pub struct Binary1or2RowIterator<'a> {
  reader: BulkBinaryRowDeserializer<'a, BufReader<File>>,
}

impl<'a> Binary1or2RowIterator<'a> {
  /// We assume here that the reader has already consumed the `<STREAM>` tag.
  pub fn new(
    reader: &'a mut Reader<BufReader<File>>,
    context: &[TableElem],
    is_binary2: bool,
  ) -> Self {
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

impl<'a> Iterator for Binary1or2RowIterator<'a> {
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

/// Structure made to iterate on the raw rows of a "simple" VOTable.
/// By "simple", we mean a VOTable containing a single resource containing itself a single table.  
pub struct SimpleVOTableRowIterator {
  reader: Reader<BufReader<File>>,
  reader_buff: Vec<u8>,
  votable: VOTable<VoidTableDataContent>,
  data_type: TableOrBinOrBin2,
}

impl SimpleVOTableRowIterator {
  /// Starts parsing the VOTable till (includive):
  /// * `TABLEDATA` for the `TABLEDATA` tag
  /// * `STREAM` for `BINARY` and `BINARY2` tags
  pub fn open_file_and_read_to_data<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (mut votable, mut resource, mut reader) =
      VOTableWrapper::<VoidTableDataContent>::manual_from_ivoa_xml_file(path, &mut reader_buff)?;
    match resource.read_till_next_resource_or_table_by_ref(&mut reader, &mut reader_buff)? {
      Some(ResourceOrTable::<_>::Table(mut table)) => {
        if let Some(mut data) = table.read_till_data_by_ref(&mut reader, &mut reader_buff)? {
          match data.read_till_table_bin_or_bin2_or_fits_by_ref(&mut reader, &mut reader_buff)? {
            Some(TableOrBinOrBin2::TableData) => {
              table.set_data_by_ref(data);
              resource.push_table_by_ref(table);
              votable.push_resource_by_ref(resource);
              return Ok(SimpleVOTableRowIterator {
                reader,
                reader_buff,
                votable,
                data_type: TableOrBinOrBin2::TableData,
              });
            }
            Some(TableOrBinOrBin2::Binary) => {
              let stream = Stream::open_stream(&mut reader, &mut reader_buff)?;
              let binary = Binary::from_stream(stream);
              data.set_binary_by_ref(binary);
              table.set_data_by_ref(data);
              resource.push_table_by_ref(table);
              votable.push_resource_by_ref(resource);
              return Ok(SimpleVOTableRowIterator {
                reader,
                reader_buff,
                votable,
                data_type: TableOrBinOrBin2::Binary,
              });
            }
            Some(TableOrBinOrBin2::Binary2) => {
              let stream = Stream::open_stream(&mut reader, &mut reader_buff)?;
              let binary2 = Binary2::from_stream(stream);
              data.set_binary2_by_ref(binary2);
              table.set_data_by_ref(data);
              resource.push_table_by_ref(table);
              votable.push_resource_by_ref(resource);
              return Ok(SimpleVOTableRowIterator {
                reader,
                reader_buff,
                votable,
                data_type: TableOrBinOrBin2::Binary2,
              });
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
      _ => Err(VOTableError::Custom(String::from(
        "No table found in the first VOTable resource",
      ))),
    }
  }

  /// An external code have to take charge of the parsing o the data part of the VOTable till:
  /// * `</TABLEDATA>` for `<TABLEDATA>`
  /// * `</BINARY>` for `<BINARY>`
  /// * `</BINARY2>` for `<BINARY2>`
  pub fn borrow_mut_reader_and_buff(&mut self) -> (&mut Reader<BufReader<File>>, &mut Vec<u8>) {
    (&mut self.reader, &mut self.reader_buff)
  }

  /// You can call this method only if you have not yet consumed:
  /// * `</TABLEDATA>` int he case of `<TABLEDATA>`
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

  pub fn read_to_end(self) -> Result<VOTable<VoidTableDataContent>, VOTableError> {
    let Self {
      mut reader,
      mut reader_buff,
      mut votable,
      data_type: _,
    } = self;

    votable.resources[0].tables[0]
      .data
      .as_mut()
      .unwrap()
      .read_sub_elements_by_ref(&mut reader, &mut reader_buff, &Vec::default())?;
    votable.resources[0].tables[0].read_sub_elements_by_ref(&mut reader, &mut reader_buff, &())?;
    votable.resources[0].read_sub_elements_by_ref(&mut reader, &mut reader_buff, &())?;
    votable.read_sub_elements_by_ref(&mut reader, &mut reader_buff, &())?;
    Ok(votable)
  }
}

pub trait TableIter: Iterator<Item = Result<Vec<VOTableValue>, VOTableError>> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent>;
}

/// Returns an Iterator on the tables a VOTable contains.
/// For each table, an iterator on the table rows is provided.
/// The iteration on a table rows must be complete before iterating to the the new table.
pub struct VOTableIterator<R: BufRead> {
  reader: Reader<R>,
  reader_buff: Vec<u8>,
  // fn next_table -> (&votable, &resource, &table, &rows, Enum)
  //
  votable: VOTable<VoidTableDataContent>,
  resource_stack: Vec<Resource<VoidTableDataContent>>,
}

impl VOTableIterator<BufReader<File>> {
  pub fn from_file<P: AsRef<Path>>(
    path: P,
  ) -> Result<VOTableIterator<BufReader<File>>, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (votable, resource, reader) =
      VOTableWrapper::<VoidTableDataContent>::manual_from_ivoa_xml_file(path, &mut reader_buff)?;
    let mut resource_stack = Vec::default();
    resource_stack.push(resource);
    Ok(VOTableIterator::<BufReader<File>> {
      reader,
      reader_buff,
      votable,
      resource_stack,
    })
  }

  pub fn end_of_it(self) -> VOTable<VoidTableDataContent> {
    self.votable
  }
}

impl<R: BufRead> VOTableIterator<R> {
  /*pub fn next_table_row_string_iter<'a>(&'a mut self) -> Result<Option<RowStringIterator<'a, R>>, VOTableError> {
    loop {
      if let Some(mut resource) = self.resource_stack.pop() {
        match resource.read_till_next_resource_or_table_by_ref(&mut self.reader, &mut self.reader_buff)? {
          Some(ResourceOrTable::<_>::Resource(sub_resource)) => {
            self.resource_stack.push(resource);
            self.resource_stack.push(sub_resource);
          },
          Some(ResourceOrTable::<_>::Table(mut table)) => {
            if let Some(mut data) = table.read_till_data_by_ref(&mut self.reader, &mut self.reader_buff)? {
              match data.read_till_table_bin_or_bin2_or_fits_by_ref(&mut self.reader, &mut self.reader_buff)? {
                Some(TableOrBinOrBin2::TableData) => {
                  table.set_data_by_ref(data);
                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = RowStringIterator::new(
                    &mut self.reader,
                    &mut self.reader_buff,
                    // self.resource_stack.last_mut().unwrap().tables.last_mut().unwrap()
                  );
                  return Ok(Some(row_it));
                },
                Some(TableOrBinOrBin2::Binary) => {
                  let _stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  todo!()
                },
                Some(TableOrBinOrBin2::Binary2) => {
                  let _stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  todo!()
                },
                Some(TableOrBinOrBin2::Fits(_fits)) => {
                  todo!()
                },
                None => {
                  todo!()
                },
              }
            } else {
              resource.push_table_by_ref(table);
              self.resource_stack.push(resource);
            }
          },
          None => self.votable.push_resource_by_ref(resource),
        }
      } else {
        match self.votable.read_till_next_resource_by_ref(&mut self.reader, &mut self.reader_buff)? {
          Some(resource) => self.resource_stack.push(resource),
          None => return Ok(None),
        }
      }
    }
  }*/

  pub fn next_table_row_value_iter<'a>(
    &'a mut self,
  ) -> Result<Option<Box<dyn 'a + TableIter>>, VOTableError> {
    loop {
      if let Some(mut resource) = self.resource_stack.pop() {
        match resource
          .read_till_next_resource_or_table_by_ref(&mut self.reader, &mut self.reader_buff)?
        {
          Some(ResourceOrTable::<_>::Resource(sub_resource)) => {
            self.resource_stack.push(resource);
            self.resource_stack.push(sub_resource);
          }
          Some(ResourceOrTable::<_>::Table(mut table)) => {
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

                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = DataTableRowValueIterator::new(
                    &mut self.reader,
                    &mut self.reader_buff,
                    self
                      .resource_stack
                      .last_mut()
                      .unwrap()
                      .tables
                      .last_mut()
                      .unwrap(),
                    schema,
                  );
                  return Ok(Some(Box::new(row_it)));
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

                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = BinaryRowValueIterator::new(
                    &mut self.reader,
                    self
                      .resource_stack
                      .last_mut()
                      .unwrap()
                      .tables
                      .last_mut()
                      .unwrap(),
                    schema,
                  );
                  return Ok(Some(Box::new(row_it)));
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

                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = Binary2RowValueIterator::new(
                    &mut self.reader,
                    self
                      .resource_stack
                      .last_mut()
                      .unwrap()
                      .tables
                      .last_mut()
                      .unwrap(),
                    schema,
                  );
                  return Ok(Some(Box::new(row_it)));
                }
                Some(TableOrBinOrBin2::Fits(fits)) => {
                  data.set_fits_by_ref(fits);
                  table.set_data_by_ref(data);
                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                }
                None => {
                  return Err(VOTableError::Custom(String::from("Unexpected empty DATA")));
                }
              }
            } else {
              resource.push_table_by_ref(table);
              self.resource_stack.push(resource);
            }
          }
          None => self.votable.push_resource_by_ref(resource),
        }
      } else {
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
  use crate::{
    impls::{
      b64::read::BinaryDeserializer, visitors::FixedLengthArrayVisitor, Schema, VOTableValue,
    },
    iter::{Binary1or2RowIterator, SimpleVOTableRowIterator, TabledataRowIterator},
    table::TableElem,
  };
  use serde::{de::DeserializeSeed, Deserializer};
  use std::io::Cursor;

  #[test]
  fn test_simple_votable_read_iter_tabledata() {
    println!();
    println!("-- next_table_row_value_iter dss12.vot --");
    println!();

    let mut svor =
      SimpleVOTableRowIterator::open_file_and_read_to_data("resources/sdss12.vot").unwrap();
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
  fn test_simple_votable_read_iter_binary1() {
    println!();
    println!("-- next_table_row_value_iter binary.b64 --");
    println!();

    let mut svor =
      SimpleVOTableRowIterator::open_file_and_read_to_data("resources/binary.b64").unwrap();

    let context = svor.votable.resources[0].tables[0].elems.as_slice();
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

    let mut svor =
      SimpleVOTableRowIterator::open_file_and_read_to_data("resources/gaia_dr3.b264").unwrap();
    let context = svor.votable.resources[0].tables[0].elems.as_slice();
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
}
