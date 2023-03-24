
use std::{
  str, 
  collections::HashMap,
  io::{BufRead, Write},
};

use quick_xml::{Reader, Writer, events::{Event, BytesText, attributes::Attributes}};

use paste::paste;

use serde_json::Value;

use super::{
  QuickXmlReadWrite,
  error::VOTableError,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ParamRef {
  #[serde(rename = "ref")]
  ref_: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  utype: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // content
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
}

impl ParamRef {

  pub fn new<S: Into<String>>(ref_: S) -> Self {
    Self {
      ref_: ref_.into(),
      ucd: None,
      utype: None,
      extra: Default::default(),
      content: None
    }
  }

  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  
  impl_builder_insert_extra!();

  impl_builder_opt_string_attr!(content);
}

impl QuickXmlReadWrite for ParamRef {
  const TAG: &'static str = "PARAMref";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut paramref = Self::new(NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      paramref = match attr.key {
        b"ref" => { paramref.ref_ = value.to_string(); paramref },
        b"ucd" => paramref.set_ucd(value),
        b"utype" => paramref.set_utype(value),
        _ => paramref.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    if paramref.ref_.as_str() == NULL {
      Err(VOTableError::Custom(format!("Attributes 'ref' is mandatory in tag '{}'", Self::TAG)))
    } else {
      Ok(paramref)
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    read_content!(Self, self, reader, reader_buff)
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
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("ref", self.ref_.as_str()));
    write_opt_string_attr!(self, elem_writer, ucd);
    write_opt_string_attr!(self, elem_writer, utype);
    write_extra!(self, elem_writer);
    write_content!(self, elem_writer);
    Ok(())
  }
}
