//! Single (large) Table Streaming Converter.

use std::{
  fs::File,
  io::{stdin, stdout, BufRead, BufReader, BufWriter, Cursor, Write},
  path::PathBuf,
  str::FromStr,
  thread::scope,
};

use clap::Args;
use crossbeam::channel::bounded;
use serde::{de::DeserializeSeed, Deserializer};

use votable::{
  data::{tabledata::FieldIteratorUnbuffered, TableOrBinOrBin2},
  error::VOTableError,
  impls::{
    b64::{
      read::BinaryDeserializer,
      write::{general_purpose, B64Formatter, BinarySerializer, EncoderWriter},
    },
    mem::InMemTableDataRows,
    visitors::FixedLengthArrayVisitor,
    TableSchema, VOTableValue,
  },
  iter::SimpleVOTableRowIterator,
  votable::new_xml_writer,
  TableElem, VOTable, VoidTableDataContent,
};

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
  XmlTabledata,
  XmlBinary,
  XmlBinary2,
  CSV,
}
impl FromStr for OutputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "xml-td" => Ok(OutputFormat::XmlTabledata),
      "xml-bin" => Ok(OutputFormat::XmlBinary),
      "xml-bin2" => Ok(OutputFormat::XmlBinary2),
      "csv" => Ok(OutputFormat::CSV),
      _ => Err(format!(
        "Unrecognized format. Actual: '{}'. Expected: 'xml-td', 'xml-bin', 'xml-bin2', or 'csv'",
        s
      )),
    }
  }
}

/// Convert a single table XML VOTable in streaming mode.
/// Tags after `</TABLE>` are preserved.
#[derive(Debug, Args)]
pub struct StreamConvert {
  /// Path of the input XML VOTable [default: read from stdin]
  #[clap(short = 'i', long = "in", value_name = "FILE")]
  input: Option<PathBuf>,
  /// Path of the output file [default: write to stdout]
  #[clap(short = 'o', long = "out", value_name = "FILE")]
  output: Option<PathBuf>,
  /// Format of the output file ('xml-td', 'xml-bin', 'xml-bin2' or 'csv').
  #[clap(short = 'f', long = "out-fmt", value_enum)]
  output_fmt: OutputFormat,
  /// Separator used for the 'csv' format.
  #[arg(short, long, default_value_t = ',')]
  separator: char,
  /// Exec concurrently using N threads (row order not preserved!)
  #[arg(long, value_name = "N")]
  parallel: Option<usize>,
  /// Number of rows process by a same thread in `parallel` mode
  #[arg(long, default_value_t = 10_000_usize)]
  chunk_size: usize,
}

impl StreamConvert {
  pub fn exec(self) -> Result<(), VOTableError> {
    self.choose_input_and_exec()
  }

  pub fn choose_input_and_exec(self) -> Result<(), VOTableError> {
    match &self.input {
      Some(path) => {
        SimpleVOTableRowIterator::from_file(path).and_then(|it| self.choose_output_and_exec(it))
      }
      None => {
        let stdin = stdin();
        // let handle = stdin.lock();
        SimpleVOTableRowIterator::from_reader(BufReader::new(stdin))
          .and_then(|it| self.choose_output_and_exec(it))
      }
    }
  }

  pub fn choose_output_and_exec<R: BufRead + Send>(
    self,
    it: SimpleVOTableRowIterator<R>,
  ) -> Result<(), VOTableError> {
    match &self.output {
      Some(path) => {
        let file = File::create(path).map_err(VOTableError::Io)?;
        let write = BufWriter::new(file);
        self.do_exec_gen(it, write)
      }
      None => {
        let stdout = stdout();
        let handle = stdout.lock();
        self.do_exec_gen(it, handle)
      }
    }
  }

