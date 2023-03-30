
use std::{
  str,
  slice::Iter,
  io::BufRead
};

use base64::{
  read::DecoderReader,
  engine::general_purpose,
};

use quick_xml::{Reader, events::Event};

use serde::{
  Deserializer,
  de::DeserializeSeed
};

use crate::{
  is_empty,
  table::Table,
  error::VOTableError,
  impls::{
    Schema, VOTableValue, 
    mem::VoidTableDataContent,
    visitors::FixedLengthArrayVisitor,
    b64::read::{B64Cleaner, BinaryDeserializer},
  }
};
use crate::iter::TableIter;

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
    schema: Vec<Schema>
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
}

impl<'a, R: BufRead> Iterator for DataTableRowValueIterator<'a, R> {
  type Item = Result<Vec<VOTableValue>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff).map_err(VOTableError::Read);
      match event {
        Err(e) => return Some(Err(e)),
        Ok(mut event) =>
          match &mut event {
            Event::Start(ref e) =>
              match e.local_name() {
                b"TR" => {
                  let fit = DataTableFieldValueIterator { reader: self.reader, reader_buff: self.reader_buff, it_schema: self.schema.iter()};
                  let res = fit.collect::<Result<Vec<VOTableValue>, VOTableError>>();
                  self.reader_buff.clear();
                  return Some(res);
                },
                _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Row Iterator: {:?}", event)))),
              }
            Event::End(e) =>
              match e.local_name() {
                b"TABLEDATA" => return None,
                _ => return Some(Err(VOTableError::Custom(format!("Unexpected end of tag in Row Iterator '{:?}'", str::from_utf8(e.local_name()))))),
              }
            Event::Eof => return Some(Err(VOTableError::Custom(String::from("Premature end of file in Row Iterator")))),
            Event::Text(e) if is_empty(e) => {},
            _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
          }
      }
    }
  }
}

pub struct DataTableFieldValueIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
  it_schema: Iter<'a, Schema>,
}

impl<'a, R: BufRead> Iterator for DataTableFieldValueIterator<'a, R> {
  type Item = Result<VOTableValue, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff);
      match event {
        Err(e) => return Some(Err(VOTableError::Read(e))),
        Ok(mut event) =>
          match &mut event {
            Event::Start(ref e) =>
              match e.local_name() {
                b"TD" => {
                  let event = self.reader.read_event(self.reader_buff);
                  match event {
                    Err(e) => return Some(Err(VOTableError::Read(e))),
                    Ok(Event::Text(e)) => {
                      match e.unescape_and_decode(&self.reader) {
                        Err(e) => return Some(Err(VOTableError::Read(e))),
                        Ok(s) => match self.it_schema.next() {
                          Some(schema) => return Some(schema.value_from_str(s.trim())),
                          None => return Some(Err(VOTableError::Custom(String::from("More TDs than Field schemas...")))),
                        }
                      }
                    },
                    _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
                  }
                }
                _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
              }
            Event::Empty(e) if e.local_name() == b"TD" => return Some(Ok(VOTableValue::String(String::from("")))),
            Event::End(e) =>
              match e.local_name() {
                b"TD" => {},
                b"TR" => return None,
                _ => return Some(Err(VOTableError::Custom(format!("Unexpected end of tag in Field Iterator '{:?}'", str::from_utf8(e.local_name()))))),
              }
            Event::Eof => return Some(Err(VOTableError::Custom(String::from("Premature end of file in Field Iterator")))),
            Event::Text(e) if is_empty(e) => {},
            _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
          }
      }
    }
  }
}


pub struct BinaryRowValueIterator<'a, R: BufRead> {
  table: &'a mut Table<VoidTableDataContent>,
  schema: Vec<Schema>,
  binary_deser: BinaryDeserializer<'a, R>,
}

impl<'a, R: BufRead> BinaryRowValueIterator<'a, R> {

  pub fn new(
    reader: &'a mut Reader<R>,
    table: &'a mut Table<VoidTableDataContent>,
    schema: Vec<Schema>
  ) -> Self {
    let internal_reader = reader.get_mut();
    let b64_cleaner = B64Cleaner::new(internal_reader);
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    let binary_deser =  BinaryDeserializer::new(decoder);
    Self {
      table,
      schema,
      binary_deser
    }
  }
  
}

impl<'a, R: BufRead> TableIter for BinaryRowValueIterator<'a, R> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent> {
    self.table
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
      Some(Ok(std::mem::replace(&mut row, Vec::with_capacity(self.schema.len()))))
    } else {
      None
    }
  }
}



pub struct Binary2RowValueIterator<'a, R: BufRead> {
  table: &'a mut Table<VoidTableDataContent>,
  schema: Vec<Schema>,
  binary_deser: BinaryDeserializer<'a, R>,
  n_bytes: usize, 
}

impl<'a, R: BufRead> Binary2RowValueIterator<'a, R> {

  pub fn new(
    reader: &'a mut Reader<R>,
    table: &'a mut Table<VoidTableDataContent>,
    schema: Vec<Schema>
  ) -> Self {
    let internal_reader = reader.get_mut();
    let b64_cleaner = B64Cleaner::new(internal_reader);
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    let binary_deser =  BinaryDeserializer::new(decoder);
    let n_bytes = (schema.len() + 7) / 8;
    Self {
      table,
      schema,
      binary_deser,
      n_bytes
    }
  }

}

impl<'a, R: BufRead> TableIter for Binary2RowValueIterator<'a, R> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent> {
    self.table
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
        Ok(null_flags ) => null_flags,
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
          },
          Err(e) => return Some(Err(e)),
        }
      }
      Some(Ok(std::mem::replace(&mut row, Vec::with_capacity(self.schema.len()))))
    } else {
      None
    }
  }
}

