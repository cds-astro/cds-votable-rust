//! Struct dedicated to the `FIELDref` tag.

use std::{
  collections::HashMap,
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesText, Event},
  Reader, Writer,
};
use serde_json::Value;

use super::{
  error::VOTableError,
  utils::{discard_comment, discard_event},
  QuickXmlReadWrite, TableDataContent, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldRef {
  #[serde(rename = "ref")]
  pub ref_: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // content
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
}

impl FieldRef {
  pub fn new<S: Into<String>>(ref_: S) -> Self {
    Self {
      ref_: ref_.into(),
      ucd: None,
      utype: None,
      extra: Default::default(),
      content: None,
    }
  }

  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);

  impl_builder_insert_extra!();

  impl_builder_opt_string_attr!(content);

  /// Calls a closure on each (key, value) attribute pairs.
  pub fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("ref", self.ref_.as_str());
    if let Some(ucd) = &self.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
    for (k, v) in &self.extra {
      f(k.as_str(), v.to_string().as_str());
    }
  }

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_fieldref(self)
  }
}

impl QuickXmlReadWrite for FieldRef {
  const TAG: &'static str = "FIELDref";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut paramref = Self::new(NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      paramref = match attr.key {
        b"ref" => {
          paramref.ref_ = value.to_string();
          paramref
        }
        b"ucd" => paramref.set_ucd(value),
        b"utype" => paramref.set_utype(value),
        _ => paramref.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    if paramref.ref_.as_str() == NULL {
      Err(VOTableError::Custom(format!(
        "Attributes 'ref' is mandatory in tag '{}'",
        Self::TAG
      )))
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
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    read_content_by_ref!(Self, self, reader, reader_buff)
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
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

#[cfg(test)]
mod tests {
  use crate::{
    fieldref::FieldRef,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_fieldref_read_write() {
    let xml = r#"<FIELDref ref="col4" ucd="UCD" utype="ut"></FIELDref>"#; // Test read
    let field = test_read::<FieldRef>(xml);
    assert_eq!(field.ref_.as_str(), "col4");
    assert_eq!(field.utype, Some("ut".to_string()));
    assert_eq!(field.ucd, Some("UCD".to_string()));
    // Test write
    test_writer(field, xml)
  }
}
