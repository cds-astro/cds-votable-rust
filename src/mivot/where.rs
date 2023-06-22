use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::{Reader, Writer};
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Where {
  primary_key: String,
  foreign_key: String,
}
impl Where {
  impl_empty_new!([primary_key, foreign_key], []);
}
impl_quickrw_e!([primary_key, foreign_key], [], "WHERE", Where, ());
