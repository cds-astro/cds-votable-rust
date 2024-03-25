use std::{
  io::{BufRead, BufReader, Write},
  mem,
};

use base64::{engine::general_purpose, read::DecoderReader, write::EncoderWriter};
use quick_xml::{
  events::{BytesStart, BytesText, Event},
  Reader, Writer,
};
use serde::{de::DeserializeSeed, ser::SerializeTuple, Deserializer, Serializer};

use crate::{
  data::tabledata::{parse_fields, FieldIterator, TableData, EOTR},
  error::VOTableError,
  impls::TableSchema,
  impls::{
    b64::{
      read::{B64Cleaner, BinaryDeserializer},
      write::{B64Formatter, BinarySerializer},
    },
    visitors::FixedLengthArrayVisitor,
    Schema, VOTableValue,
  },
  table::TableElem,
  utils::{discard_comment, is_empty, unexpected_event},
  TableDataContent, VOTableElement,
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
    let n_fields = context
      .iter()
      .filter(|e| matches!(e, TableElem::Field(_)))
      .count();
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) if e.local_name() == b"TR" => {
          let fields = parse_fields::<_, EOTR>(reader, reader_buff, n_fields)?;
          if fields.len() == n_fields {
            self.rows.push(fields);
          } else {
            return Err(VOTableError::WrongFieldNumber(n_fields, fields.len()));
          }
        }
        Event::End(e) if e.local_name() == TableData::<Self>::TAG_BYTES => {
          reader_buff.clear();
          return Ok(());
        }
        Event::Text(e) if is_empty(e) => {}
        Event::Comment(e) => discard_comment(e, reader, TableData::<Self>::TAG),
        Event::Eof => return Err(VOTableError::PrematureEOF(TableData::<Self>::TAG)),
        _ => return Err(unexpected_event(event, TableData::<Self>::TAG)),
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
pub struct InMemTableDataRows {
  rows: Vec<Vec<VOTableValue>>,
}

impl InMemTableDataRows {
  pub fn new(rows: Vec<Vec<VOTableValue>>) -> Self {
    Self { rows }
  }

  pub fn write_tabledata_rows<W, I>(
    writer: &mut Writer<W>,
    iterator: I,
    schema: TableSchema,
  ) -> Result<(), VOTableError>
  where
    W: Write,
    I: Iterator,
    I::Item: AsRef<[VOTableValue]>,
  {
    let tr_tag = BytesStart::borrowed_name(b"TR");
    for row_ref in iterator {
      writer
        .write_event(Event::Start(tr_tag.to_borrowed()))
        .map_err(VOTableError::Write)
        .and_then(|_| Self::write_tabledata_row(writer, row_ref.as_ref().iter(), &schema))
        .and_then(|_| {
          writer
            .write_event(Event::End(tr_tag.to_end()))
            .map_err(VOTableError::Write)
        })?;
    }
    Ok(())
  }

  pub fn write_tabledata_row<W, I>(
    writer: &mut Writer<W>,
    row_it: I,
    _schema: &TableSchema,
  ) -> Result<(), VOTableError>
  where
    W: Write,
    I: Iterator,
    I::Item: AsRef<VOTableValue>,
  {
    // The schema could be used to avoid checking for characters to escape in case of integer or float
    // (for instance). But I am not sure it is worth it...
    for field in row_it {
      writer
        .create_element(b"TD")
        .write_text_content(BytesText::from_plain_str(
          field.as_ref().to_string().as_str(),
        ))
        .map_err(VOTableError::Write)?;
    }
    Ok(())
  }

  /// Write in the given `writer` all rows in the given `iterator` in Base64 according to
  /// the `BINARY` scheme.
  pub fn write_binary_rows<W, I>(
    writer: W,
    iterator: I,
    schema: TableSchema,
  ) -> Result<(), VOTableError>
  where
    W: Write,
    I: Iterator,
    I::Item: AsRef<[VOTableValue]>,
  {
    // Create serializer
    let mut serializer = BinarySerializer::new(EncoderWriter::new(
      B64Formatter::new(writer),
      &general_purpose::STANDARD,
    ));
    // Write data
    for row_ref in iterator {
      Self::write_binary_row(&mut serializer, row_ref.as_ref().iter(), &schema)?;
    }
    Ok(())
  }

  /// Write in the given `writer` all rows in the given `iterator` in Base64 according to
  /// the `BINARY2` scheme.
  pub fn write_binary2_rows<W, I>(
    writer: W,
    iterator: I,
    schema: TableSchema,
  ) -> Result<(), VOTableError>
  where
    W: Write,
    I: Iterator,
    I::Item: AsRef<[VOTableValue]>,
  {
    // Create serializer
    let mut serializer = BinarySerializer::new(EncoderWriter::new(
      B64Formatter::new(writer),
      &general_purpose::STANDARD,
    ));
    // Write data
    for row_ref in iterator {
      Self::write_binary2_row(&mut serializer, row_ref, &schema)?;
    }
    Ok(())
  }

  pub fn write_binary_row<W, I>(
    serializer: &mut BinarySerializer<W>,
    row_it: I,
    schema: &TableSchema,
  ) -> Result<(), VOTableError>
  where
    W: Write,
    I: Iterator,
    I::Item: AsRef<VOTableValue>,
  {
    let n_expected = schema.0.len();
    let mut n_actual = 0;
    for (field_ref, schema_ref) in row_it.zip(schema.0.iter()) {
      schema_ref.serialize_seed(field_ref.as_ref(), &mut *serializer)?;
      n_actual += 1;
    }
    if n_expected == n_actual {
      Ok(())
    } else {
      Err(VOTableError::WrongFieldNumber(n_expected, n_actual))
    }
  }

  pub fn write_binary2_row<W, R>(
    serializer: &mut BinarySerializer<W>,
    row: R,
    schema: &TableSchema,
  ) -> Result<(), VOTableError>
  where
    W: Write,
    R: AsRef<[VOTableValue]>,
  {
    let row = row.as_ref();
    // Compute size of null flags
    let n_null_flag_bytes = (schema.0.len() + 7) / 8;
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
    Self::write_binary_row(serializer, row.iter(), schema)
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
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) if e.local_name() == b"TR" => {
          let fields = FieldIterator::new(reader, reader_buff)
            .zip(schema.iter())
            .map(|(f_res, s)| f_res.and_then(|f| s.value_from_str(f.trim())))
            .collect::<Result<Vec<VOTableValue>, VOTableError>>()?;
          if fields.len() == schema.len() {
            self.rows.push(fields);
          } else {
            return Err(VOTableError::WrongFieldNumber(schema.len(), fields.len()));
          }
        }
        Event::End(e) if e.local_name() == TableData::<Self>::TAG_BYTES => {
          reader_buff.clear();
          return Ok(());
        }
        Event::Text(e) if is_empty(e) => {}
        Event::Comment(e) => discard_comment(e, reader, TableData::<Self>::TAG),
        Event::Eof => return Err(VOTableError::PrematureEOF(TableData::<Self>::TAG)),
        _ => return Err(unexpected_event(event, TableData::<Self>::TAG)),
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
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Self::write_tabledata_rows(writer, self.rows.iter(), TableSchema::from(context))
  }

  fn write_in_binary<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Self::write_binary_rows(writer.inner(), self.rows.iter(), TableSchema::from(context))
  }

  fn write_in_binary2<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError> {
    Self::write_binary2_rows(writer.inner(), self.rows.iter(), TableSchema::from(context))
  }
}
