//! Struct dedicated to the `DEFINITIONS` tag.
use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use super::{
  coosys::CooSys,
  error::VOTableError,
  param::Param,
  utils::{discard_comment, discard_event},
  QuickXmlReadWrite, TableDataContent, VOTableVisitor,
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

impl QuickXmlReadWrite for Definitions {
  const TAG: &'static str = "DEFINITIONS";
  type Context = ();

  fn from_attributes(mut attrs: Attributes) -> Result<Self, VOTableError> {
    let definitions = Self::new();
    if let Some(attr_res) = attrs.next() {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
    }
    Ok(definitions)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, context)
      .map(|()| reader)
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
          CooSys::TAG_BYTES => self
            .elems
            .push(DefinitionsElem::CooSys(from_event_start_by_ref!(
              CooSys,
              reader,
              reader_buff,
              e
            ))),
          Param::TAG_BYTES => self
            .elems
            .push(DefinitionsElem::Param(from_event_start_by_ref!(
              Param,
              reader,
              reader_buff,
              e
            ))),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          CooSys::TAG_BYTES => self
            .elems
            .push(DefinitionsElem::CooSys(CooSys::from_event_empty(e)?)),
          Param::TAG_BYTES => self
            .elems
            .push(DefinitionsElem::Param(Param::from_event_empty(e)?)),
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

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    if self.elems.is_empty() {
      let elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag
      writer
        .write_event(Event::Start(tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem_vec_no_context!(self, elems, writer);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    }
  }
}
