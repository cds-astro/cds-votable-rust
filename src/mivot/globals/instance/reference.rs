//! Contains the static `REFERENCE` structures which is **child of** `INSTANCE` in `GLOBALS`.
//!
//! A `REFERENCE` is made to be replaced by an `INSTANCE` or a `COLLECTION` that can be retrieved
//! either dynamically (in `TEMPLATES`) or statically (in `GLOBALS` or in `TEMPLATES`).

use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

/// Static `REFERENCE` **child of** `INSTANCE` in `GLOBALS`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Reference {
  /// name of the referenced `INSTANCE` or `COLLECTION` in the data model.
  pub dmrole: String,
  /// `dmid` of the referenced `INSTANCE` or `COLLECTION`.
  pub dmref: String,
}

impl Reference {
  pub fn new<S: Into<String>>(dmrole: S, dmref: S) -> Self {
    Self {
      dmrole: dmrole.into(),
      dmref: dmref.into(),
    }
  }

  impl_builder_mandatory_string_attr!(dmrole);
  impl_builder_mandatory_string_attr!(dmref);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_reference_static_childof_instance(self)
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
    Self::new(DEFAULT_VALUE, DEFAULT_VALUE)
      .set_attrs(attrs)
      .and_then(|r| {
        if r.dmrole.as_str() == DEFAULT_VALUE || r.dmref.as_str() == DEFAULT_VALUE {
          Err(VOTableError::Custom(format!(
            "Mandatory attributes 'dmrole' or 'dmref' not found in tag '{}'",
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
        "dmrole" => self.set_dmrole_by_ref(val),
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
    f("dmrole", self.dmref.as_str());
    f("dmref", self.dmref.as_str());
  }
}

#[cfg(test)]
mod tests {
  use super::Reference;

  use crate::{mivot::test::get_xml, tests::test_read};

  #[test]
  fn test_staticref_read() {
    let xml = get_xml("./resources/mivot/6/test_6_ok_6.1.xml");
    println!("testing 6.1");
    test_read::<Reference>(&xml);
  }
}
