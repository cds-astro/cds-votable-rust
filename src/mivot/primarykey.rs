use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct PrimaryKeyB in Collection
    @elem value String: attribute default value => MAND
    @elem dmtype String: Modeled node related => MAND
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKeyB {
  dmtype: String,
  value: String,
}
impl PrimaryKeyB {
  impl_empty_new!([dmtype, value], []);
}
impl QuickXmlReadWrite for PrimaryKeyB {
  const TAG: &'static str = "PRIMARY\\_KEY";
  type Context = ();
  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut tag = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      tag = match attr.key {
        b"dmtype" => {
          value_checker(value, &"dmtype")?;
          tag.dmtype = value.to_string();
          tag
        }
        b"value" => {
          tag.value = value.to_string();
          tag
        }
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      };
    }
    if tag.dmtype.as_str() == NULL || tag.value.as_str() == NULL {
      Err(VOTableError::Custom(format!(
        "Attributes dmtype value are mandatory in tag {}",
        Self::TAG
      )))
    } else {
      Ok(tag)
    }
  }
  empty_read_sub!();
  impl_write_e!([dmtype, value], []);
}

/*
    struct PrimaryKeyA in Instance
    @elem value String: attribute default value => MAND
    @elem dmtype String: Modeled node related => MAND
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKeyA {
  dmtype: String,
  ref_: String,
}
impl PrimaryKeyA {
  impl_empty_new!([dmtype, ref_], []);
}
impl_quickrw_e!(
  [dmtype, "dmtype", ref_, "ref"], // MANDATORY ATTRIBUTES
  [],                              // OPTIONAL ATTRIBUTES
  "PRIMARY\\_KEY",                 // TAG, here : <ATTRIBUTE>
  PrimaryKeyA,                     // Struct on which to impl
  ()                               // Context type
);

#[cfg(test)]
mod tests {
  use crate::{
    mivot::{
      primarykey::{PrimaryKeyA, PrimaryKeyB},
      test::{get_xml, test_error},
    },
    tests::test_read,
  };

  #[test]
  fn test_pk_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.1.xml");
    println!("testing 11.1");
    test_read::<PrimaryKeyA>(&xml);
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.2.xml");
    println!("testing 11.2");
    test_read::<PrimaryKeyB>(&xml);
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.8.xml");
    println!("testing 11.8");
    test_read::<PrimaryKeyB>(&xml);

    // KO MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.3.xml");
    println!("testing 11.3"); // Name required.
    test_error::<PrimaryKeyB>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.4.xml");
    println!("testing 11.4"); // Name required.
    test_error::<PrimaryKeyB>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.5.xml");
    println!("testing 11.5"); // Name required.
    test_error::<PrimaryKeyB>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.6.xml");
    println!("testing 11.6"); // Name required.
    test_error::<PrimaryKeyB>(&xml, false);
    let xml = get_xml("./resources/mivot/11/test_11_ko_11.7.xml");
    println!("testing 11.7"); // Name required.
    test_error::<PrimaryKeyB>(&xml, false);
  }
}
