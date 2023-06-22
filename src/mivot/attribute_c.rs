use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
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
impl_quickrw_e!(
    [dmtype],                                // MANDATORY ATTRIBUTES
    [ref_, "ref", value, array_index, unit], // OPTIONAL ATTRIBUTES
    "ATTRIBUTE",                             // TAG, here : <ATTRIBUTE>
    AttributePatC,                           // Struct on which to impl
    ()                                       // Context type
);
