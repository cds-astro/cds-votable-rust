
use std::{
  fmt::{self, Debug},
  collections::HashMap,
  io::{BufRead, Write},
  str::{self, FromStr},
};

use quick_xml::{Reader, Writer, events::{Event, BytesText, attributes::Attributes}};

use paste::paste;

use serde_json::Value;

use super::{error::VOTableError, QuickXmlReadWrite};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ContentRole {
  Query,
  Hint,
  Doc,
  Location,
}

impl FromStr for ContentRole {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "query" => Ok(ContentRole::Query),
      "hint" => Ok(ContentRole::Hint),
      "doc" => Ok(ContentRole::Doc),
      "location" => Ok(ContentRole::Location),
      _ => Err(format!("Unknown content-role variant. Actual: '{}'. Expected: 'query', 'hint', 'doc' or 'location'.", s))
    }
  }
}

impl From<&ContentRole> for &'static str {
  fn from(content_role: &ContentRole) -> Self {
    match content_role {
      ContentRole::Query => "query",
      ContentRole::Hint => "hint",
      ContentRole::Doc => "doc",
      ContentRole::Location => "location",
    }
  }
}

impl fmt::Display for ContentRole {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    Debug::fmt(self, f)
  }
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Link {
  #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "content-role", skip_serializing_if = "Option::is_none")]
  pub content_role: Option<ContentRole>,
  #[serde(rename = "content-type", skip_serializing_if = "Option::is_none")]
  pub content_type: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub value: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub href: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // content
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
}

impl Link {
  pub fn new() -> Self {
    Self::default()
  }

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_attr!(content_role, ContentRole);
  impl_builder_opt_string_attr!(content_type);
  impl_builder_opt_string_attr!(title);
  impl_builder_opt_string_attr!(value);
  impl_builder_opt_string_attr!(href);

  impl_builder_insert_extra!();

  impl_builder_opt_string_attr!(content);
}

impl QuickXmlReadWrite for Link {
  const TAG: &'static str = "LINK";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut link = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      link = match attr.key {
        b"ID" => link.set_id(value),
        b"content-role" => link.set_content_role(ContentRole::from_str(value).map_err(VOTableError::Custom)?),
        b"content-type" => link.set_content_type(value),
        b"title" => link.set_title(value),
        b"value" => link.set_value(value),
        b"href" => link.set_href(value),
        _ => link.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    Ok(link)
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
    write_opt_string_attr!(self, elem_writer, ID);
    write_opt_into_attr!(self, elem_writer, content_role, "content-role");
    write_opt_string_attr!(self, elem_writer, content_type, "content-type");
    write_opt_string_attr!(self, elem_writer, title);
    write_opt_string_attr!(self, elem_writer, value);
    write_opt_string_attr!(self, elem_writer, href);
    write_extra!(self, elem_writer);
    write_content!(self, elem_writer);
    Ok(())
  }
}