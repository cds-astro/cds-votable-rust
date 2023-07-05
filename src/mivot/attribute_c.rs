use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct Attribute => pattern c valid in collections
    @elem dmtype String: Modeled node related => MAND
    @elem ref Option<String>: reference to a VOTable element => OPT
    @elem value Option<String>: attribute default value => OPT
    @elem array_index Option<String>: attribute size of array, only present if the attribute is an array (example: String = char array) => OPT
    @elem unit Option<String>: the unit used for the value (example: km/s) => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributePatC {
  // MANDATORY
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
impl AttributePatC {
  impl_empty_new!([dmtype], [ref_, value, array_index, unit]);

  /*
      function setters, enable the setting of an optional through self.set_"var"
  */
  impl_builder_opt_string_attr!(ref_);
  impl_builder_opt_string_attr!(value);
  impl_builder_opt_string_attr!(array_index);
  impl_builder_opt_string_attr!(unit);
}
impl QuickXmlReadWrite for AttributePatC {
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
        b"dmtype" => {
          value_checker(value, "dmtype")?;
          tag.dmtype = value.to_string();
          tag
        }

        b"dmrole" => tag,

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
          if !(value.is_empty() || value.len() == 0) {
            tag.set_unit(value)
          } else {
            tag
          }
        }

        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      };
    }
    if tag.dmtype.as_str() == NULL {
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

  impl_write_e!([dmtype], [ref_, "ref", value, unit, array_index]);
}
// impl_quickrw_e!(
//   [dmtype],                                // MANDATORY ATTRIBUTES
//   [ref_, "ref", value, array_index, unit], // OPTIONAL ATTRIBUTES
//   [dmrole],
//   "ATTRIBUTE",                             // TAG, here : <ATTRIBUTE>
//   AttributePatC,                           // Struct on which to impl
//   ()                                       // Context type
// );

#[cfg(test)]
mod tests {
  use crate::{
    mivot::{
      attribute_c::AttributePatC,
      test::{get_xml},
    },
    tests::test_read,
  };

  #[test]
  fn test_attribute_c_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.17.xml");
    println!("testing 7.17");
    test_read::<AttributePatC>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.18.xml");
    println!("testing 7.18");
    test_read::<AttributePatC>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.19.xml");
    println!("testing 7.19");
    test_read::<AttributePatC>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.20.xml");
    println!("testing 7.20");
    test_read::<AttributePatC>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.21.xml");
    println!("testing 7.21");
    test_read::<AttributePatC>(&xml);

    // KO MODELS
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.16.xml");
    println!("testing 7.16"); // dmrole + dmtype + ref; must have no or empty dmrole in this context. (parser can overlook this and write it correctly later)
    test_read::<AttributePatC>(&xml); // Should read correctly
  }
}
