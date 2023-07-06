use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct PrimaryKey
    @elem value String: attribute default value => MAND
    @elem dmtype String: Modeled node related => MAND
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKey {
  dmtype: String,
  value: String,
}
impl PrimaryKey {
  impl_empty_new!([dmtype, value], []);
}
impl_quickrw_e!(
  [dmtype, value], // MANDATORY ATTRIBUTES
  [],              // OPTIONAL ATTRIBUTES
  "PRIMARY\\_KEY", // TAG, here : <ATTRIBUTE>
  PrimaryKey,      // Struct on which to impl
  ()               // Context type
);

#[cfg(test)]
mod tests {
  use crate::{
    mivot::{
      primarykey::PrimaryKey,
      test::{get_xml, test_error},
    },
    tests::test_read,
  };

  #[test]
  fn test_pk_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.1.xml");
    println!("testing 11.1");
    test_read::<PrimaryKey>(&xml);
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.2.xml");
    println!("testing 11.2");
    test_read::<PrimaryKey>(&xml);
    let xml = get_xml("./resources/mivot/11/test_10_ok_11.8.xml");
    println!("testing 11.8");
    test_read::<PrimaryKey>(&xml);

    // KO MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.3.xml");
    println!("testing 11.3"); // Name required.
    test_error::<PrimaryKey>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.4.xml");
    println!("testing 11.4"); // Name required.
    test_error::<PrimaryKey>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.5.xml");
    println!("testing 11.5"); // Name required.
    test_error::<PrimaryKey>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.6.xml");
    println!("testing 11.6"); // Name required.
    test_error::<PrimaryKey>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.7.xml");
    println!("testing 11.7"); // Name required.
    test_error::<PrimaryKey>(&xml, false);
  }
}
