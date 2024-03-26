//! Defines the `WHERE` **child of** `TEMPLATES`.

use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

/// The `WHERE` when it is a **child of** `TEMPLATES`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Where {
  #[serde(rename = "primarykey")]
  pub primary_key: String,
  pub value: String,
}
impl Where {
  pub fn new<S: Into<String>>(primary_key: S, value: S) -> Self {
    Self {
      primary_key: primary_key.into(),
      value: value.into(),
    }
  }

  impl_builder_mandatory_string_attr!(primary_key, primarykey);
  impl_builder_mandatory_string_attr!(value);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_where_childof_templates(self)
  }
}

impl VOTableElement for Where {
  const TAG: &'static str = "WHERE";

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
      .and_then(|w| {
        if w.primary_key.as_str() == DEFAULT_VALUE || w.value.as_str() == DEFAULT_VALUE {
          Err(VOTableError::Custom(format!(
            "Mandatory attribute 'primarykey' and/or 'value' not found in tag '{}'",
            Self::TAG
          )))
        } else {
          Ok(w)
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
        "primarykey" => self.set_primarykey_by_ref(val),
        "value" => self.set_value_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("primarykey", self.primary_key.as_str());
    f("value", self.value.as_str());
  }
}

#[cfg(test)]
mod tests {
  use crate::{mivot::test::get_xml, tests::test_read};

  use super::Where;

  #[test]
  fn test_where_read() {
    // OK WHERES
    let xml = get_xml("./resources/mivot/10/test_10_ok_10.3.xml");
    println!("testing 10.3");
    test_read::<Where>(&xml);
  }
}
