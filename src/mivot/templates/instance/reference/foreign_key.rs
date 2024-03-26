use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

/// Only used in `REFERENCE` in `TEMPLATE`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForeignKey {
  /// Identifier of the `FIELD` (in a table of the `VOTable`) that must match the primary key of
  /// the referenced collection.
  #[serde(rename = "ref")]
  pub ref_: String,
}
impl ForeignKey {
  pub fn new<S: Into<String>>(ref_: S) -> Self {
    Self { ref_: ref_.into() }
  }

  impl_builder_mandatory_string_attr!(ref_, ref);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_foreign_key(self)
  }
}

impl VOTableElement for ForeignKey {
  const TAG: &'static str = "FOREIGN_KEY";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("").set_attrs(attrs).and_then(|fk| {
      if fk.ref_.is_empty() {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'ref' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(fk)
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
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("ref", self.ref_.as_str());
  }
}

#[cfg(test)]
mod tests {
  use super::ForeignKey;
  use crate::{
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  #[test]
  fn test_fk_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/12/test_12_ok_12.1.xml");
    println!("testing 12.1");
    test_read::<ForeignKey>(&xml);

    // KO MODELS
    let xml = get_xml("./resources/mivot/12/test_12_ko_12.2.xml");
    println!("testing 12.2"); // Name required.
    test_error::<ForeignKey>(&xml, false);
    let xml = get_xml("./resources/mivot/12/test_12_ko_12.3.xml");
    println!("testing 12.3"); // Name required.
    test_error::<ForeignKey>(&xml, false);
  }
}
