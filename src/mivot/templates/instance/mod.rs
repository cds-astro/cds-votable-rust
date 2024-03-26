//! Here `INSTANCE` is a **child of** `TEMPLATES`.
//! Hence, it has **no** `dmrole`.

use std::{
  io::{BufRead, Write},
  str,
};

use log::debug;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{attribute::AttributeChildOfInstance as Attribute, VodmlVisitor},
  utils::{is_empty, unexpected_attr_err},
  HasSubElements, HasSubElems, QuickXmlReadWrite, VOTableElement,
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
  pub fn new<S: Into<String>>(dmtype: S) -> Self {
    Self {
      dmid: None,
      dmtype: dmtype.into(),
      primarykeys: Default::default(),
      elems: Default::default(),
    }
  }

  impl_builder_opt_string_attr!(dmid);
  impl_builder_mandatory_string_attr!(dmtype);

  impl_builder_push!(PrimaryKey);

  impl_builder_push_elem!(Attribute, InstanceElem);
  impl_builder_push_elem!(Instance, InstanceElem, InstanceChildOfInstance);
  impl_builder_push_elem!(Reference, InstanceElem);
  impl_builder_push_elem!(Collection, InstanceElem);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_instance_childof_templates_start(self)?;
    for pk in self.primarykeys.iter_mut() {
      pk.visit(visitor)?;
    }
    for elem in self.elems.iter_mut() {
      elem.visit(visitor)?;
    }
    visitor.visit_instance_childof_templates_ended(self)
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
    Self::new("").set_attrs(attrs).and_then(|instance| {
      if instance.dmtype.is_empty() {
        Err(VOTableError::Custom(format!(
          "Attribute 'dmtype' is mandatory and must be non-empty in tag '{}' child of TEMPLATES",
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
        "dmtype" => self.set_dmtype_by_ref(val),
        _ => {
          if !val.as_ref().is_empty() {
            return Err(unexpected_attr_err(key, "INSTANCE child of TEMPLATES"));
          }
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
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          InstanceChildOfInstance::TAG_BYTES => self.push_instance_by_ref(
            from_event_start_by_ref!(InstanceChildOfInstance, reader, reader_buff, e),
          ),
          Reference::TAG_BYTES => {
            self.push_reference_by_ref(from_event_start_by_ref!(Reference, reader, reader_buff, e))
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
            self.push_collection_by_ref(collection)
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          PrimaryKey::TAG_BYTES => self.push_primarykey_by_ref(PrimaryKey::from_event_empty(e)?),
          Attribute::TAG_BYTES => self.push_attribute_by_ref(Attribute::from_event_empty(e)?),
          Reference::TAG_BYTES => self.push_reference_by_ref(Reference::from_event_empty(e)?),
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
