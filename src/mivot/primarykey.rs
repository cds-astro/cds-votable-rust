use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

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
    "PRIMARY\\_KEY",   // TAG, here : <ATTRIBUTE>
    PrimaryKey,      // Struct on which to impl
    ()               // Context type
);
