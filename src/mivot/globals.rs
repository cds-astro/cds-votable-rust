use quick_xml::{
  events::{BytesStart, Event},
  Writer,
};

use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};

use super::{
  collection::Collection,
  instance::{GlobOrTempInstance, InstanceContexts},
};
use std::{io::Write, str};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum GlobalsElem {
  Instance(GlobOrTempInstance),
  Collection(Collection),
}
impl GlobalsElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      GlobalsElem::Instance(elem) => elem.write(writer, &InstanceContexts::Writing),
      GlobalsElem::Collection(elem) => elem.write(writer, &()),
    }
  }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Globals {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<GlobalsElem>,
}
impl QuickXmlReadWrite for Globals {
  const TAG: &'static str = "GLOBALS";
  type Context = ();

  fn from_attributes(
    attrs: quick_xml::events::attributes::Attributes,
  ) -> Result<Self, crate::error::VOTableError> {
    if attrs.count() > 0 {
      eprintln!("Unexpected attributes in GLOBALS (not serialized!)");
    }
    Ok(Self::default())
  }

  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    mut reader: quick_xml::Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          GlobOrTempInstance::TAG_BYTES => {
            self.elems.push(GlobalsElem::Instance(from_event_start!(
              GlobOrTempInstance,
              reader,
              reader_buff,
              e,
              InstanceContexts::B
            )))
          }
          Collection::TAG_BYTES => self.elems.push(GlobalsElem::Collection(from_event_start!(
            Collection,
            reader,
            reader_buff,
            e
          ))),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ));
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut quick_xml::Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    todo!()
  }

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut quick_xml::Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    write_elem_vec_no_context!(self, elems, writer);
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}
