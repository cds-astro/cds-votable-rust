//! Module dedicated to the `MODEL` tag.

use std::str;

use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{value_checker, VodmlVisitor},
  QuickXmlReadWrite,
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
  impl_new!([name], [url]);
  impl_empty_new!([name], [url]);
  impl_builder_opt_string_attr!(url);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_model(self)
  }
}
impl_quickrw_e!(
  [name],  // MANDATORY ATTRIBUTES
  [url],   // OPTIONAL ATTRIBUTES
  "MODEL", // TAG, here : <ATTRIBUTE>
  Model,   // Struct on which to impl
  ()       // Context type
);

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
