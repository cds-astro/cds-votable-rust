use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::{Reader, Writer};
use std::str;

/*
    struct Globals or templates instance => pattern a
    @elem primary_key String: FIELD identifier of the primary key column => MAND
    @elem foreign_key Option<String>: FIELD identifier of the foreign key column => NO or MAND mutually exclusive with value
    @elem value Option<String>: Literal key value. Used when the key relates to a COLLECTION in the GLOBALS => NO or MAND mutually exclusive with foreign_key
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Where {
  pub primary_key: String,
  pub foreign_key: String,
}
impl Where {
  impl_new!([primary_key, foreign_key], []);
  impl_empty_new!([primary_key, foreign_key], []);
}
impl_quickrw_e!(
  [primary_key, "primarykey", foreign_key, "foreignkey"],
  [],
  "WHERE",
  Where,
  ()
);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NoFkWhere {
  pub primary_key: String,
  pub value: String,
}
impl NoFkWhere {
  impl_new!([primary_key, value], []);
  impl_empty_new!([primary_key, value], []);
}
impl_quickrw_e!(
  [primary_key, "primarykey", value, "value"],
  [],
  "WHERE",
  NoFkWhere,
  ()
);

#[cfg(test)]
mod tests {
  use crate::{
    mivot::{
      r#where::{NoFkWhere, Where},
      test::{get_xml, test_error},
    },
    tests::test_read,
  };

  #[test]
  fn test_where_read() {
    // OK WHERES
    let xml = get_xml("./resources/mivot/10/test_10_ok_10.2.xml");
    println!("testing 10.2");
    test_read::<Where>(&xml);
    let xml = get_xml("./resources/mivot/10/test_10_ok_10.3.xml");
    println!("testing 10.3");
    test_read::<NoFkWhere>(&xml);

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
