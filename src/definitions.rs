//! Struct dedicated to the `DEFINITIONS` tag.
use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use super::{
  coosys::CooSys,
  error::VOTableError,
  param::Param,
  utils::{discard_comment, discard_event, unexpected_attr_warn},
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "def_type")]
pub enum DefinitionsElem {
  CooSys(CooSys),
  Param(Param),
}

impl DefinitionsElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      DefinitionsElem::CooSys(elem) => elem.write(writer, &()),
      DefinitionsElem::Param(elem) => elem.write(writer, &()),
    }
  }
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      DefinitionsElem::CooSys(e) => e.visit(visitor),
      DefinitionsElem::Param(e) => e.visit(visitor),
    }
  }
}

/// Struct corresponding to the `DEFINITION` XML tag.
/// Deprecated since VOTable 1.1, see
/// [IVOA doc](https://www.ivoa.net/documents/VOTable/20040811/REC-VOTable-1.1-20040811.html#ToC19)
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Definitions {
  // no attributes
  // sub-elems
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<DefinitionsElem>,
}

impl Default for Definitions {
  fn default() -> Self {
    Definitions::new()
  }
}

impl Definitions {
  pub fn new() -> Self {
    Self {
      elems: Default::default(),
    }
  }

  impl_builder_push_elem!(Param, DefinitionsElem);
  impl_builder_push_elem!(CooSys, DefinitionsElem);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_definitions_start(self)?;
    for e in &mut self.elems {
      e.visit(visitor)?;
    }
    visitor.visit_definitions_ended(self)
  }
}

impl VOTableElement for Definitions {
  const TAG: &'static str = "DEFINITIONS";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new().set_attrs(attrs)
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (k, _) in attrs {
      unexpected_attr_warn(k.as_ref(), Self::TAG);
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, _f: F)
  where
    F: FnMut(&str, &str),
  {
  }
}

impl HasSubElements for Definitions {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.elems.is_empty()
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          CooSys::TAG_BYTES => push_from_event_start!(self, CooSys, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          CooSys::TAG_BYTES => push_from_event_empty!(self, CooSys, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(()),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec_no_context!(self, elems, writer);
    Ok(())
  }
}
