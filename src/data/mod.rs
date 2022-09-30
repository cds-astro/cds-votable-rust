
use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use paste::paste;

use serde;
use crate::is_empty;

use super::{
  QuickXmlReadWrite, TableDataContent,
  error::VOTableError,
  info::Info,
  table::TableElem,
};

// Sub modules
pub mod tabledata;
pub mod binary;
pub mod binary2;
pub mod fits;
pub mod stream;


use self::{
  tabledata::TableData,
  binary::Binary,
  binary2::Binary2,
  fits::Fits,
};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "data_type")]
pub enum DataElem<C: TableDataContent> {
  // AstroRes ??
  TableData(TableData<C>),
  Binary(Binary<C>),
  Binary2(Binary2<C>),
  Fits(Fits)
}

impl<C: TableDataContent> DataElem<C> {
  
  fn read_sub_elements<R: BufRead>(
    &mut self,
    reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Vec<TableElem>,
  ) -> Result<Reader<R>, VOTableError> {
    match self {
      DataElem::TableData(ref mut e) => e.read_sub_elements_and_clean(reader, reader_buff, context),
      DataElem::Binary(ref mut e) => e.read_sub_elements_and_clean(reader, reader_buff, context),
      DataElem::Binary2(ref mut e) => e.read_sub_elements_and_clean(reader, reader_buff, context),
      DataElem::Fits(ref mut e) => e.read_sub_elements_and_clean(reader, reader_buff, context),
    }
  }

  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>,
    context: &Vec<TableElem>,
  ) -> Result<(), VOTableError> {
    match self {
      DataElem::TableData(elem) => elem.write(writer, context),
      DataElem::Binary(elem) => elem.write(writer, context),
      DataElem::Binary2(elem) => elem.write(writer, context),
      DataElem::Fits(elem) => elem.write(writer, context),
    }
  }
  
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Data<C: TableDataContent> {
  #[serde(flatten)]
  data: DataElem<C>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  infos: Vec<Info>
}

impl<C: TableDataContent> Data<C> {
  pub fn new_empty() -> Self {
    Self { 
      data: DataElem::TableData(TableData::default()),
      infos: vec![]
    }
  }
  
  pub fn set_tabledata(mut self, content: C) -> Self {
    self.data = DataElem::TableData(TableData::new(content));
    self
  }

  impl_builder_push!(Info);
}

impl<C: TableDataContent> QuickXmlReadWrite for Data<C> {
  const TAG: &'static str = "DATA";
  type Context = Vec<TableElem>;

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let data = Self::new_empty();
    if attrs.count() > 0 {
      eprintln!("No attribute expected in {}: attribute(s) ignored.", Self::TAG);
    }
    Ok(data)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.name() {
            TableData::<C>::TAG_BYTES => reader = self.data.read_sub_elements(reader, reader_buff, context)?,
            Binary::<C>::TAG_BYTES => {
              self.data = DataElem::Binary(Binary::new());
              reader = self.data.read_sub_elements(reader, reader_buff, context)?
            }
            Binary2::<C>::TAG_BYTES => {
              self.data = DataElem::Binary2(Binary2::new());
              reader = self.data.read_sub_elements(reader, reader_buff, context)?
            } 
            Fits::TAG_BYTES => {
              self.data = DataElem::Fits(Fits::new());
              reader = self.data.read_sub_elements(reader, reader_buff, context)?
            }
            Info::TAG_BYTES => self.infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.name() {
            Info::TAG_BYTES => self.infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn write<W: Write>(&mut self, writer: &mut Writer<W>, context: &Self::Context) -> Result<(), VOTableError> {
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-element
    self.data.write(writer, context)?;
    write_elem_vec_empty_context!(self, infos, writer);
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}
