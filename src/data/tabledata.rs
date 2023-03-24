
use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart,BytesEnd, attributes::Attributes}};

use serde;

use crate::{
  QuickXmlReadWrite, TableDataContent,
  error::VOTableError,
  table::TableElem
};

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TableData<C: TableDataContent> {
  #[serde(flatten)]
  content: C,
}

impl<C: TableDataContent> TableData<C> {
  pub fn new(content: C) -> Self {
    Self { content }
  }
}



impl<C: TableDataContent> QuickXmlReadWrite for TableData<C> {
  const TAG: &'static str = "TABLEDATA";
  type Context = Vec<TableElem>;

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let table_data = Self::default();
    if attrs.count() > 0 {
      eprintln!("No attribute expected in {}: attribute(s) ignored.", Self::TAG);
    }
    Ok(table_data)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self.content.read_datatable_content(reader, reader_buff, context)
  }


  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    todo!()
  }

  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    context: &Self::Context
  ) -> Result<(), VOTableError> {
    writer.write_event(Event::Start(BytesStart::borrowed_name(Self::TAG_BYTES))).map_err(VOTableError::Write)?;
    self.content.write_in_datatable(writer, context)?;
    writer.write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES))).map_err(VOTableError::Write)
  }
}