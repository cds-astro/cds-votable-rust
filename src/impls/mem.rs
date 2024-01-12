use std::io::BufReader;
use std::{
  io::{BufRead, Write},
  mem,
};

use quick_xml::{
  events::{BytesStart, BytesText, Event},
  Reader, Writer,
};

use base64::{engine::general_purpose, read::DecoderReader, write::EncoderWriter};

use serde::{de::DeserializeSeed, ser::SerializeTuple, Deserializer, Serializer};

use crate::impls::TableSchema;
use crate::{
  data::tabledata::TableData,
  error::VOTableError,
  impls::{
    b64::{
      read::{B64Cleaner, BinaryDeserializer},
      write::{B64Formatter, BinarySerializer},
    },
    visitors::FixedLengthArrayVisitor,
    Schema, VOTableValue,
  },
  is_empty,
  table::TableElem,
  QuickXmlReadWrite, TableDataContent,
};

/// Do not parse/contains any data.
/// Only made for VOTable parsers taking charge of parsing (and dealing with) the data part
/// of the VOTable.
#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct VoidTableDataContent(());

impl TableDataContent for VoidTableDataContent {
  fn ensures_consistency(&mut self, _context: &[TableElem]) -> Result<(), String> {
    Ok(())
  }

  fn read_datatable_content<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Read/write not implemented for VoidTableDataContent",
    )))
  }

  fn read_binary_content<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Read/write not implemented for VoidTableDataContent",
    )))
  }

  fn read_binary2_content<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Read/write not implemented for VoidTableDataContent",
    )))
  }

  fn write_in_datatable<W: Write>(
    &mut self,
    _writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Read/write not implemented for VoidTableDataContent",
    )))
  }

  fn write_in_binary<W: Write>(
    &mut self,
    _writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Read/write not implemented for VoidTableDataContent",
    )))
  }

  fn write_in_binary2<W: Write>(
    &mut self,
    _writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Read/write not implemented for VoidTableDataContent",
    )))
  }
}

/// Save in memory all rows in a vector.
/// Each row is itself a vector of field.
/// Each field is a String (this is not memory efficient, if the full VOTable fits in memory,
/// we should save each field as a `&str`).
#[derive(Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InMemTableDataStringRows {
  rows: Vec<Vec<String>>,
}

impl InMemTableDataStringRows {
  pub fn new(rows: Vec<Vec<String>>) -> Self {
    Self { rows }
  }
}

impl TableDataContent for InMemTableDataStringRows {
  fn ensures_consistency(&mut self, _context: &[TableElem]) -> Result<(), String> {
    // No need to ensure consistency since everything is string...
    Ok(())
  }

  fn read_datatable_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    let mut row: Vec<String> = Vec::with_capacity(context.len());
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          b"TR" => {}
          b"TD" => {
            let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
            match &mut event {
              Event::Text(e) => {
                row.push(e.unescape_and_decode(reader).map_err(VOTableError::Read)?)
              }
              _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
            }
          }
          _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
        },
        Event::Empty(e) if e.local_name() == b"TD" => row.push(String::from("")),
        Event::End(e) => match e.local_name() {
          b"TD" => {}
          b"TR" => self
            .rows
            .push(mem::replace(&mut row, Vec::with_capacity(context.len()))),
          TableData::<Self>::TAG_BYTES => return Ok(()),
          _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
        },
        Event::Eof => return Err(VOTableError::PrematureEOF(TableData::<Self>::TAG)),
        Event::Text(e) if is_empty(e) => {}
        _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
      }
    }
  }

  fn read_binary_content<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "InMemTableDataStringRows not able to read/write BINARY data",
    )))
  }

  fn read_binary2_content<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "InMemTableDataStringRows not able to read/write BINARY2 data",
    )))
  }

  fn write_in_datatable<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    let tr_tag = BytesStart::borrowed_name(b"TR");
    for row in &self.rows {
      writer
        .write_event(Event::Start(tr_tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      for field in row {
        let elem_writer = writer.create_element(b"TD");
        elem_writer
          .write_text_content(BytesText::from_plain_str(field.as_str()))
          .map_err(VOTableError::Write)?;
      }
      writer
        .write_event(Event::End(tr_tag.to_end()))
        .map_err(VOTableError::Write)?;
    }
    Ok(())
  }

  fn write_in_binary<W: Write>(
    &mut self,
    _writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "InMemTableDataStringRows not able to read/write BINARY data",
    )))
  }

  fn write_in_binary2<W: Write>(
    &mut self,
    _writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "InMemTableDataStringRows not able to read/write BINARY2 data",
    )))
  }
}

/// Save in memory all rows in a vector.
/// Each row is itself a vector of field.
/// Each field is `VOTableValue` parse according to the table schema.
#[derive(Default, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
// #[derive(Default, Debug, PartialEq, serde::Serialize)]
pub struct InMemTableDataRows {
  rows: Vec<Vec<VOTableValue>>,
}

impl InMemTableDataRows {
  pub fn new(rows: Vec<Vec<VOTableValue>>) -> Self {
    Self { rows }
  }
}

// Implement Deserialize using the &[Schema] which implement DeserializeSeed...

impl TableDataContent for InMemTableDataRows {
  fn ensures_consistency(&mut self, context: &[TableElem]) -> Result<(), String> {
    let table_schema = TableSchema::from(context);
    for row in self.rows.iter_mut() {
      for (schema, value) in table_schema.iter().zip(row.iter_mut()) {
        schema.replace_by_proper_value_if_necessary(value)?;
      }
    }
    Ok(())
  }

