use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ForeignKey {
  #[serde(rename = "ref")]
  ref_: String,
}
impl ForeignKey {
    impl_empty_new!([ref_], []);
}

impl_quickrw_e!(
    [ref_, "ref"], // MANDATORY ATTRIBUTES
    [],            // OPTIONAL ATTRIBUTES
    "FOREIGN_KEY", // TAG, here : <ATTRIBUTE>
    ForeignKey,    // Struct on which to impl
    ()             // Context type
);
