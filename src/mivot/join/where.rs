//! Defines the `WHERE` **child of** `JOIN`.

use std::str;

use bstringify::bstringify;

use paste::paste;

use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{value_checker, VodmlVisitor},
  QuickXmlReadWrite,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// The `WHERE` when it is a **child of** `JOIN`.
pub struct Where {
  pub foreign_key: String,
  pub primary_key: String,
}

impl Where {
  impl_new!([primary_key, foreign_key], []);
  impl_empty_new!([primary_key, foreign_key], []);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_where_childof_join(self)
  }
}

impl_quickrw_e!(
  [primary_key, "primarykey", foreign_key, "foreignkey"],
  [],
  "WHERE",
  Where,
  ()
);

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