  fn read_datatable_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    let schema: Vec<Schema> = TableSchema::from(context).unwrap();
    let mut row: Vec<VOTableValue> = Vec::with_capacity(schema.len());
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          b"TR" => {}
          b"TD" => {
            let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
            match &mut event {
              Event::Text(e) => {
                let s = e.unescape_and_decode(reader).map_err(VOTableError::Read)?;
                let value = schema[row.len()].value_from_str(s.trim())?;
                // eprintln!("Value: {}", s);
                /* let value = serde_json::from_str(s.as_str().trim())
                .map_err(|e| VOTableError::Custom(format!("JSON parse error: {:?}", e)))?;*/
                row.push(value)
              }
              _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
            }
          }
          _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
        },
        Event::Empty(e) if e.local_name() == b"TD" => row.push(VOTableValue::Null),
        Event::End(e) => match e.local_name() {
          b"TD" => {}
          b"TR" => self
            .rows
            .push(mem::replace(&mut row, Vec::with_capacity(context.len()))),
          TableData::<Self>::TAG_BYTES => return Ok(()),
          _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
        },
        Event::Eof => return Err(VOTableError::PrematureEOF(TableData::<Self>::TAG)),
        Event::Text(e) if is_empty(e) => {}
        _ => eprintln!("Discarded event in {}: {:?}", TableData::<Self>::TAG, event),
      }
    }
  }

  fn read_binary_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    // Prepare reader
    let mut internal_reader = reader.get_mut();
    let b64_cleaner = B64Cleaner::new(&mut internal_reader);
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    let mut binary_deser = BinaryDeserializer::new(BufReader::new(decoder));
    // Get schema
    let schema: Vec<Schema> = TableSchema::from(context).unwrap();
    // Read rows
    while let Ok(true) = binary_deser.has_data_left() {
      let mut row: Vec<VOTableValue> = Vec::with_capacity(schema.len());
      for field_schema in schema.iter() {
        let field = field_schema.deserialize(&mut binary_deser)?;
        row.push(field);
      }
      self
        .rows
        .push(mem::replace(&mut row, Vec::with_capacity(schema.len())));
    }
    Ok(())
  }

  fn read_binary2_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    // Prepare reader
    let mut internal_reader = reader.get_mut();
    let b64_cleaner = B64Cleaner::new(&mut internal_reader);
    let decoder = DecoderReader::new(b64_cleaner, &general_purpose::STANDARD);
    let mut binary_deser = BinaryDeserializer::new(BufReader::new(decoder));
    // Get schema
    let schema: Vec<Schema> = TableSchema::from(context).unwrap();
    // Read rows
    let n_bytes = (schema.len() + 7) / 8;
    while let Ok(true) = binary_deser.has_data_left() {
      let mut row: Vec<VOTableValue> = Vec::with_capacity(schema.len());
      let bytes_visitor = FixedLengthArrayVisitor::new(n_bytes);
      let null_flags: Vec<u8> = (&mut binary_deser).deserialize_tuple(n_bytes, bytes_visitor)?;
      for (i_col, field_schema) in schema.iter().enumerate() {
        let field = field_schema.deserialize(&mut binary_deser)?;
        let is_null = (null_flags[i_col >> 3] & (128_u8 >> (i_col & 7))) != 0;
        if is_null {
          row.push(VOTableValue::Null)
        } else {
          row.push(field)
        };
      }
      self
        .rows
        .push(mem::replace(&mut row, Vec::with_capacity(schema.len())));
    }
    Ok(())
  }

  fn write_in_datatable<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &[TableElem],
  ) -> Result<(), VOTableError> {
    let tr_tag = BytesStart::borrowed_name(b"TR");
    for row in &self.rows {
      writer
        .write_event(Event::Start(tr_tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      for field in row {
        let elem_writer = writer.create_element(b"TD");
        elem_writer
          .write_text_content(BytesText::from_plain_str(field.to_string().as_str()))
          .map_err(VOTableError::Write)?;
      }
      writer
        .write_event(Event::End(tr_tag.to_end()))
        .map_err(VOTableError::Write)?;
    }
    Ok(())
  }

  fn write_in_binary<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    // Get FIELD schema (ignoring PARAMS, ...)
    let schema: Vec<Schema> = TableSchema::from(context).unwrap();
    // Create serializer
    let mut serializer = BinarySerializer::new(EncoderWriter::new(
      B64Formatter::new(writer.inner()),
      &general_purpose::STANDARD,
    ));
    // Write data
    for row in &self.rows {
      for (field_ref, schema_ref) in row.iter().zip(schema.iter()) {
        schema_ref.serialize_seed(field_ref, &mut serializer)?;
      }
    }
    Ok(())
  }

  fn write_in_binary2<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    // Get FIELD schema (ignoring PARAMS, ...)
    let schema: Vec<Schema> = TableSchema::from(context).unwrap();
    // Compute size of null flags
    let n_null_flag_bytes = (schema.len() + 7) / 8;
    // Create serializer
    let mut serializer = BinarySerializer::new(EncoderWriter::new(
      B64Formatter::new(writer.inner()),
      &general_purpose::STANDARD,
    ));
    // Write data
    for row in &self.rows {
      // Check null values
      let mut null_flags = vec![0_u8; n_null_flag_bytes];
      for (i, field) in row.iter().enumerate() {
        if matches!(field, VOTableValue::Null) {
          null_flags[i >> 3] |= 128_u8 >> (i & 7);
        }
      }
      // Write null falgs
      let mut seq_ser = serializer.serialize_tuple(n_null_flag_bytes)?;
      for byte in null_flags {
        SerializeTuple::serialize_element(&mut seq_ser, &byte)?;
      }
      SerializeTuple::end(seq_ser)?;
      // Write remaining
      for (field_ref, schema_ref) in row.iter().zip(schema.iter()) {
        schema_ref.serialize_seed(field_ref, &mut serializer)?;
      }
    }
    Ok(())
  }
}
