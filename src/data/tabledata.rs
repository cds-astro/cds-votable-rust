use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{
  events::{attributes::Attributes, BytesEnd, BytesStart, Event},
  Reader, Writer,
};

use serde;

use crate::{error::VOTableError, table::TableElem, QuickXmlReadWrite, TableDataContent};

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TableData<C: TableDataContent> {
  #[serde(flatten)]
  pub content: C,
}

impl<C: TableDataContent> TableData<C> {
  pub fn new(content: C) -> Self {
    Self { content }
  }

  pub(crate) fn ensures_consistency(&mut self, context: &[TableElem]) -> Result<(), String> {
    self.content.ensures_consistency(context)
  }

  pub(crate) fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    writer
      .write_event(Event::Start(BytesStart::borrowed_name(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)
  }

  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    writer
      .write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)
  }
}

impl<C: TableDataContent> QuickXmlReadWrite for TableData<C> {
  const TAG: &'static str = "TABLEDATA";
  type Context = Vec<TableElem>;

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let table_data = Self::default();
    if attrs.count() > 0 {
      eprintln!(
        "No attribute expected in {}: attribute(s) ignored.",
        Self::TAG
      );
    }
    Ok(table_data)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, context)
      .map(|()| reader)
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self
      .content
      .read_datatable_content(reader, reader_buff, context)
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self
      .write_to_data_beginning(writer)
      .and_then(|()| self.content.write_in_datatable(writer, context))
      .and_then(|()| self.write_from_data_end(writer))
  }
}
