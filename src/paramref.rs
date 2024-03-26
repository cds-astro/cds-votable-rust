//! Struct dedicated to the `PARAMref` tag.

use std::{collections::HashMap, str};

use paste::paste;
use serde_json::Value;

use super::{
  error::VOTableError, HasContent, HasContentElem, TableDataContent, VOTableElement, VOTableVisitor,
};

/// Struct corresponding to the `PARAMRef` XML tag.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ParamRef {
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

impl ParamRef {
  pub fn new<S: Into<String>>(ref_: S) -> Self {
    Self {
      ref_: ref_.into(),
      ucd: None,
      utype: None,
      extra: Default::default(),
      content: None,
    }
  }

  // attributes
  impl_builder_mandatory_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  // extra attributes
  impl_builder_insert_extra!();

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_paramref(self)
  }
}

impl_has_content!(ParamRef);

impl VOTableElement for ParamRef {
  const TAG: &'static str = "PARAMref";

  type MarkerType = HasContentElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::new(DEFAULT_VALUE).set_attrs(attrs).and_then(|pref| {
      if pref.ref_.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'ref' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(pref)
      }
    })
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
        "ref" => self.set_ref_by_ref(val),
        "ucd" => self.set_ucd_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
        _ => self.insert_extra_str_by_ref(key, val),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
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
    for_each_extra_attribute!(self, f);
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    paramref::ParamRef,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_paramref_read_write() {
    let xml = r#"<PARAMref ref="col3" ucd="UCD" utype="ut"></PARAMref>"#; // Test read
    let param = test_read::<ParamRef>(xml);
    assert_eq!(param.ref_.as_str(), "col3");
    assert_eq!(param.utype, Some("ut".to_string()));
    assert_eq!(param.ucd, Some("UCD".to_string()));
    // Test write
    test_writer(param, xml)
  }
}
