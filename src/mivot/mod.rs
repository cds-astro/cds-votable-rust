use std::io::Write;

use quick_xml::Writer;

use crate::error::VOTableError;

pub mod attribute;
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
    fn write<W: Write>(
        &mut self,
        writer: &mut Writer<W>,
    ) -> Result<(), VOTableError>;
}
pub trait ElemImpl<T: ElemType> {
    fn push_to_elems(&mut self, elem: T);
}
