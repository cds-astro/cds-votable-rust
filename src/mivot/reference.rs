use super::foreignkey::ForeignKey;

use crate::{error::VOTableError, QuickXmlReadWrite};

use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Reference {
  dmrole: String,
  dmref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  foreign_keys: Vec<ForeignKey>,
}
impl Reference {
  fn new<N: Into<String>>(dmrole: N, dmref: N) -> Self {
    Self {
      dmrole: dmrole.into(),
      dmref: dmref.into(),
      foreign_keys: vec![],
    }
  }
}

impl QuickXmlReadWrite for Reference {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut foreign_key = Self::new(NULL, NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      foreign_key = match attr.key {
        b"dmrole" => {
          foreign_key.dmrole = value.to_string();
          foreign_key
        }
        b"dmref" => {
          foreign_key.dmref = value.to_string();
          foreign_key
        }
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    if foreign_key.dmrole.as_str() == NULL || foreign_key.dmref.as_str() == NULL {
      Err(VOTableError::Custom(format!(
        "Attributes 'dmrole' and 'dmref' are mandatory in tag '{}'",
        Self::TAG
      )))
    } else {
      Ok(foreign_key)
    }
  }

  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, crate::error::VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Empty(ref e) => match e.local_name() {
          ForeignKey::TAG_BYTES => self.foreign_keys.push(ForeignKey::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    todo!()
  }

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    if self.foreign_keys.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer = elem_writer.with_attribute(("dmrole", self.dmrole.as_str()));
      elem_writer = elem_writer.with_attribute(("dmref", self.dmref.as_str()));
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      tag.push_attribute(("dmrole", self.dmrole.as_str()));
      tag.push_attribute(("dmref", self.dmref.as_str()));
      writer
        .write_event(Event::Start(tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem_vec!(self, foreign_keys, writer, context);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    }
  }
}
