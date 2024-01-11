use std::str;

use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{value_checker, VodmlVisitor},
  QuickXmlReadWrite,
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
  impl_new!([ref_], []);
  impl_empty_new!([ref_], []);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_foreign_key(self)
  }
}

impl_quickrw_e!(
  [ref_, "ref"], // MANDATORY ATTRIBUTES
  [],            // OPTIONAL ATTRIBUTES
  "FOREIGN_KEY", // TAG, here : <FOREIGN\_KEY>
  ForeignKey,    // Struct on which to impl
  ()             // Context type
);

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
