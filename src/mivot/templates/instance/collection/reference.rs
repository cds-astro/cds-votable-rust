use std::{
  io::{BufRead, Write},
  str,
};

use bstringify::bstringify;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  mivot::{
    globals::collection::reference::Reference as ReferenceStatic,
    templates::instance::reference::foreign_key::ForeignKey, value_checker, VodmlVisitor,
  },
  utils::{discard_comment, discard_event, is_empty},
  QuickXmlReadWrite,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum Reference {
  Static(ReferenceStatic),
  Dynamic(ReferenceDyn),
}

impl Reference {
  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    match self {
      Reference::Static(r) => r.visit(visitor),
      Reference::Dynamic(r) => r.visit(visitor),
    }
  }
}

impl QuickXmlReadWrite for Reference {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut dmref = String::new();
    let mut sourceref = String::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        match attr.key {
          b"dmref" => dmref.push_str(value),
          b"sourceref" => sourceref.push_str(value),
          _ => return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)),
        }
      }
    }
    match (dmref.as_str(), sourceref.as_str()) {
      (dmref, "") => Ok(Reference::Static(ReferenceStatic::new(dmref))),
      ("", sourceref) => Ok(Reference::Dynamic(ReferenceDyn::new(sourceref))),
      _ =>
        Err(VOTableError::Custom(format!(
          "Either attribute 'dmref' or 'sourceref', not both, are accepted in tag '{}' child of COLLECTIOn",
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
      Reference::Static(static_ref_mut) => {
        static_ref_mut.read_sub_elements_by_ref(reader, reader_buff, context)
      }
      Reference::Dynamic(dyn_ref_mut) => {
        dyn_ref_mut.read_sub_elements_by_ref(reader, reader_buff, context)
      }
    }
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    match self {
      Reference::Static(static_ref) => static_ref.write(writer, context),
      Reference::Dynamic(dyn_ref) => dyn_ref.write(writer, context),
    }
  }
}
/// Dynamic `REFERENCE` **child of** `COLLECTION` in `TEMPLATES`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReferenceDyn {
  /// Reference to the `dmid` of the `COLLECTION` or `TEMPLATES` to be searches.
  pub sourceref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub foreignkeys: Vec<ForeignKey>, // TODO: ensure contains a least one FK!
}
impl ReferenceDyn {
  impl_new!([sourceref], [], [foreignkeys]);
  impl_empty_new!([sourceref], [], [foreignkeys]);

  impl_builder_push!(ForeignKey);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_reference_dynamic_childof_collection_in_templates(self)?;
    for fk in self.foreignkeys.iter_mut() {
      fk.visit(visitor)?;
    }
    Ok(())
  }
}

impl QuickXmlReadWrite for ReferenceDyn {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

  impl_builder_from_attr!([sourceref], []);

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
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            Self::TAG,
          ))
        }
        Event::Empty(ref e) => match e.local_name() {
          ForeignKey::TAG_BYTES => self.push_foreignkey_by_ref(ForeignKey::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          return if self.foreignkeys.is_empty() {
            Err(VOTableError::Custom(
              "A Dynamic Reference must have at least one ForeignKey".to_owned(),
            ))
          } else {
            Ok(())
          }
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    if self.foreignkeys.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer = elem_writer.with_attribute(("sourceref", self.sourceref.as_str()));
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      tag.push_attribute(("sourceref", self.sourceref.as_str()));
      writer
        .write_event(Event::Start(tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem_vec!(self, foreignkeys, writer, context);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    }
  }
}
