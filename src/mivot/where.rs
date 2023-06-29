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
    foreign_key: String,
}
impl Where {
    impl_empty_new!([primary_key, foreign_key], []);
}
impl_quickrw_e!(
    [primary_key, "primarykey", foreign_key, "foreignkey"],
    [],
    "WHERE",
    Where,
    ()
);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NoFkWhere {
    primary_key: String,
    value: String,
}
impl NoFkWhere {
    impl_empty_new!([primary_key, value], []);
}
impl_quickrw_e!(
    [primary_key, "primarykey", value, "value"],
    [],
    "WHERE",
    NoFkWhere,
    ()
);
