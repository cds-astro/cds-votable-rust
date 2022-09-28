
use std::{mem, io::{BufRead, Write}};

use quick_xml::{
  Reader, Writer, 
  events::{Event, BytesStart, BytesText}
};

use base64::read::DecoderReader;
use serde::{
  Deserializer,
  de::DeserializeSeed
};

use crate::impls::{
  b64::BinaryDeserializer,
  visitors::FixedLengthArrayVisitor
};

use super::super::{
  is_empty,
  TableDataContent, QuickXmlReadWrite,
  table::TableElem,
  data::tabledata::TableData,
  error::VOTableError,
  impls::{
    Schema, VOTableValue, b64::B64Cleaner
  }
};

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct InMemTableDataStringRows {
  rows: Vec<Vec<String>>,
}

impl InMemTableDataStringRows {
  pub fn new(rows: Vec<Vec<String>>) -> Self {
      Self { rows }
  }
}

/*
fn read_td_content<R: BufRead>(mut reader: Reader<R>, mut reader_buff: &mut Vec<u8>) -> Result<String, VOTableError> {
  let mut event = reader.read_event(&mut reader_buff).map_err(VOTableError::Read)?;
  match &mut event {
    Event::Text(e) => e.unescape_and_decode(&reader).map_err(VOTableError::Read),
    _ => Err(VOTableError::Custom(format!("Wring event in TD. Expected: Text. Actual: {:?}", event))),
  }
}*/


impl TableDataContent for InMemTableDataStringRows {
  fn read_datatable_content<R: BufRead>(&mut self, mut reader: Reader<R>, reader_buff: &mut Vec<u8>, context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    let mut row: Vec<String> = Vec::with_capacity(context.len());
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) =>
          match e.name() {
            b"TR" => {}
            b"TD" => {
              let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
              match &mut event {
                Event::Text(e) => row.push(e.unescape_and_decode(&reader).map_err(VOTableError::Read)?),
                _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
              }
            }
            _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
          }
        Event::Empty(e) if e.name() == b"TD" => row.push(String::from("")),
        Event::End(e) =>
          match e.name() {
            b"TD" => {}
            b"TR" => self.rows.push(mem::replace(&mut row, Vec::with_capacity(context.len()))),
            TableData::<Self>::TAG_BYTES => return Ok(reader),
            _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
          }
        Event::Eof => return Err(VOTableError::PrematureEOF(TableData::<Self>::TAG)),
        Event::Text(e) if is_empty(e) => { },
        _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
      }
    }
  }

  fn read_binary_content<R: BufRead>(&mut self, _reader: Reader<R>, _reader_buff: &mut Vec<u8>, _context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    Err(
      VOTableError::Custom(
        String::from("InMemTableDataStringRows not able to read/write BINARY data")
      )
    )
  }

  fn read_binary2_content<R: BufRead>(&mut self, _reader: Reader<R>, _reader_buff: &mut Vec<u8>, _context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    Err(
      VOTableError::Custom(
        String::from("InMemTableDataStringRows not able to read/write BINARY2 data")
      )
    )
  }

  fn write_in_datatable<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    let tr_tag = BytesStart::borrowed_name(b"TR");
    for row in &self.rows {
      writer.write_event(Event::Start(tr_tag.to_borrowed())).map_err(VOTableError::Write)?;
      for field in row {
        let elem_writer = writer.create_element(b"TD");
        elem_writer.write_text_content(
          BytesText::from_plain_str(field.as_str())
        ).map_err(VOTableError::Write)?;
      }
      writer.write_event(Event::End(tr_tag.to_end())).map_err(VOTableError::Write)?;
    }
    Ok(())
  }

  fn write_in_binary<W: Write>(&mut self, _writer: &mut Writer<W>) -> Result<(), VOTableError> {
    Err(
      VOTableError::Custom(
        String::from("InMemTableDataStringRows not able to read/write BINARY data")
      )
    )
  }

  fn write_in_binary2<W: Write>(&mut self, _writer: &mut Writer<W>) -> Result<(), VOTableError> {
    Err(
      VOTableError::Custom(
        String::from("InMemTableDataStringRows not able to read/write BINARY2 data")
      )
    )
  }
}


#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct InMemTableDataRows {
  rows: Vec<Vec<VOTableValue>>,
}

impl InMemTableDataRows {
  pub fn new(rows: Vec<Vec<VOTableValue>>) -> Self {
    Self { rows }
  }
}

/*
impl Visitor for InMemTableDataRows {
  type Value: Vec<Row>;

  fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
    todo!()
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, serde::de::Error> where A: SeqAccess<'de> {
    todo!()
  }
}

IMPL VISITOR ROW!!!
*/

impl TableDataContent for InMemTableDataRows {
  
