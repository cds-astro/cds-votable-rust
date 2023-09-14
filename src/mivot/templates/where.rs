//! Defines the `WHERE` **child of** `TEMPLATES`.

use std::str;

use bstringify::bstringify;

use paste::paste;

use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};

/// The `WHERE` when it is a **child of** `TEMPLATES`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Where {
  pub primary_key: String,
  pub value: String,
}
impl Where {
  impl_new!([primary_key, value], []);
  impl_empty_new!([primary_key, value], []);
}
impl_quickrw_e!(
  [primary_key, "primarykey", value, "value"],
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
    let xml = get_xml("./resources/mivot/10/test_10_ok_10.3.xml");
    println!("testing 10.3");
    test_read::<Where>(&xml);
  }
}