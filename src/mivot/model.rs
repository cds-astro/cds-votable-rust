//! Module dedicated to the `MODEL` tag.

use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

/// Structure storing the content of the `MODEL` tag.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Model {
  ///  Name of the mapped model as declared in the VO-DML/XML model serialization
  pub name: String,
  /// URL to the VO-DML/XML serialization of the model (optional, but **must not** be empty if present).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>,
}
impl Model {
  pub fn new<S: Into<String>>(name: S) -> Self {
    Self {
      name: name.into(),
      url: None,
    }
  }

  impl_builder_mandatory_string_attr!(name);
  impl_builder_opt_string_attr!(url);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_model(self)
  }
}
impl VOTableElement for Model {
  const TAG: &'static str = "MODEL";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("").set_attrs(attrs).and_then(|model| {
      if model.name.is_empty() {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'name' not found or empty in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(model)
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
        "name" => self.set_name_by_ref(val),
        "url" => {
          if val.as_ref().is_empty() {
            return Err(VOTableError::Custom(String::from(
              "'url' attribute must not be empty",
            )));
          } else {
            self.set_url_by_ref(val)
          }
        }
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("name", self.name.as_str());
    if let Some(url) = &self.url {
      f("url", url.as_str());
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::model::Model,
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  #[test]
  fn test_model_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/2/test_2_ok_2.1.xml");
    println!("testing 2.1");
    test_read::<Model>(&xml);
    let xml = get_xml("./resources/mivot/2/test_2_ok_2.2.xml");
    println!("testing 2.2");
    test_read::<Model>(&xml);
    // KO MODELS
    let xml = get_xml("./resources/mivot/2/test_2_ko_2.3.xml");
    println!("testing 2.3"); // Name required.
    test_error::<Model>(&xml, false);
    let xml = get_xml("./resources/mivot/2/test_2_ko_2.4.xml");
    println!("testing 2.4"); // Name must not be empty.
    test_error::<Model>(&xml, false);
    let xml = get_xml("./resources/mivot/2/test_2_ko_2.5.xml");
    println!("testing 2.5"); // Url must not be empty (when present).
    test_error::<Model>(&xml, false);
  }
}
