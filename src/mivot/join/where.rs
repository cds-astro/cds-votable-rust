//! Defines the `WHERE` **child of** `JOIN`.

use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// The `WHERE` when it is a **child of** `JOIN`.
pub struct Where {
  #[serde(rename = "foreignkey")]
  pub foreign_key: String,
  #[serde(rename = "primarykey")]
  pub primary_key: String,
}

impl Where {
  pub fn new<S: Into<String>>(foreign_key: S, primary_key: S) -> Self {
    Self {
      foreign_key: foreign_key.into(),
      primary_key: primary_key.into(),
    }
  }

  impl_builder_mandatory_string_attr!(foreign_key, foreignkey);
  impl_builder_mandatory_string_attr!(primary_key, primarykey);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_where_childof_join(self)
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
    Self::new("", "").set_attrs(attrs).and_then(|w| {
      if w.foreign_key.is_empty() || w.primary_key.is_empty() {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'foreignkey' and/or 'primarykey' not found in tag '{}'",
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
        "foreignkey" => self.set_foreignkey_by_ref(val),
        "primarykey" => self.set_primarykey_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("foreignkey", self.foreign_key.as_str());
    f("primarykey", self.primary_key.as_str());
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  use super::Where;

  #[test]
  fn test_where_read() {
    // OK WHERES
    let xml = get_xml("./resources/mivot/10/test_10_ok_10.2.xml");
    println!("testing 10.2");
    test_read::<Where>(&xml);

    // KO WHERES
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.1.xml");
    println!("testing 10.1"); // Name required.
    test_error::<Where>(&xml, false);
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.5.xml");
    println!("testing 10.5"); // Name required.
    test_error::<Where>(&xml, false);
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.6.xml");
    println!("testing 10.6"); // Name required.
    test_error::<Where>(&xml, false);
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.7.xml");
    println!("testing 10.7"); // Name required.
    test_error::<Where>(&xml, false);
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.8.xml");
    println!("testing 10.8"); // Name required.
    test_error::<Where>(&xml, false);
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.9.xml");
    println!("testing 10.9"); // Name required.
    test_error::<Where>(&xml, false);
    let xml = get_xml("./resources/mivot/10/test_10_ko_10.10.xml");
    println!("testing 10.10"); // Name required.
    test_error::<Where>(&xml, false);
  }
}
