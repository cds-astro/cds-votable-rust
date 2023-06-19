use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use paste::paste;
use quick_xml::events::{BytesStart, Event};
use std::str;

use super::{
  instance::{GlobOrTempInstance, InstanceContexts},
  r#where::Where,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Templates {
  #[serde(skip_serializing_if = "Option::is_none")]
  tableref: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  wheres: Vec<Where>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  instances: Vec<GlobOrTempInstance>,
}
impl Templates {
  fn new() -> Self {
    Self {
      tableref: None,
      wheres: vec![],
      instances: vec![],
    }
  }
  impl_builder_opt_string_attr!(tableref);
}
impl QuickXmlReadWrite for Templates {
  const TAG: &'static str = "TEMPLATES";
  type Context = ();

  fn from_attributes(
    attrs: quick_xml::events::attributes::Attributes,
  ) -> Result<Self, crate::error::VOTableError> {
    let mut templates = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      templates = match attr.key {
        b"tableref" => templates.set_tableref(value),
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    Ok(templates)
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
          GlobOrTempInstance::TAG_BYTES => self.instances.push(from_event_start!(
            GlobOrTempInstance,
            reader,
            reader_buff,
            e,
            InstanceContexts::A
          )),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Where::TAG_BYTES => self.wheres.push(Where::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
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
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    //OPTIONAL
    push2write_opt_string_attr!(self, tag, tableref);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    write_elem_vec_empty_context!(self, wheres, writer);
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}
