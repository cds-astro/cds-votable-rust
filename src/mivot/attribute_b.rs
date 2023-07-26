use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct Attribute => pattern b generic attribute
    @elem dmrole String: Modeled node related => MAND
    @elem dmtype String: Modeled node related => MAND
    @elem value String: attribute default value => MAND
    @elem array_index Option<String>: attribute size of array, only present if the attribute is an array (example: String = char array) => OPT
    @elem unit Option<String>: the unit used for the value (example: km/s) => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributePatB {
  // MANDATORY
  pub dmrole: String,
  pub dmtype: String,
  pub value: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  pub array_index: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
}
impl AttributePatB {
  impl_empty_new!([dmrole, dmtype, value], [array_index, unit]);
  /*
      function setters, enable the setting of an optional through self.set_"var"
  */
  impl_builder_opt_string_attr!(array_index);
  impl_builder_opt_string_attr!(unit);
  impl_builder_mand_string_attr!(dmrole);
  impl_builder_mand_string_attr!(dmtype);
  impl_builder_mand_string_attr!(value);
}
impl_quickrw_e!(
  [dmrole, dmtype, value], // MANDATORY ATTRIBUTES
  [array_index, unit],     // OPTIONAL ATTRIBUTES
  "ATTRIBUTE",             // TAG, here : <ATTRIBUTE>
  AttributePatB,           // Struct on which to impl
  ()                       // Context type
);