  pub fn do_exec_gen<R, W>(
    self,
    it: SimpleVOTableRowIterator<R>,
    mut write: W,
  ) -> Result<(), VOTableError>
  where
    R: BufRead + Send,
    W: Write,
  {
    match it.data_type() {
      // In binary64, 6 bit  -> 1 ASCII Char
      //   3 bytes = 24 bits -> 4 ASCII Chars
      TableOrBinOrBin2::TableData => {
        match self.output_fmt {
          OutputFormat::XmlTabledata => to_same(it, write),
          OutputFormat::XmlBinary => match self.parallel {
            None => to_binary(it, write),
            Some(n_threads) => td_to_binary_par(it, write, n_threads, self.chunk_size),
          },
          OutputFormat::XmlBinary2 => match self.parallel {
            None => to_binary2(it, write),
            Some(n_threads) => td_to_binary2_par(it, write, n_threads, self.chunk_size),
          },
          OutputFormat::CSV => {
            let mut raw_row_it = it.to_owned_tabledata_row_iterator();
            // Write header
            write_csv_header(&raw_row_it.votable, &mut write, self.separator)?;
            // Write data
            match self.parallel {
              None => {
                while let Some(row) = raw_row_it.next().transpose()? {
                  let mut field_it = // tdrow2strfields(row.as_slice());
                    FieldIteratorUnbuffered::new(row.as_slice()).map(|res| match res {
                      Ok(s) => s,
                      Err(e) => panic!("Error parsing a row: {:?}", e),
                    });
                  if let Some(field) = field_it.next() {
                    write_1st_csv_field_with_newline(&mut write, field.as_str(), self.separator)?;
                    for field in field_it {
                      write_csv_field(&mut write, field.as_str(), self.separator)?;
                    }
                  }
                }
                // End with a newline
                write.write_all(b"\n").map_err(VOTableError::Io)
              }
              Some(n_threads) => {
                let schema = get_schema(&raw_row_it.votable);
                convert_par(
                  &mut raw_row_it,
                  schema,
                  tdrow2csvrow,
                  write,
                  self.separator,
                  n_threads,
                  self.chunk_size,
                )
              }
            }
          }
        }
      }
      TableOrBinOrBin2::Binary => match self.output_fmt {
        OutputFormat::XmlTabledata => match self.parallel {
          None => to_tabledata(it, write),
          Some(n_threads) => binary_to_td_par(it, write, n_threads, self.chunk_size),
        },
        OutputFormat::XmlBinary => to_same(it, write),
        OutputFormat::XmlBinary2 => match self.parallel {
          None => to_binary2(it, write),
          Some(n_threads) => binary_to_binary2_par(it, write, n_threads, self.chunk_size),
        },
        OutputFormat::CSV => match self.parallel {
          None => to_csv(it, write, self.separator),
          Some(n_threads) => {
            let mut raw_row_it = it.to_owned_binary_row_iterator();
            let schema = get_schema(&raw_row_it.votable);
            // Write header
            write_csv_header(&raw_row_it.votable, &mut write, self.separator)?;
            // Write data
            convert_par(
              &mut raw_row_it,
              schema,
              binrow2csvrow,
              write,
              self.separator,
              n_threads,
              self.chunk_size,
            )
          }
        },
      },
      TableOrBinOrBin2::Binary2 => match self.output_fmt {
        OutputFormat::XmlTabledata => match self.parallel {
          None => to_tabledata(it, write),
          Some(n_threads) => binary2_to_td_par(it, write, n_threads, self.chunk_size),
        },
        OutputFormat::XmlBinary => match self.parallel {
          None => to_binary(it, write),
          Some(n_threads) => binary2_to_binary_par(it, write, n_threads, self.chunk_size),
        },
        OutputFormat::XmlBinary2 => to_same(it, write),
        OutputFormat::CSV => match self.parallel {
          None => to_csv(it, write, self.separator),
          Some(n_threads) => {
            let mut raw_row_it = it.to_owned_binary2_row_iterator();
            let schema = get_schema(&raw_row_it.votable);
            // Write header
            write_csv_header(&raw_row_it.votable, &mut write, self.separator)?;
            // Write data
            convert_par(
              &mut raw_row_it,
              schema,
              bin2row2csvrow,
              write,
              self.separator,
              n_threads,
              self.chunk_size,
            )
          }
        },
      },
      _ => unreachable!(), // Error before reaching this point.
    }
  }
}

/// # Panics
/// if the given VOTable does not contain a table.
fn get_schema(votable: &VOTable<VoidTableDataContent>) -> TableSchema {
  TableSchema::from(
    votable
      .get_first_table()
      .expect("No table found!")
      .elems
      .as_slice(),
  )
}

