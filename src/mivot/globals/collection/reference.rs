//! Contains the static `REFERENCE` structure which is **child of** `COLLECTION` in `GLOBALS`.
//!
//! A `REFERENCE` is made to be replaced by an `INSTANCE` or a `COLLECTION` that can be retrieved
//! either dynamically (in `TEMPLATES`) or statically (in `GLOBALS` or in `TEMPLATES`).

use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

/// Static `REFERENCE` **child of** `COLLECTION` in `GLOBALS`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Reference {
  /// `dmid` of the referenced `INSTANCE` or `COLLECTION`.
  pub dmref: String,
}

impl Reference {
  pub fn new<S: Into<String>>(dmref: S) -> Self {
    Self {
      dmref: dmref.into(),
    }
  }

  impl_builder_mandatory_string_attr!(dmref);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_reference_static_childof_collection(self)
  }
}

impl VOTableElement for Reference {
  const TAG: &'static str = "REFERENCE";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::new(DEFAULT_VALUE).set_attrs(attrs).and_then(|r| {
      if r.dmref.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'dmref' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(r)
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
        "dmref" => self.set_dmref_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("dmref", self.dmref.as_str());
  }
}
