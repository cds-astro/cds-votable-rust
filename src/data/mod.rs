
use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use paste::paste;

use serde;

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

use crate::{
  is_empty,
  impls::mem::VoidTableDataContent
};

use self::{
  tabledata::TableData,
  binary::Binary,
  binary2::Binary2,
  fits::Fits,
};

pub enum TableOrBinOrBin2 {
  TableData,
  Binary,
  Binary2,
  Fits(Fits),
}

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
      DataElem::Fits(ref mut e) => e.read_sub_elements_and_clean(reader, reader_buff, &()),
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
      DataElem::Fits(elem) => elem.write(writer, &()),
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

  pub fn set_binary_by_ref(&mut self, binary: Binary<C>) {
    self.data = DataElem::Binary(binary);
  }

  pub fn set_binary2_by_ref(&mut self, binary2: Binary2<C>) {
    self.data = DataElem::Binary2(binary2);
  }

  pub fn set_fits_by_ref(&mut self, fits: Fits) {
    self.data = DataElem::Fits(fits);
  }

  impl_builder_push!(Info);

}

impl Data<VoidTableDataContent> {

  pub(crate) fn read_till_table_bin_or_bin2_or_fits_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<TableOrBinOrBin2>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            TableData::<VoidTableDataContent>::TAG_BYTES => return Ok(Some(TableOrBinOrBin2::TableData)),
            Binary::<VoidTableDataContent>::TAG_BYTES => return Ok(Some(TableOrBinOrBin2::Binary)),
            Binary2::<VoidTableDataContent>::TAG_BYTES => return Ok(Some(TableOrBinOrBin2::Binary2)),
            Fits::TAG_BYTES => return Ok(Some(TableOrBinOrBin2::Fits(from_event_start_by_ref!(Fits, reader, reader_buff, e)))),
            Info::TAG_BYTES => self.infos.push(from_event_start_by_ref!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES => self.infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(None),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }
  
  /*pub(crate) fn read_till_table_bin_or_bin2_or_fits<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<(Reader<R>, Option<TableOrBinOrBin2>), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            TableData::<VoidTableDataContent>::TAG_BYTES => return Ok((reader, Some(TableOrBinOrBin2::TableData))),
            Binary::<VoidTableDataContent>::TAG_BYTES => return Ok((reader, Some(TableOrBinOrBin2::Binary))),
            Binary2::<VoidTableDataContent>::TAG_BYTES => return Ok((reader, Some(TableOrBinOrBin2::Binary2))),
            Fits::TAG_BYTES => return Ok((reader, Some(TableOrBinOrBin2::Fits))),
            Info::TAG_BYTES => self.infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES => self.infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok((reader, None)),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }*/

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
          match e.local_name() {
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
              self.data = DataElem::Fits(from_event_start!(Fits, reader, reader_buff, e))
            }
            Info::TAG_BYTES => self.infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES => self.infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    todo!()
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
