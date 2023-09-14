//! Here `INSTANCE` is a **child of** `COLLECTION` which is **child of** `GLOBALS`.
//! Hence:
//! * it has **no** `dmrole`
//! * it **must have** a (static) `PRIMARY_KEY`

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
  error::VOTableError, is_empty, mivot::attribute::AttributeChildOfInstance as Attribute,
  QuickXmlReadWrite,
};

use super::super::instance::{
  collection::Collection, instance::Instance as InstanceChildOfInstance,
  primary_key::PrimaryKeyStatic as PrimaryKey, reference::Reference, InstanceElem,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
  pub fn new<S: Into<String>>(dmtype: S, primarykey: PrimaryKey) -> Self {
    Self {
      dmid: None,
      dmtype: dmtype.into(),
      primarykeys: vec![primarykey],
      elems: Default::default(),
    }
  }

  /// Temporary because we must then ensure that `primarykeys` is not empty.
  pub(crate) fn new_tmp<S: Into<String>>(dmtype: S) -> Self {
    Self {
      dmid: None,
      dmtype: dmtype.into(),
      primarykeys: Default::default(),
      elems: Default::default(),
    }
  }

  impl_builder_opt_string_attr!(dmid);

  impl_builder_push!(PrimaryKey);

  impl_builder_push_elem!(Attribute, InstanceElem);
  impl_builder_push_elem!(Instance, InstanceElem, InstanceChildOfInstance);
  impl_builder_push_elem!(Reference, InstanceElem);
  impl_builder_push_elem!(Collection, InstanceElem);
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
              "INSTANCE child of COLLECTION child of GLOBALS",
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
      // TODO: replace this by a code similar to COLLECTION not to use a constructor with empty PRIMARY_KEY
      let mut elem = Self::new_tmp(dmtype);
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
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          return if self.primarykeys.is_empty() {
            Err(VOTableError::Custom(String::from(
              "`INSTANCE` child of `COLLECTION` must contains at leas ont `PRIMARY_KEY`",
            )))
          } else {
            Ok(())
          }
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
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