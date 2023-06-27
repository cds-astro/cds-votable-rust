use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::{Reader, Writer};
use std::str;

/*
    struct Globals or templates instance => pattern a
    @elem primary_key String: FIELD identifier of the primary key column => MAND
    @elem foreign_key Option<String>: FIELD identifier of the foreign key column => NO or MAND mutually exclusive with value
    @elem value Option<String>: Literal key value. Used when the key relates to a COLLECTION in the GLOBALS => NO or MAND mutually exclusive with foreign_key
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Where {
    primary_key: String,
    foreign_key: Option<String>,
    value: Option<String>,
}
impl Where {
    impl_empty_new!([primary_key], [foreign_key, value]);
    impl_builder_opt_string_attr!(foreign_key);
    impl_builder_opt_string_attr!(value);
}
impl_quickrw_e!(
    [primary_key, "primarykey"],
    [foreign_key, "foreignkey", value],
    "WHERE",
    Where,
    ()
);
