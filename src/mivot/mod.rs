extern crate core;

use crate::error::VOTableError;
use quick_xml::Writer;
use std::io::Write;

use self::primarykey::PrimaryKey;

#[macro_use]
pub mod macros;

pub mod attribute_a;
pub mod attribute_b;
pub mod attribute_c;
pub mod collection;
pub mod foreignkey;
pub mod globals;
pub mod instance;
pub mod join;
pub mod model;
pub mod primarykey;
pub mod reference;
pub mod report;
pub mod templates;
pub mod vodml;
pub mod r#where;

pub trait ElemType {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError>;
}
pub trait ElemImpl<T: ElemType> {
  fn push_to_elems(&mut self, elem: T);
}

pub trait InstanceType {
    fn push2_pk(&mut self, pk: PrimaryKey);
}