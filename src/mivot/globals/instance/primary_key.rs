use std::str;

use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{error::VOTableError, mivot::VodmlVisitor, QuickXmlReadWrite};

/// `Static` primary key can be both in `GLOBALS` or in `TEMPLATES`, but `GLOBALS` contains only
/// static `PRIMARY_KEY`s
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKeyStatic {
  pub dmtype: String,
  pub value: String,
}
impl PrimaryKeyStatic {
  impl_new!([dmtype, value], []);
  impl_empty_new!([dmtype, value], []);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_primarykey_static(self)
  }
}

impl QuickXmlReadWrite for PrimaryKeyStatic {
  const TAG: &'static str = "PRIMARY_KEY";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut tag = Self::new_empty();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        tag = match attr.key {
          b"dmtype" => {
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