  fn read_datatable_content<R: BufRead>(&mut self, mut reader: Reader<R>, reader_buff: &mut Vec<u8>, context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    let schema: Vec<Schema> = context.iter()
      .filter_map(|table_elem|
        match table_elem {
          TableElem::Field(field) =>  Some(field.into()),
          _ => None
        }
      ).collect();
    let mut row: Vec<VOTableValue> = Vec::with_capacity(schema.len());
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) =>
          match e.name() {
            b"TR" => { }
            b"TD" => {
              let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
              match &mut event {
                Event::Text(e) => {
                  let s = e.unescape_and_decode(&reader).map_err(VOTableError::Read)?;
                  let value = schema[row.len()].value_from_str(s.trim())?;
                  // eprintln!("Value: {}", s);
                  /* let value = serde_json::from_str(s.as_str().trim())
                    .map_err(|e| VOTableError::Custom(format!("JSON parse error: {:?}", e)))?;*/
                  row.push(value)
                },
                _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
              }
            }
            _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
          }
        Event::Empty(e) if e.name() == b"TD" => {
          row.push(VOTableValue::Null)
        },
        Event::End(e) =>
          match e.name() {
            b"TD" => {}
            b"TR" => self.rows.push(mem::replace(&mut row, Vec::with_capacity(context.len()))),
            TableData::<Self>::TAG_BYTES => return Ok(reader),
            _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
          }
        Event::Eof => return Err(VOTableError::PrematureEOF(TableData::<Self>::TAG)),
        Event::Text(e) if is_empty(e) => { },
        _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
      }
    }
  }

  fn read_binary_content<R: BufRead>(&mut self, mut reader: Reader<R>, _reader_buff: &mut Vec<u8>, context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    // Prepare reader
    let mut internal_reader = reader.get_mut();
    let mut b64_cleaner = B64Cleaner::new(&mut internal_reader);
    let decoder = DecoderReader::new(&mut b64_cleaner, base64::STANDARD);
    let mut binary_deser =  BinaryDeserializer::new(decoder);
    // Get schema
    let schema: Vec<Schema> = context.iter()
      .filter_map(|table_elem|
        match table_elem {
          TableElem::Field(field) =>  Some(field.into()),
          _ => None
        }
      ).collect();
    // Read rows
    while let Ok(true) = binary_deser.has_data_left() {
      let mut row: Vec<VOTableValue> = Vec::with_capacity(schema.len());
      for field_schema in schema.iter() {
        let field = field_schema.deserialize(&mut binary_deser)?;
        row.push(field);
      }
      self.rows.push(mem::replace(&mut row, Vec::with_capacity(schema.len())));
    }
    Ok(reader)
  }

  fn read_binary2_content<R: BufRead>(&mut self, mut reader: Reader<R>, _reader_buff: &mut Vec<u8>, context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    // Prepare reader
    let mut internal_reader = reader.get_mut();
    let mut b64_cleaner = B64Cleaner::new(&mut internal_reader);
    let decoder = DecoderReader::new(&mut b64_cleaner, base64::STANDARD);
    let mut binary_deser =  BinaryDeserializer::new(decoder);
    // Get schema
    let schema: Vec<Schema> = context.iter()
      .filter_map(|table_elem|
        match table_elem {
          TableElem::Field(field) =>  Some(field.into()),
          _ => None
        }
      ).collect();
    // Read rows
    let n_bytes = (schema.len() + 7) / 8;
    while let Ok(true) = binary_deser.has_data_left() {
      let mut row: Vec<VOTableValue> = Vec::with_capacity(schema.len());
      let bytes_visitor = FixedLengthArrayVisitor::new(n_bytes);
      let null_flags: Vec<u8> = (&mut binary_deser).deserialize_tuple(n_bytes, bytes_visitor)?;
      for (i_col, field_schema) in schema.iter().enumerate() {
        let field = field_schema.deserialize(&mut binary_deser)?;
        let is_null = (null_flags[i_col >> 3] & (128_u8 >> i_col.rem_euclid(8))) != 0;
        if is_null {
          row.push(VOTableValue::Null) 
        } else {
          row.push(field)
        };
      }
      self.rows.push(mem::replace(&mut row, Vec::with_capacity(schema.len())));
    }
    Ok(reader)
  }

  fn write_in_datatable<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    let tr_tag = BytesStart::borrowed_name(b"TR");
    for row in &self.rows {
      writer.write_event(Event::Start(tr_tag.to_borrowed())).map_err(VOTableError::Write)?;
      for field in row {
        let elem_writer = writer.create_element(b"TD");
        elem_writer.write_text_content(
          BytesText::from_plain_str(field.to_string().as_str())
        ).map_err(VOTableError::Write)?;
      }
      writer.write_event(Event::End(tr_tag.to_end())).map_err(VOTableError::Write)?;
    }
    Ok(())
  }

  fn write_in_binary<W: Write>(&mut self, _writer: &mut Writer<W>) -> Result<(), VOTableError> {
    todo!()
  }

  fn write_in_binary2<W: Write>(&mut self, _writer: &mut Writer<W>) -> Result<(), VOTableError> {
    todo!()
  }
}
