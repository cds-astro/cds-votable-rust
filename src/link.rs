//! Struct dedicated to the `LINK` tag.

use std::{
  collections::HashMap,
  fmt::{self, Debug},
  io::{BufRead, Write},
  str::{self, FromStr},
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
  QuickXmlReadWrite,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContentRole {
  Query,
  Hints,
  Doc,
  Location,
}

impl FromStr for ContentRole {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "query" => Ok(ContentRole::Query),
      "hints" => Ok(ContentRole::Hints),
      "doc" => Ok(ContentRole::Doc),
      "location" => Ok(ContentRole::Location),
      _ => Err(format!("Unknown content-role variant. Actual: '{}'. Expected: 'query', 'hints', 'doc' or 'location'.", s))
    }
  }
}

impl From<&ContentRole> for &'static str {
  fn from(content_role: &ContentRole) -> Self {
    match content_role {
      ContentRole::Query => "query",
      ContentRole::Hints => "hints",
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

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

  /// Calls a closure on each (key, value) attribute pairs.
  pub fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(id) = &self.id {
      f("ID", id.as_str());
    }
    if let Some(content_role) = &self.content_role {
      f("content-role", content_role.to_string().as_str());
    }
    if let Some(content_type) = &self.content_type {
      f("content-type", content_type.as_str());
    }
    if let Some(title) = &self.title {
      f("title", title.as_str());
    }
    if let Some(value) = &self.value {
      f("value", value.as_str());
    }
    if let Some(href) = &self.href {
      f("href", href.as_str());
    }
    for (k, v) in &self.extra {
      f(k.as_str(), v.to_string().as_str());
    }
  }
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
        b"content-role" => {
          link.set_content_role(ContentRole::from_str(value).map_err(VOTableError::Custom)?)
        }
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

#[cfg(test)]
mod tests {
  use crate::{
    link::Link,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_link_read_write() {
    let xml =
      r#"<LINK ID="id" content-role="doc" content-type="text/text" href="http://127.0.0.1/"/>"#; // Test read
    let link = test_read::<Link>(xml);
    assert_eq!(link.id, Some("id".to_string()));
    assert_eq!(link.href, Some("http://127.0.0.1/".to_string()));
    let role = format!("{}", link.content_role.as_ref().unwrap());
    assert_eq!(role, "Doc".to_string());
    assert_eq!(link.content_type, Some("text/text".to_string()));
    // Test write
    test_writer(link, xml);
  }
}