/// # Panics
/// if the given VOTable does not contain a table.
fn get_colnames(votable: &VOTable<VoidTableDataContent>) -> Vec<String> {
  votable
    .get_first_table()
    .expect("No table found!")
    .elems
    .iter()
    .filter_map(|table_elem| match table_elem {
      TableElem::Field(field) => Some(field.name.clone()),
      _ => None,
    })
    .collect()
}

fn write_1st_csv_field<W: Write>(
  write: &mut W,
  field: &str,
  sep: char,
) -> Result<(), VOTableError> {
  if need_double_quotes(field, sep) {
    write_double_quoted_field(field, write)
  } else {
    write.write_all(field.as_bytes())
  }
  .map_err(VOTableError::Io)
}
fn write_1st_csv_field_with_newline<W: Write>(
  write: &mut W,
  field: &str,
  sep: char,
) -> Result<(), VOTableError> {
  write
    .write_all(b"\n")
    .map_err(VOTableError::Io)
    .and_then(|_| write_1st_csv_field(write, field, sep))
}
fn write_csv_field<W: Write>(write: &mut W, field: &str, sep: char) -> Result<(), VOTableError> {
  if need_double_quotes(field, sep) {
    write
      .write_all(b",")
      .and_then(|_| write_double_quoted_field(field, write))
  } else {
    write.write_fmt(format_args!(",{}", field))
  }
  .map_err(VOTableError::Io)
}
fn need_double_quotes(field: &str, sep: char) -> bool {
  field.contains(sep) && !(field.starts_with('"') && field.ends_with('"'))
}
/// According to [RFC-4180](https://www.ietf.org/rfc/rfc4180.txt):
/// > If double-quotes are used to enclose fields, then a double-quote
/// > appearing inside a field must be escaped by preceding it with
/// > another double quote.
fn write_double_quoted_field<W: Write>(field: &str, write: &mut W) -> Result<(), std::io::Error> {
  if field.contains('"') {
    write.write_fmt(format_args!("\"{}\"", field.replace('"', "\"\"")))
  } else {
    write.write_fmt(format_args!("\"{}\"", field))
  }
}

fn tdrow2csvrow(bytes: &[u8], _schema: &TableSchema, sep: char) -> Box<[u8]> {
  fieldit2csvrow(
    FieldIteratorUnbuffered::new(bytes).map(|res| match res {
      Ok(s) => s,
      Err(e) => panic!("Error parsing a row: {:?}", e),
    }),
    sep,
  )
}

fn binrow2fieldit<'a>(
  bytes: &'a [u8],
  schema: &'a TableSchema,
) -> impl Iterator<Item = VOTableValue> + 'a {
  let mut binary_deser = BinaryDeserializer::new(Cursor::new(bytes));
  schema
    .iter()
    .map(move |field_schema| field_schema.deserialize(&mut binary_deser).unwrap())
}

fn binrow2csvrow(bytes: &[u8], schema: &TableSchema, sep: char) -> Box<[u8]> {
  fieldit2csvrow(
    binrow2fieldit(bytes, schema).map(|field| field.to_string()),
    sep,
  )
}

fn bin2row2fieldit<'a>(
  bytes: &'a [u8],
  schema: &'a TableSchema,
) -> impl Iterator<Item = VOTableValue> + 'a {
  let n_bytes = (schema.as_slice().len() + 7) / 8;
  let mut binary_deser = BinaryDeserializer::new(Cursor::new(bytes));
  let bytes_visitor = FixedLengthArrayVisitor::new(n_bytes);
  let null_flags: Vec<u8> = (&mut binary_deser)
    .deserialize_tuple(n_bytes, bytes_visitor)
    .unwrap();
  schema.iter().enumerate().map(move |(i_col, field_schema)| {
    let field = field_schema.deserialize(&mut binary_deser).unwrap();
    let is_null = (null_flags[i_col >> 3] & (128_u8 >> (i_col & 7))) != 0;
    if is_null {
      VOTableValue::Null
    } else {
      field
    }
  })
}

fn bin2row2csvrow(bytes: &[u8], schema: &TableSchema, sep: char) -> Box<[u8]> {
  fieldit2csvrow(
    bin2row2fieldit(bytes, schema).map(|field| field.to_string()),
    sep,
  )
}

