//! Contains the static `REFERENCE` structure which is **child of** `COLLECTION` in `GLOBALS`.
//!
//! A `REFERENCE` is made to be replaced by an `INSTANCE` or a `COLLECTION` that can be retrieved
//! either dynamically (in `TEMPLATES`) or statically (in `GLOBALS` or in `TEMPLATES`).

use std::str;

use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};

/// Static `REFERENCE` **child of** `COLLECTION` in `GLOBALS`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Reference {
  /// `dmid` of the referenced `INSTANCE` or `COLLECTION`.
  pub dmref: String,
}

impl Reference {
  impl_new!([dmref], []);
  impl_empty_new!([dmref], []);
}
impl_quickrw_e! {
  [dmref],
  [],
  "REFERENCE",
  Reference,
  ()
}