//! Define the `PrimaryKey` structures which are specific to `TEMPLATES`.

use std::{
  io::{BufRead, Write},
  str,
};

use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{globals::instance::primary_key::PrimaryKeyStatic, value_checker, VodmlVisitor},
  QuickXmlReadWrite,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum PrimaryKey {
  Static(PrimaryKeyStatic),
  Dynamic(PrimaryKeyDyn),
}

impl PrimaryKey {
  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    match self {
      PrimaryKey::Static(pk) => pk.visit(visitor),
      PrimaryKey::Dynamic(pk) => pk.visit(visitor),
    }
  }
}

impl QuickXmlReadWrite for PrimaryKey {
  const TAG: &'static str = "PRIMARY_KEY";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut dmtype = String::new();
    let mut value = String::new(); // static
    let mut ref_ = String::new(); // dynamic
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let val = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !val.is_empty() {
        match attr.key {
          b"dmtype" => dmtype.push_str(val),
          b"value" => value.push_str(val),
          b"ref" => ref_.push_str(val),
          _ => return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)),
        }
      }
    }
    match (dmtype.as_str(), value.as_str(), ref_.as_str()) {
      ("", _, _) => Err(VOTableError::Custom(format!(
        "Attribute 'dmtype' is mandatory and must be non-empty in tag '{}'",
        Self::TAG
      ))),
      (_, "", "") => Err(VOTableError::Custom(format!(
        "Either attribute 'value' or 'ref' is mandatory and must be non-empty in tag '{}'",
        Self::TAG
      ))),
      (dmtype, value, "") => Ok(PrimaryKey::Static(PrimaryKeyStatic::new(dmtype, value))),
      (dmtype, "", ref_) => Ok(PrimaryKey::Dynamic(PrimaryKeyDyn::new(dmtype, ref_))),
      _ => Err(VOTableError::Custom(format!(
        "Either attribute 'value' or 'ref' is mandatory and must be non-empty in tag '{}'",
        Self::TAG
      ))),
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, _context)
      .map(|()| reader)
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    match self {
      PrimaryKey::Static(static_pk_ref_mut) => {
        static_pk_ref_mut.read_sub_elements_by_ref(reader, reader_buff, context)
      }
      PrimaryKey::Dynamic(dyn_pk_ref_mut) => {
        dyn_pk_ref_mut.read_sub_elements_by_ref(reader, reader_buff, context)
      }
    }
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    match self {
      PrimaryKey::Static(pk) => pk.write(writer, context),
      PrimaryKey::Dynamic(pk) => pk.write(writer, context),
    }
  }
}

/// `Dynamic` primary key are only possible in `TEMPLATE` since
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKeyDyn {
  /// Type of the key.
  pub dmtype: String,
  /// Reference to a `FIELD` in a `TABLE` of the `VOTable`.
  pub ref_: String,
}
impl PrimaryKeyDyn {
  impl_new!([dmtype, ref_], []);
  impl_empty_new!([dmtype, ref_], []);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_primarykey_dynamic(self)
  }
}
impl_quickrw_e!(
  [dmtype, "dmtype", ref_, "ref"], // MANDATORY ATTRIBUTES
  [],                              // OPTIONAL ATTRIBUTES
  "PRIMARY_KEY",                   // TAG, here : <ATTRIBUTE>
  PrimaryKeyDyn,                   // Struct on which to impl
  ()                               // Context type
);

#[cfg(test)]
mod tests {
  use crate::{mivot::test::get_xml, tests::test_read};

  use super::PrimaryKeyDyn;

  #[test]
  fn test_pk_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/11/test_11_ok_11.1.xml");
    println!("testing 11.1");
    test_read::<PrimaryKeyDyn>(&xml);
  }
}
