//! Iterator on `TABLEDATA` rows in which a row is a `Vector` of `VOTableValue`.

use std::io::{BufRead, BufReader};

use base64::{
  engine::{general_purpose, GeneralPurpose},
  read::DecoderReader,
};
use quick_xml::{events::Event, Reader};
use serde::{de::DeserializeSeed, Deserializer};

use crate::{
  data::tabledata::FieldIterator,
  error::VOTableError,
  impls::{
    b64::read::{B64Cleaner, BinaryDeserializer},
    mem::VoidTableDataContent,
    visitors::FixedLengthArrayVisitor,
    Schema, VOTableValue,
  },
  iter::TableIter,
  table::Table,
  utils::{discard_comment, is_empty, unexpected_event},
  Binary, Binary2, Stream, TableData, VOTableElement,
};

/// Possible iterators and a table row value.
pub enum RowValueIterator<'a, R: BufRead> {
  TableData(DataTableRowValueIterator<'a, R>),
  BinaryTable(BinaryRowValueIterator<'a, R>),
  Binary2Table(Binary2RowValueIterator<'a, R>),
}

impl<'a, R: BufRead> TableIter for RowValueIterator<'a, R> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent> {
    match self {
      Self::TableData(o) => o.table(),
      Self::BinaryTable(o) => o.table(),
      Self::Binary2Table(o) => o.table(),
    }
  }

  fn read_to_end(self) -> Result<(), VOTableError> {
    match self {
      Self::TableData(o) => o.read_to_end(),
      Self::BinaryTable(o) => o.read_to_end(),
      Self::Binary2Table(o) => o.read_to_end(),
    }
  }
}
impl<'a, R: BufRead> Iterator for RowValueIterator<'a, R> {
  type Item = Result<Vec<VOTableValue>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Self::TableData(o) => o.next(),
      Self::BinaryTable(o) => o.next(),
      Self::Binary2Table(o) => o.next(),
    }
  }
}

pub struct DataTableRowValueIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
  table: &'a mut Table<VoidTableDataContent>,
  schema: Vec<Schema>,
}

impl<'a, R: BufRead> DataTableRowValueIterator<'a, R> {
  pub fn new(
    reader: &'a mut Reader<R>,
    reader_buff: &'a mut Vec<u8>,
    table: &'a mut Table<VoidTableDataContent>,
    schema: Vec<Schema>,
  ) -> Self {
    Self {
      reader,
      reader_buff,
      table,
      schema,
    }
  }
}

impl<'a, R: BufRead> TableIter for DataTableRowValueIterator<'a, R> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent> {
    self.table
  }

  fn read_to_end(self) -> Result<(), VOTableError> {
    self
      .reader
      .read_to_end(
        TableData::<VoidTableDataContent>::TAG_BYTES,
        self.reader_buff,
      )
      .map_err(VOTableError::Read)
  }
}

impl<'a, R: BufRead> Iterator for DataTableRowValueIterator<'a, R> {
  type Item = Result<Vec<VOTableValue>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff);
      match event {
        Err(e) => return Some(Err(VOTableError::Read(e))),
        Ok(mut event) => match &mut event {
          Event::Start(ref e) if e.local_name() == b"TR" => {
            let nf = self.schema.len();
            return Some(
              FieldIterator::new(self.reader, self.reader_buff)
                .zip(self.schema.iter())
                .map(|(f_res, s)| f_res.and_then(|f| s.value_from_str(f.trim())))
                .collect::<Result<Vec<VOTableValue>, VOTableError>>()
                .and_then(|fields| {
                  if fields.len() == nf {
                    self.reader_buff.clear();
                    Ok(fields)
                  } else {
                    Err(VOTableError::WrongFieldNumber(nf, fields.len()))
                  }
                }),
            );
          }
          Event::End(e) if e.local_name() == b"TABLEDATA" => return None,
          Event::Eof => return Some(Err(VOTableError::PrematureEOF("TABLEDATA"))),
          Event::Text(e) if is_empty(e) => {}
          Event::Comment(e) => discard_comment(e, self.reader, "TABLEDATA"),
          _ => return Some(Err(unexpected_event(event, "TABLEDATA"))),
        },
      }
    }
  }
}

pub struct BinaryRowValueIterator<'a, R: BufRead> {
  table: &'a mut Table<VoidTableDataContent>,
  schema: Vec<Schema>,
  binary_deser: BinaryDeserializer<BufReader<DecoderReader<'a, GeneralPurpose, B64Cleaner<'a, R>>>>,
}

impl<'a, R: BufRead> BinaryRowValueIterator<'a, R> {
  pub fn new(
    reader: &'a mut Reader<R>,
    table: &'a mut Table<VoidTableDataContent>,
    schema: Vec<Schema>,
  ) -> Self {
    let internal_reader = reader.get_mut();
    let b64_cleaner = B64Cleaner::new(internal_reader);
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    let binary_deser = BinaryDeserializer::new(BufReader::new(decoder));
    Self {
      table,
      schema,
      binary_deser,
    }
  }
}

