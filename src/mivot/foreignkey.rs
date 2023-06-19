use crate::{error::VOTableError, QuickXmlReadWrite};

use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::{io::Write, str};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ForeignKey {
  #[serde(rename = "ref")]
  ref_: String,
}
impl ForeignKey {
  fn new<N: Into<String>>(ref_: N) -> Self {
    Self { ref_: ref_.into() }
  }
}

impl QuickXmlReadWrite for ForeignKey {
  const TAG: &'static str = "FOREIGN_KEY";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut foreign_key = Self::new(NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      foreign_key = match attr.key {
        b"ref" => {
          foreign_key.ref_ = value.to_string();
          foreign_key
        }
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    if foreign_key.ref_.as_str() == NULL {
      Err(VOTableError::Custom(format!(
        "Attribute 'ref' is mandatory in tag '{}'",
        Self::TAG
      )))
    } else {
      Ok(foreign_key)
    }
  }

  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    mut _reader: Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, crate::error::VOTableError> {
    unreachable!()
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    unreachable!()
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("ref", self.ref_.as_str()));
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}
