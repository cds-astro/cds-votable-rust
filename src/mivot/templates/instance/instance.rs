//! Here `INSTANCE` is a **child of** `INSTANCE`.
//! Hence, it **must have** `dmrole`.

use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  mivot::{attribute::AttributeChildOfInstance as Attribute, VodmlVisitor},
  utils::{discard_comment, discard_event, is_empty},
  QuickXmlReadWrite,
};

use super::{collection::Collection, primary_key::PrimaryKey, reference::Reference, InstanceElem};

/// The same as `INSTANCE` **child of** `GLOBALS`, but with having a `dmrole`.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dmid: Option<String>,
  pub dmrole: String,
  pub dmtype: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub primarykeys: Vec<PrimaryKey>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<InstanceElem>,
}

impl Instance {
  impl_new!([dmrole, dmtype], [dmid], [primarykeys, elems]);

  impl_builder_opt_string_attr!(dmid);

  impl_builder_push!(PrimaryKey);

  impl_builder_push_elem!(Attribute, InstanceElem);
  impl_builder_push_elem!(Instance, InstanceElem);
  impl_builder_push_elem!(Reference, InstanceElem);
  impl_builder_push_elem!(Collection, InstanceElem);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_instance_childof_instance_in_templates(self)?;
    for pk in self.primarykeys.iter_mut() {
      pk.visit(visitor)?;
    }
    for elem in self.elems.iter_mut() {
      elem.visit(visitor)?;
    }
    Ok(())
  }
}

impl QuickXmlReadWrite for Instance {
  const TAG: &'static str = "INSTANCE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut dmid = String::new();
    let mut dmrole = String::new();
    let mut dmtype = String::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        match attr.key {
          b"dmid" => dmid.push_str(value),
          b"dmrole" => dmrole.push_str(value),
          b"dmtype" => dmtype.push_str(value),
          _ => {
            return Err(VOTableError::UnexpectedAttr(
              attr.key.to_vec(),
              "INSTANCE child of INSTANCE child of TEMPLATES",
            ))
          }
        }
      }
    }
    if dmtype.is_empty() || dmrole.is_empty() {
      Err(VOTableError::Custom(format!(
        "Attributes 'dmtype' and 'dmrole' are mandatory and must be non-empty in tag '{}' child of `INSTANCE`",
        Self::TAG
      )))
    } else {
      let mut elem = Self::new(dmrole, dmtype);
      if !dmid.is_empty() {
        elem = elem.set_dmid(dmid);
      }
      Ok(elem)
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
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Instance::TAG_BYTES => self
            .elems
            .push(InstanceElem::Instance(from_event_start_by_ref!(
              Instance,
              reader,
              reader_buff,
              e
            ))),
          Reference::TAG_BYTES => {
            self
              .elems
              .push(InstanceElem::Reference(from_event_start_by_ref!(
                Reference,
                reader,
                reader_buff,
                e
              )))
          }
          Collection::TAG_BYTES => {
            self
              .elems
              .push(InstanceElem::Collection(from_event_start_by_ref!(
                Collection,
                reader,
                reader_buff,
                e
              )))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          PrimaryKey::TAG_BYTES => self.primarykeys.push(PrimaryKey::from_event_empty(e)?),
          Attribute::TAG_BYTES => self
            .elems
            .push(InstanceElem::Attribute(Attribute::from_event_empty(e)?)),
          Reference::TAG_BYTES => self
            .elems
            .push(InstanceElem::Reference(Reference::from_event_empty(e)?)),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
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
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, dmid);
    tag.push_attribute(("dmrole", self.dmrole.as_str()));
    tag.push_attribute(("dmtype", self.dmtype.as_str()));
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem_vec!(self, primarykeys, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}
