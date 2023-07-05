use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct Attribute => pattern a valid in Instances
    @elem dmrole String: Modeled node related => MAND
    @elem dmtype String: Modeled node related => MAND
    @elem ref Option<String>: reference to a VOTable element => OPT
    @elem value Option<String>: attribute default value => OPT
    @elem array_index Option<String>: attribute size of array, only present if the attribute is an array (example: String = char array) => OPT
    @elem unit Option<String>: the unit used for the value (example: km/s) => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributePatA {
  // MANDATORY
  dmrole: String,
  dmtype: String,
  // OPTIONAL
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  value: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  array_index: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  unit: Option<String>,
}
impl AttributePatA {
  impl_empty_new!([dmrole, dmtype], [ref_, value, array_index, unit]);
  /*
      function setters, enable the setting of an optional through self.set_"var"
  */
  impl_builder_opt_string_attr!(ref_);
  impl_builder_opt_string_attr!(value);
  impl_builder_opt_string_attr!(array_index);
  impl_builder_opt_string_attr!(unit);
}
impl QuickXmlReadWrite for AttributePatA {
  const TAG: &'static str = "ATTRIBUTE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut tag = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      tag = match attr.key {
        b"dmrole" => {
          value_checker(value, "dmrole")?;
          tag.dmrole = value.to_string();
          tag
        }
        b"dmtype" => {
          value_checker(value, "dmtype")?;
          tag.dmtype = value.to_string();
          tag
        }

        b"ref" => {
          value_checker(value, "ref")?;
          tag.set_ref_(value)
        }

        b"value" => tag.set_value(value),

        b"arrayindex" => {
          value_checker(value, "arrayindex")?;
          if value.parse::<i32>().is_ok() {
            if !(value.parse::<i32>().unwrap() < 0) {
              tag.set_array_index(value)
            } else {
              return Err(VOTableError::Custom(
                "If attribute array_index is present it must be a superior or equal to 0"
                  .to_owned(),
              ));
            }
          } else {
            return Err(VOTableError::Custom(
              "If attribute array_index is present it must be a number".to_owned(),
            ));
          }
        }

        b"unit" => {
          value_checker(value, "unit")?;
          tag.set_unit(value)
        }

        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      };
    }
    if tag.dmrole.as_str() == NULL || tag.dmtype.as_str() == NULL {
      Err(VOTableError::Custom(format!(
        "Attributes dmrole dmtype are mandatory in tag {}",
        &Self::TAG
      )))
    } else if tag.ref_.is_none() && tag.value.is_none() {
      Err(VOTableError::Custom(format!(
        "One of the attributes ref or value should be present in tag {}",
        &Self::TAG
      )))
    } else if tag.array_index.is_some() && tag.ref_.is_none() {
      Err(VOTableError::Custom(format!(
        "Arrayindex is only associated with ref in {}",
        &Self::TAG
      )))
    } else {
      Ok(tag)
    }
  }

  empty_read_sub!();

  impl_write_e!([dmrole, dmtype], [ref_, "ref", value, unit, array_index]);
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::attribute_a::AttributePatA,
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  #[test]
  fn test_attribute_a_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.1.xml");
    println!("testing 7.1");
    test_read::<AttributePatA>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.2.xml");
    println!("testing 7.2");
    test_read::<AttributePatA>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.3.xml");
    println!("testing 7.3");
    test_read::<AttributePatA>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.5.xml");
    println!("testing 7.5");
    test_read::<AttributePatA>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.6.xml");
    println!("testing 7.6");
    test_read::<AttributePatA>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.7.xml");
    println!("testing 7.7");
    test_read::<AttributePatA>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.14.xml");
    println!("testing 7.14");
    test_read::<AttributePatA>(&xml);
    // KO MODELS
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.4.xml");
    println!("testing 7.4"); // valid dmrole + dmtype; must have one or both of (value, ref)
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.8.xml");
    println!("testing 7.8"); // valid dmrole + dmtype + value + arrayindex; arrayindex only assoc. with ref.
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.9.xml");
    println!("testing 7.9"); // no dmrole + dmtype + value; must have non-empty dmrole
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.10.xml");
    println!("testing 7.10"); // valid dmrole + no dmtype + value; must have non-empty dmtype.
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.11.xml");
    println!("testing 7.11"); // empty dmrole + dmtype + ref; must have non-empty dmrole in this context
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.12.xml");
    println!("testing 7.12"); // dmtype must not be empty
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.13.xml");
    println!("testing 7.13"); // ref must not be empty
    test_error::<AttributePatA>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.15.xml");
    println!("testing 7.15"); // valid dmrole + dmtype + ref + (arrayindex < 0); invalid arrayindex value (< 0)
    test_error::<AttributePatA>(&xml, false);
  }
}