impl<'a, R: BufRead> TableIter for BinaryRowValueIterator<'a, R> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent> {
    self.table
  }

  fn read_to_end(self) -> Result<(), VOTableError> {
    let reader = self
      .binary_deser
      .into_inner()
      .into_inner()
      .into_inner()
      .into_inner();
    // Stream::<VoidTableDataContent>::TAG_BYTES,
    let mut stream_buf = [0_u8; 8];
    let mut bin_buf = [0_u8; 8]; // we could have reuse stream_buf, but not for BINARY2...
    skip_until(reader, b'<')
      .and_then(|_| reader.read_exact(&mut stream_buf))
      .map_err(VOTableError::Io)
      .and_then(|_| {
        if &stream_buf == b"/STREAM>" {
          Ok(())
        } else {
          Err(VOTableError::UnexpectedEndTag(
            (&stream_buf)[1..8].to_vec(),
            Stream::<VoidTableDataContent>::TAG,
          ))
        }
      })?;
    skip_until(reader, b'<')
      .and_then(|_| reader.read_exact(&mut bin_buf))
      .map_err(VOTableError::Io)
      .and_then(|_| {
        if &bin_buf == b"/BINARY>" {
          Ok(())
        } else {
          Err(VOTableError::UnexpectedEndTag(
            (&bin_buf)[1..8].to_vec(),
            Binary::<VoidTableDataContent>::TAG,
          ))
        }
      })
  }
}

impl<'a, R: BufRead> Iterator for BinaryRowValueIterator<'a, R> {
  type Item = Result<Vec<VOTableValue>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(true) = self.binary_deser.has_data_left() {
      let mut row: Vec<VOTableValue> = Vec::with_capacity(self.schema.len());
      for field_schema in self.schema.iter() {
        match field_schema.deserialize(&mut self.binary_deser) {
          Ok(field) => row.push(field),
          Err(e) => return Some(Err(e)),
        }
      }
      Some(Ok(std::mem::replace(
        &mut row,
        Vec::with_capacity(self.schema.len()),
      )))
    } else {
      None
    }
  }
}

pub struct Binary2RowValueIterator<'a, R: BufRead> {
  table: &'a mut Table<VoidTableDataContent>,
  schema: Vec<Schema>,
  binary_deser: BinaryDeserializer<BufReader<DecoderReader<'a, GeneralPurpose, B64Cleaner<'a, R>>>>,
  n_bytes: usize,
}

impl<'a, R: BufRead> Binary2RowValueIterator<'a, R> {
  pub fn new(
    reader: &'a mut Reader<R>,
    table: &'a mut Table<VoidTableDataContent>,
    schema: Vec<Schema>,
  ) -> Self {
    let internal_reader = reader.get_mut();
    let b64_cleaner = B64Cleaner::new(internal_reader);
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    let binary_deser = BinaryDeserializer::new(BufReader::new(decoder));
    let n_bytes = (schema.len() + 7) / 8;
    Self {
      table,
      schema,
      binary_deser,
      n_bytes,
    }
  }
}

impl<'a, R: BufRead> TableIter for Binary2RowValueIterator<'a, R> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent> {
    self.table
  }

  fn read_to_end(self) -> Result<(), VOTableError> {
    let reader = self
      .binary_deser
      .into_inner()
      .into_inner()
      .into_inner()
      .into_inner();
    // Stream::<VoidTableDataContent>::TAG_BYTES,
    let mut stream_buf = [0_u8; 8];
    let mut bin2_buf = [0_u8; 9]; // we could have reuse stream_buf, but not for BINARY2...

    skip_until(reader, b'<')
      .and_then(|_| reader.read_exact(&mut stream_buf))
      .map_err(VOTableError::Io)
      .and_then(|_| {
        if &stream_buf == b"/STREAM>" {
          Ok(())
        } else {
          Err(VOTableError::UnexpectedEndTag(
            (&stream_buf)[1..8].to_vec(),
            Stream::<VoidTableDataContent>::TAG,
          ))
        }
      })?;
    skip_until(reader, b'<')
      .and_then(|_| reader.read_exact(&mut bin2_buf))
      .map_err(VOTableError::Io)
      .and_then(|_| {
        if &bin2_buf == b"/BINARY2>" {
          Ok(())
        } else {
          Err(VOTableError::UnexpectedEndTag(
            (&bin2_buf)[1..9].to_vec(),
            Binary2::<VoidTableDataContent>::TAG,
          ))
        }
      })
  }
}

impl<'a, R: BufRead> Iterator for Binary2RowValueIterator<'a, R> {
  type Item = Result<Vec<VOTableValue>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(true) = self.binary_deser.has_data_left() {
      let mut row: Vec<VOTableValue> = Vec::with_capacity(self.schema.len());
      let bytes_visitor = FixedLengthArrayVisitor::new(self.n_bytes);
      let null_flags_res = (&mut self.binary_deser).deserialize_tuple(self.n_bytes, bytes_visitor);
      let null_flags: Vec<u8> = match null_flags_res {
        Ok(null_flags) => null_flags,
        Err(e) => return Some(Err(e)),
      };
      for (i_col, field_schema) in self.schema.iter().enumerate() {
        match field_schema.deserialize(&mut self.binary_deser) {
          Ok(field) => {
            let is_null = (null_flags[i_col >> 3] & (128_u8 >> (i_col & 7))) != 0;
            if is_null {
              row.push(VOTableValue::Null)
            } else {
              row.push(field)
            };
          }
          Err(e) => return Some(Err(e)),
        }
      }
      Some(Ok(std::mem::replace(
        &mut row,
        Vec::with_capacity(self.schema.len()),
      )))
    } else {
      None
    }
  }
}

// Method copied from https://doc.rust-lang.org/src/std/io/mod.rs since it is nightly (so far) :o/
fn skip_until<R: BufRead + ?Sized>(r: &mut R, delim: u8) -> std::io::Result<usize> {
  let mut read = 0;
  loop {
    let (done, used) = {
      let available = match r.fill_buf() {
        Ok(n) => n,
        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
        Err(e) => return Err(e),
      };
      match memchr::memchr(delim, available) {
        Some(i) => (true, i + 1),
        None => (false, available.len()),
      }
    };
    r.consume(used);
    read += used;
    if done || used == 0 {
      return Ok(read);
    }
  }
}
