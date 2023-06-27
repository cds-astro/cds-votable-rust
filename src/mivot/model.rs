use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::{Reader, Writer};
use std::str;

/*
    struct Model
    @elem name String: Name of the mapped model as declared in the VO-DML/XML model serialization. => MAND
    @elem url Option<String>: URL to the VO-DML/XML serialization of the model. If present, this attribute MUST not be empty. => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Model {
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  url: Option<String>,
}
impl Model {
    impl_empty_new!([name], [url]);
    impl_builder_opt_string_attr!(url);
}
impl_quickrw_e!(
    [name],  // MANDATORY ATTRIBUTES
    [url],   // OPTIONAL ATTRIBUTES
    "MODEL", // TAG, here : <ATTRIBUTE>
    Model,   // Struct on which to impl
    ()       // Context type
);
