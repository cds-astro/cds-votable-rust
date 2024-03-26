use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

/// `Static` primary key can be both in `GLOBALS` or in `TEMPLATES`, but `GLOBALS` contains only
/// static `PRIMARY_KEY`s
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKeyStatic {
  pub dmtype: String,
  pub value: String,
}

impl PrimaryKeyStatic {
  pub fn new<S: Into<String>>(dmtype: S, value: S) -> Self {
    Self {
      dmtype: dmtype.into(),
      value: value.into(),
    }
  }

  impl_builder_mandatory_string_attr!(dmtype);
  impl_builder_mandatory_string_attr!(value);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_primarykey_static(self)
  }
}

impl VOTableElement for PrimaryKeyStatic {
  const TAG: &'static str = "PRIMARY_KEY";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("", "").set_attrs(attrs).and_then(|pk| {
      if pk.dmtype.is_empty() || pk.value.is_empty() {
        Err(VOTableError::Custom(format!(
          "Mandatory attributes 'dmtype' or 'value' not found in tag '{}'",
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
    f("dmtype", self.dmtype.as_str());
    f("value", self.value.as_str());
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  use super::PrimaryKeyStatic;

  #[test]
  fn test_pk_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.2.xml");
    println!("testing 11.2");
    test_read::<PrimaryKeyStatic>(&xml);
    // Should not be valid according to: empty attribute <=> np attribute, right?!
    //  let xml = get_xml("./resources/mivot/11/test_11_ok_11.8.xml");
    //  println!("testing 11.8");
    //  test_read::<PrimaryKeyStatic>(&xml);

    // KO MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.3.xml");
    println!("testing 11.3"); // Name required.
    test_error::<PrimaryKeyStatic>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.4.xml");
    println!("testing 11.4"); // Name required.
    test_error::<PrimaryKeyStatic>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.5.xml");
    println!("testing 11.5"); // Name required.
    test_error::<PrimaryKeyStatic>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.6.xml");
    println!("testing 11.6"); // Name required.
    test_error::<PrimaryKeyStatic>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.7.xml");
    println!("testing 11.7"); // Name required.
    test_error::<PrimaryKeyStatic>(&xml, false);
  }
}
