//! Define the `PrimaryKey` structures which are specific to `TEMPLATES`.

use std::str;

use paste::paste;

use crate::{
  error::VOTableError,
  mivot::{globals::instance::primary_key::PrimaryKeyStatic, VodmlVisitor},
  utils::unexpected_attr_err,
  EmptyElem, VOTableElement,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum PrimaryKey {
  Static(PrimaryKeyStatic),
  Dynamic(PrimaryKeyDyn),
}

impl PrimaryKey {
  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    match self {
      PrimaryKey::Static(pk) => pk.visit(visitor),
      PrimaryKey::Dynamic(pk) => pk.visit(visitor),
    }
  }
}

impl VOTableElement for PrimaryKey {
  const TAG: &'static str = "PRIMARY_KEY";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut dmtype = String::new();
    let mut value = String::new(); // static
    let mut ref_ = String::new(); // dynamic
    for (key, val) in attrs {
      let key = key.as_ref();
      let val = val.as_ref();
      if !val.is_empty() {
        match key {
          "dmtype" => dmtype.push_str(val),
          "value" => value.push_str(val),
          "ref" => ref_.push_str(val),
          _ => return Err(unexpected_attr_err(key, Self::TAG)),
        }
      }
    }
    match (dmtype.as_str(), value.as_str(), ref_.as_str()) {
      ("", _, _) => Err(VOTableError::Custom(format!(
        "Attribute 'dmtype' is mandatory and must be non-empty in tag '{}'",
        Self::TAG
      ))),
      (_, "", "") => Err(VOTableError::Custom(format!(
        "Either attribute 'value' or 'ref' is mandatory and must be non-empty in tag '{}'",
        Self::TAG
      ))),
      (dmtype, value, "") => Ok(PrimaryKey::Static(PrimaryKeyStatic::new(dmtype, value))),
      (dmtype, "", ref_) => Ok(PrimaryKey::Dynamic(PrimaryKeyDyn::new(dmtype, ref_))),
      _ => Err(VOTableError::Custom(format!(
        "Either attribute 'value' or 'ref' is mandatory and must be non-empty in tag '{}'",
        Self::TAG
      ))),
    }
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    match self {
      PrimaryKey::Static(pk) => pk.set_attrs_by_ref(attrs),
      PrimaryKey::Dynamic(pk) => pk.set_attrs_by_ref(attrs),
    }
  }

  fn for_each_attribute<F>(&self, f: F)
  where
    F: FnMut(&str, &str),
  {
    match self {
      PrimaryKey::Static(pk) => pk.for_each_attribute(f),
      PrimaryKey::Dynamic(pk) => pk.for_each_attribute(f),
    }
  }
}

/// `Dynamic` primary key are only possible in `TEMPLATE` since
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKeyDyn {
  /// Type of the key.
  pub dmtype: String,
  /// Reference to a `FIELD` in a `TABLE` of the `VOTable`.
  pub ref_: String,
}

impl PrimaryKeyDyn {
  pub fn new<S: Into<String>>(dmtype: S, ref_: S) -> Self {
    Self {
      dmtype: dmtype.into(),
      ref_: ref_.into(),
    }
  }

  impl_builder_mandatory_string_attr!(dmtype);
  impl_builder_mandatory_string_attr!(ref_, ref);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_primarykey_dynamic(self)
  }
}

impl VOTableElement for PrimaryKeyDyn {
  const TAG: &'static str = "PRIMARY_KEY";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("", "").set_attrs(attrs).and_then(|pk| {
      if pk.dmtype.is_empty() || pk.ref_.is_empty() {
        Err(VOTableError::Custom(format!(
          "Mandatory attributes 'dmtype' or 'ref' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(pk)
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
        "dmtype" => self.set_dmtype_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("dmtype", self.dmtype.as_str());
    f("ref", self.ref_.as_str());
  }
}

#[cfg(test)]
mod tests {
  use crate::{mivot::test::get_xml, tests::test_read};

  use super::PrimaryKeyDyn;

  #[test]
  fn test_pk_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.1.xml");
    println!("testing 11.1");
    test_read::<PrimaryKeyDyn>(&xml);
  }
}
