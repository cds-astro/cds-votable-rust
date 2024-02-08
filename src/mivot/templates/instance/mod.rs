//! Here `INSTANCE` is a **child of** `TEMPLATES`.
//! Hence, it has **no** `dmrole`.

use std::{
  io::{BufRead, Write},
  str,
};

use log::debug;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  mivot::{attribute::AttributeChildOfInstance as Attribute, VodmlVisitor},
  utils::is_empty,
  QuickXmlReadWrite,
};

pub mod collection;
use collection::Collection;
pub mod instance;
use instance::Instance as InstanceChildOfInstance;
pub mod primary_key;
use primary_key::PrimaryKeyDyn as PrimaryKey;
pub mod reference;
use reference::Reference;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum InstanceElem {
  Attribute(Attribute),
  Instance(InstanceChildOfInstance),
  Reference(Reference),
  Collection(Collection),
}
impl InstanceElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      InstanceElem::Attribute(elem) => elem.write(writer, &()),
      InstanceElem::Instance(elem) => elem.write(writer, &()),
      InstanceElem::Reference(elem) => elem.write(writer, &()),
      InstanceElem::Collection(elem) => elem.write(writer, &()),
    }
  }

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    match self {
      InstanceElem::Attribute(elem) => elem.visit(visitor),
      InstanceElem::Instance(elem) => elem.visit(visitor),
      InstanceElem::Reference(elem) => elem.visit(visitor),
      InstanceElem::Collection(elem) => elem.visit(visitor),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instance {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dmid: Option<String>,
  pub dmtype: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub primarykeys: Vec<PrimaryKey>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<InstanceElem>,
}

impl Instance {
  impl_new!([dmtype], [dmid], [primarykeys, elems]);

  impl_builder_opt_string_attr!(dmid);

  impl_builder_push!(PrimaryKey);

  impl_builder_push_elem!(Attribute, InstanceElem);
  impl_builder_push_elem!(Instance, InstanceElem, InstanceChildOfInstance);
  impl_builder_push_elem!(Reference, InstanceElem);
  impl_builder_push_elem!(Collection, InstanceElem);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_instance_childof_templates(self)?;
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
    let mut dmtype = String::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        match attr.key {
          b"dmid" => dmid.push_str(value),
          b"dmtype" => dmtype.push_str(value),
          _ => {
            return Err(VOTableError::UnexpectedAttr(
              attr.key.to_vec(),
              "INSTANCE child of TEMPLATES",
            ))
          }
        }
      }
    }
    if dmtype.is_empty() {
      Err(VOTableError::Custom(format!(
        "Attribute 'dmtype' is mandatory and must be non-empty in tag '{}' child of GLOBALS",
        Self::TAG
      )))
    } else {
      let mut elem = Self::new(dmtype);
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
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          InstanceChildOfInstance::TAG_BYTES => {
            self
              .elems
              .push(InstanceElem::Instance(from_event_start_by_ref!(
                InstanceChildOfInstance,
                reader,
                reader_buff,
                e
              )))
          }
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
            let (dmrole, dmid_opt) =
              Collection::get_dmrole_opt_dmid_from_atttributes(e.attributes())?;
            let collection = Collection::from_dmrole_and_reading_sub_elems(
              dmrole,
              dmid_opt,
              context,
              reader,
              reader_buff,
            )?;
            self.elems.push(InstanceElem::Collection(collection))
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
        _ => debug!("Discarded event in {}: {:?}", Self::TAG, event),
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
