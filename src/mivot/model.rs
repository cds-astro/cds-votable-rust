use crate::{error::VOTableError, QuickXmlReadWrite};
use paste::paste;
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Model {
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  url: Option<String>,
}
impl Model {
  fn new<N: Into<String>>(name: N) -> Self {
    Self {
      name: name.into(),
      url: None,
    }
  }
  impl_builder_opt_string_attr!(url);
}

impl QuickXmlReadWrite for Model {
  const TAG: &'static str = "MODEL";
  type Context = ();

  fn from_attributes(
    attrs: quick_xml::events::attributes::Attributes,
  ) -> Result<Self, crate::error::VOTableError> {
    const NULL: &str = "@TBD";
    let mut model = Self::new(NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      model = match attr.key {
        b"name" => {
          model.name = value.to_string();
          model
        }
        b"url" => model.set_url(value),
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    if model.name.as_str() == NULL {
      Err(VOTableError::Custom(format!(
        "Attributes 'name' is mandatory in tag '{}'",
        Self::TAG
      )))
    } else {
      Ok(model)
    }
  }

  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    _reader: quick_xml::Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
    unreachable!()
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut quick_xml::Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    unreachable!()
  }

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut quick_xml::Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("name", self.name.as_str()));
    write_opt_string_attr!(self, elem_writer, url);
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}