fn fieldit2csvrow<I, S>(mut it: I, sep: char) -> Box<[u8]>
where
  S: AsRef<str>,
  I: Iterator<Item = S>,
{
  fn push_field(field: &str, sep: char, buff: &mut String) {
    if need_double_quotes(field, sep) {
      buff.push('"');
      if field.contains('"') {
        buff.push_str(field.replace('"', "\"\"").as_str());
      } else {
        buff.push_str(field);
      }
      buff.push('"')
    } else {
      buff.push_str(field);
    }
  }
  let mut res: String = String::with_capacity(512);
  if let Some(s) = it.next() {
    res.push('\n');
    push_field(s.as_ref(), sep, &mut res);
    for s in it {
      res.push(',');
      push_field(s.as_ref(), sep, &mut res);
    }
  }
  res.shrink_to_fit();
  res.as_bytes().into()
}

fn to_same<R: BufRead, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .write_to_data_beginning(&mut writer, &(), false)?
  {
    it.copy_remaining_data(&mut writer.inner())
      .and_then(|_| it.read_to_end())
      .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn to_tabledata<R: BufRead, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_tabledata()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    InMemTableDataRows::write_tabledata_rows(
      &mut writer,
      it.to_row_value_iter().map(|r| match r {
        Ok(row) => row,
        Err(e) => panic!("Error reading rows: {:?}", e),
      }),
      schema,
    )
    .and_then(|_| it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn to_binary<R: BufRead, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_binary()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    InMemTableDataRows::write_binary_rows(
      writer.inner(),
      it.to_row_value_iter().map(|r| match r {
        Ok(row) => row,
        Err(e) => panic!("Error reading rows: {:?}", e),
      }),
      schema,
    )
    .and_then(|_| it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn to_binary2<R: BufRead, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_binary2()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    InMemTableDataRows::write_binary2_rows(
      writer.inner(),
      it.to_row_value_iter().map(|r| match r {
        Ok(row) => row,
        Err(e) => panic!("Error reading rows: {:?}", e),
      }),
      schema,
    )
    .and_then(|_| it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn to_csv<R: BufRead, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  mut write: W,
  separator: char,
) -> Result<(), VOTableError> {
  write_csv_header(&it.votable, &mut write, separator)?;
  // Write data
  for row in it.to_row_value_iter() {
    let mut field_it = row?.into_iter();
    if let Some(field) = field_it.next() {
      write_1st_csv_field_with_newline(&mut write, field.to_string().as_str(), separator)?;
      for field in field_it {
        write_csv_field(&mut write, field.to_string().as_str(), separator)?;
      }
    }
  }
  // End with a newline
  write.write_all(b"\n").map_err(VOTableError::Io)
}

fn td_to_binary_par<R: BufRead + Send, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_binary()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);

    let mut raw_row_it = it.to_owned_tabledata_row_iterator();
    fn convert(raw_td_row: &[u8], schema: &TableSchema, _sep: char) -> Box<[u8]> {
      let n_fields = schema.as_slice().len();
      let mut bin_row: Vec<u8> = Vec::with_capacity(512);
      let mut bin_ser = BinarySerializer::new(&mut bin_row);
      let mut n_fields_found = 0;
      for (field_res, schema) in FieldIteratorUnbuffered::new(raw_td_row).zip(schema.iter()) {
        match field_res
          .and_then(|field| schema.value_from_str(field.as_str()))
          .and_then(|field| schema.serialize_seed(&field, &mut bin_ser))
        {
          Ok(()) => {
            n_fields_found += 1;
          }
          Err(e) => panic!("Error reading row: {:?}", e),
        }
      }
      if n_fields != n_fields_found {
        panic!(
          "Wrong number of fields in row. Expected: {}. Actual: {}.",
          n_fields, n_fields_found
        );
      }
      bin_row.into_boxed_slice()
    }
    let write = EncoderWriter::new(
      B64Formatter::new(writer.inner()),
      &general_purpose::STANDARD,
    );
    convert_par(
      &mut raw_row_it,
      schema,
      convert,
      write,
      ' ',
      n_threads,
      chunk_size,
    )
    .and_then(|_| raw_row_it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn td_to_binary2_par<R: BufRead + Send, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_binary2()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);

    let mut raw_row_it = it.to_owned_tabledata_row_iterator();
    fn convert(raw_td_row: &[u8], schema: &TableSchema, _sep: char) -> Box<[u8]> {
      let mut bin_row: Vec<u8> = Vec::with_capacity(512);
      let mut bin_ser = BinarySerializer::new(&mut bin_row);
      if let Err(e) = FieldIteratorUnbuffered::new(raw_td_row)
        .zip(schema.iter())
        .map(|(res, schema)| res.and_then(|field_str| schema.value_from_str(&field_str)))
        .collect::<Result<Vec<VOTableValue>, VOTableError>>()
        .map(|fields| InMemTableDataRows::write_binary2_row(&mut bin_ser, fields, schema))
      {
        panic!("Error convertings rows: {:?}", e);
      }
      bin_row.into_boxed_slice()
    }
    let write = EncoderWriter::new(
      B64Formatter::new(writer.inner()),
      &general_purpose::STANDARD,
    );
    convert_par(
      &mut raw_row_it,
      schema,
      convert,
      write,
      ' ',
      n_threads,
      chunk_size,
    )
    .and_then(|_| raw_row_it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn binary_to_td_par<R: BufRead + Send, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_tabledata()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    let mut raw_row_it = it.to_owned_binary_row_iterator();
    fn convert(raw_bin_row: &[u8], schema: &TableSchema, _sep: char) -> Box<[u8]> {
      let mut td_row: Vec<u8> = Vec::with_capacity(512);
      td_row.append(b"\n<TR>".to_vec().as_mut());
      InMemTableDataRows::write_tabledata_row(
        &mut new_xml_writer(&mut td_row, None, None),
        binrow2fieldit(raw_bin_row, schema),
        schema,
      )
      .unwrap();
      td_row.append(b"</TR>".to_vec().as_mut());
      td_row.into_boxed_slice()
    }
    convert_par(
      &mut raw_row_it,
      schema,
      convert,
      writer.inner(),
      ' ',
      n_threads,
      chunk_size,
    )
    .and_then(|_| raw_row_it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn binary_to_binary2_par<R: BufRead + Send, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_binary2()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    let mut raw_row_it = it.to_owned_binary_row_iterator();
    fn convert(raw_bin_row: &[u8], schema: &TableSchema, _sep: char) -> Box<[u8]> {
      let mut bin_row: Vec<u8> = Vec::with_capacity(512);
      let mut bin_ser = BinarySerializer::new(&mut bin_row);
      if let Err(e) = InMemTableDataRows::write_binary2_row(
        &mut bin_ser,
        binrow2fieldit(raw_bin_row, schema).collect::<Vec<VOTableValue>>(),
        schema,
      ) {
        panic!("Error convertings rows: {:?}", e);
      }
      bin_row.into_boxed_slice()
    }
    convert_par(
      &mut raw_row_it,
      schema,
      convert,
      B64Formatter::new(writer.inner()),
      ' ',
      n_threads,
      chunk_size,
    )
    .and_then(|_| raw_row_it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

//
fn binary2_to_td_par<R: BufRead + Send, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_tabledata()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    let mut raw_row_it = it.to_owned_binary2_row_iterator();
    fn convert(raw_bin_row: &[u8], schema: &TableSchema, _sep: char) -> Box<[u8]> {
      let mut td_row: Vec<u8> = Vec::with_capacity(512);
      td_row.append(b"\n<TR>".to_vec().as_mut());
      InMemTableDataRows::write_tabledata_row(
        &mut new_xml_writer(&mut td_row, None, None),
        bin2row2fieldit(raw_bin_row, schema),
        schema,
      )
      .unwrap();
      td_row.append(b"</TR>".to_vec().as_mut());
      td_row.into_boxed_slice()
    }
    let write = EncoderWriter::new(
      B64Formatter::new(writer.inner()),
      &general_purpose::STANDARD,
    );
    convert_par(
      &mut raw_row_it,
      schema,
      convert,
      write,
      ' ',
      n_threads,
      chunk_size,
    )
    .and_then(|_| raw_row_it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn binary2_to_binary_par<R: BufRead + Send, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  if it
    .votable
    .to_binary()
    .and_then(|_| it.votable.write_to_data_beginning(&mut writer, &(), false))?
  {
    let schema = get_schema(&it.votable);
    let mut raw_row_it = it.to_owned_binary2_row_iterator();
    fn convert(raw_bin_row: &[u8], schema: &TableSchema, _sep: char) -> Box<[u8]> {
      let mut bin_row: Vec<u8> = Vec::with_capacity(512);
      let mut bin_ser = BinarySerializer::new(&mut bin_row);
      if let Err(e) = InMemTableDataRows::write_binary_row(
        &mut bin_ser,
        bin2row2fieldit(raw_bin_row, schema),
        schema,
      ) {
        panic!("Error convertings rows: {:?}", e);
      }
      bin_row.into_boxed_slice()
    }
    let write = EncoderWriter::new(
      B64Formatter::new(writer.inner()),
      &general_purpose::STANDARD,
    );
    convert_par(
      &mut raw_row_it,
      schema,
      convert,
      write,
      ' ',
      n_threads,
      chunk_size,
    )
    .and_then(|_| raw_row_it.read_to_end())
    .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
  } else {
    // No table in the VOTable
    Ok(())
  }
}

fn write_csv_header<W: Write>(
  votable: &VOTable<VoidTableDataContent>,
  mut write: W,
  separator: char,
) -> Result<(), VOTableError> {
  // Write header
  let mut colname_it = get_colnames(votable).into_iter();
  if let Some(colname) = colname_it.next() {
    write_1st_csv_field(&mut write, colname.as_str(), separator)?;
    for colname in colname_it {
      write_csv_field(&mut write, colname.as_str(), separator)?;
    }
  }
  Ok(())
}
/// # Params
/// * `convert`: convert a raw row in bytes in one format to a raw row in bytes in another format.
fn convert_par<I, W>(
  raw_row_it: &mut I,
  schema: TableSchema,
  convert: fn(&[u8], &TableSchema, char) -> Box<[u8]>,
  mut write: W,
  separator: char,
  n_threads: usize,
  chunk_size: usize,
) -> Result<(), VOTableError>
where
  I: Iterator<Item = Result<Vec<u8>, VOTableError>> + Send,
  W: Write,
{
  let schema = schema;
  // Usage of crossbeam from https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html
  let (snd1, rcv1) = bounded(1);
  let (snd2, rcv2) = bounded(1);
  scope(|s| {
    // Producer thread
    s.spawn(|| {
      let mut rows_chunk = load_n(raw_row_it, chunk_size);
      while !rows_chunk.is_empty() {
        snd1
          .send(rows_chunk)
          .expect("Unexpected error sending raw rows");
        rows_chunk = load_n(raw_row_it, chunk_size);
      }
      // Close the channel, otherwise sink will never exit the for-loop
      drop(snd1);
    });
    // Parallel processing by n_threads
    for _ in 0..n_threads {
      // Send to sink, receive from source
      let (sendr, recvr) = (snd2.clone(), rcv1.clone());
      let schema = schema.clone();
      // Spawn workers in separate threads
      s.spawn(move || {
        // Receive until channel closes
        for raw_rows_chunk in recvr.iter() {
          let converted_raw_rows_chunk = raw_rows_chunk
            .iter()
            .map(|raw_row| convert(raw_row, &schema, separator))
            .collect::<Vec<Box<[u8]>>>();
          sendr
            .send(converted_raw_rows_chunk)
            .expect("Unexpected error sending converted rows");
        }
      });
    }
    // Close the channel, otherwise sink will never exit the for-loop
    drop(snd2);
    // Sink
    for raw_rows in rcv2.iter() {
      for raw_row in raw_rows {
        match write.write_all(&raw_row) {
          Ok(()) => (),
          Err(e) => panic!("Error writing in parallel: {:?}", e),
        }
      }
    }
  });
  Ok(())
}

fn load_n<R, T>(iter: &mut T, n: usize) -> Vec<R>
where
  R: Send + Sync,
  T: Send + Iterator<Item = Result<R, VOTableError>>,
{
  match iter.take(n).collect() {
    Ok(v) => v,
    Err(e) => panic!("Error reading rows: {:?}", e),
  }
}
