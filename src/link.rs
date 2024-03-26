//! Struct dedicated to the `LINK` tag.

use std::{
  collections::HashMap,
  fmt::{self, Debug},
  str::{self, FromStr},
};

use paste::paste;
use serde_json::Value;

use super::{error::VOTableError, HasContent, HasContentElem, VOTableElement};

/// Enum for the possible values of the `content-role` attriute.
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

/// Struct corresponding to the `LINK` XML tag.
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

  // attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_opt_attr!(content_role, ContentRole);
  impl_builder_opt_string_attr!(content_type);
  impl_builder_opt_string_attr!(title);
  impl_builder_opt_string_attr!(value);
  impl_builder_opt_string_attr!(href);
  // extra attributes
  impl_builder_insert_extra!();
}
impl_has_content!(Link);

impl VOTableElement for Link {
  const TAG: &'static str = "LINK";

  type MarkerType = HasContentElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new().set_attrs(attrs)
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "ID" => self.set_id_by_ref(val),
        "content-role" => {
          self.set_content_role_by_ref(val.as_ref().parse().map_err(VOTableError::Custom)?)
        }
        "content-type" => self.set_content_type_by_ref(val),
        "title" => self.set_title_by_ref(val),
        "value" => self.set_value_by_ref(val),
        "href" => self.set_href_by_ref(val),
        _ => self.insert_extra_str_by_ref(key, val),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(id) = &self.id {
      f("ID", id.as_str());
    }
    if let Some(content_role) = &self.content_role {
      f("content-role", content_role.into());
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
    for_each_extra_attribute!(self, f);
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
