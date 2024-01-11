//! Contains the static `REFERENCE` structures which is **child of** `INSTANCE` in `GLOBALS`.
//!
//! A `REFERENCE` is made to be replaced by an `INSTANCE` or a `COLLECTION` that can be retrieved
//! either dynamically (in `TEMPLATES`) or statically (in `GLOBALS` or in `TEMPLATES`).

use std::str;

use bstringify::bstringify;

use paste::paste;

use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{value_checker, VodmlVisitor},
  QuickXmlReadWrite,
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
  impl_new!([dmrole, dmref], []);
  impl_empty_new!([dmrole, dmref], []);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_reference_static_childof_instance(self)
  }
}
impl_quickrw_e! {
  [dmrole, dmref],
  [],
  "REFERENCE",
  Reference,
  ()
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
