//! Here `INSTANCE` is a **child of** `INSTANCE`.
//! Hence, it **must have** `dmrole`.

use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{attribute::AttributeChildOfInstance as Attribute, VodmlVisitor},
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_err},
  HasSubElements, HasSubElems, QuickXmlReadWrite, VOTableElement,
};

use super::{
  collection::Collection, primary_key::PrimaryKeyStatic as PrimaryKey, reference::Reference,
  InstanceElem,
};

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
  pub fn new<S: Into<String>>(dmrole: S, dmtype: S) -> Self {
    Self {
      dmid: None,
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      primarykeys: Default::default(),
      elems: Default::default(),
    }
  }

  impl_builder_opt_string_attr!(dmid);
  impl_builder_mandatory_string_attr!(dmrole);
  impl_builder_mandatory_string_attr!(dmtype);

  impl_builder_push!(PrimaryKey);

  impl_builder_push_elem!(Attribute, InstanceElem);
  impl_builder_push_elem!(Instance, InstanceElem);
  impl_builder_push_elem!(Reference, InstanceElem);
  impl_builder_push_elem!(Collection, InstanceElem);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_instance_childof_instance_in_globals_start(self)?;
    for pk in self.primarykeys.iter_mut() {
      pk.visit(visitor)?;
    }
    for elem in self.elems.iter_mut() {
      elem.visit(visitor)?;
    }
    visitor.visit_instance_childof_instance_in_globals_ended(self)
  }
}

impl VOTableElement for Instance {
  const TAG: &'static str = "INSTANCE";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("", "")
      .set_attrs(attrs)
      .and_then(|instance| {
        if instance.dmrole.is_empty() || instance.dmtype.is_empty()  {
          Err(VOTableError::Custom(format!(
            "Attributes 'dmrole' and 'dmtype' are mandatory and must be non-empty in tag '{}' child of INSTANCE child of GLOBALS",
            Self::TAG
          )))
        } else {
          Ok(instance)
        }
      })
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "dmid" => self.set_dmid_by_ref(val),
        "dmrole" => self.set_dmrole_by_ref(val),
        "dmtype" => self.set_dmtype_by_ref(val),
        _ => {
          return Err(unexpected_attr_err(
            key,
            "INSTANCE child of INSTANCE child of GLOBALS",
          ))
        }
      }
    }
    Ok(())
  }

  /// Calls a closure on each (key, value) attribute pairs.
  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(dmid) = &self.dmid {
      f("dmid", dmid.as_str());
    }
    f("dmrole", self.dmrole.as_str());
    f("dmtype", self.dmtype.as_str());
  }
}

impl HasSubElements for Instance {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    false
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
          Instance::TAG_BYTES => push_from_event_start!(self, Instance, reader, reader_buff, e),
          Reference::TAG_BYTES => push_from_event_start!(self, Reference, reader, reader_buff, e),
          Collection::TAG_BYTES => push_from_event_start!(self, Collection, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          PrimaryKey::TAG_BYTES => push_from_event_empty!(self, PrimaryKey, e),
          Attribute::TAG_BYTES => push_from_event_empty!(self, Attribute, e),
          Reference::TAG_BYTES => push_from_event_empty!(self, Reference, e),
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

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec!(self, primarykeys, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    Ok(())
  }
}
