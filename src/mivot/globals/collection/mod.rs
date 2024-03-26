//! Here `COLLECTION` is a **child of** `GLOBALS`.
//! Hence, it has **no** `dmrole`.

use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  mivot::{join::Join, VodmlVisitor},
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_err},
  HasSubElements, HasSubElems, QuickXmlReadWrite, VOTableElement,
};

pub mod reference;
use reference::Reference;

pub mod instance;
use instance::Instance;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum InstanceOrRef {
  Instance(Instance),
  /// Reference here is **child of** `COLLECTION` in `GLOBALS`
  Reference(Reference),
}
impl InstanceOrRef {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      InstanceOrRef::Instance(elem) => elem.write(writer, &()),
      InstanceOrRef::Reference(elem) => elem.write(writer, &()),
    }
  }

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    match self {
      InstanceOrRef::Instance(elem) => elem.visit(visitor),
      InstanceOrRef::Reference(elem) => elem.visit(visitor),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type", content = "content")]
pub enum CollectionElems {
  InstanceOrRef(Vec<InstanceOrRef>),
  Join(Join),
}

impl CollectionElems {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      CollectionElems::InstanceOrRef(elems) => {
        for elem in elems {
          elem.write(writer)?;
        }
        Ok(())
      }
      CollectionElems::Join(elem) => elem.write(writer, &()),
    }
  }

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    match self {
      CollectionElems::InstanceOrRef(elems) => {
        for elem in elems.iter_mut() {
          elem.visit(visitor)?;
        }
      }
      CollectionElems::Join(join) => join.visit(visitor)?,
    };
    Ok(())
  }
}

/// `COLLECTION` is a **child of** `GLOBALS` (no `dmrole`, like `COLLECTION` **child of** `COLLECTION`).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Collection {
  pub dmid: String,
  pub elems: CollectionElems,
}
impl Collection {
  pub fn from_instances<S: Into<String>>(
    dmid: S,
    mut instances: Vec<Instance>,
  ) -> Result<Self, VOTableError> {
    let dmid = dmid.into();
    if dmid.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmid' in collection",
      )))
    } else if instances.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of instance in collection",
      )))
    } else {
      Ok(Self {
        dmid,
        elems: CollectionElems::InstanceOrRef(
          instances.drain(..).map(InstanceOrRef::Instance).collect(),
        ),
      })
    }
  }

  pub fn from_instance_or_reference_elems<S: Into<String>>(
    dmid: S,
    instance_or_reference_elems: Vec<InstanceOrRef>,
  ) -> Result<Self, VOTableError> {
    let dmid = dmid.into();
    if dmid.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmid' in collection",
      )))
    } else if instance_or_reference_elems.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of instance/reference in collection",
      )))
    } else {
      Ok(Self {
        dmid,
        elems: CollectionElems::InstanceOrRef(instance_or_reference_elems),
      })
    }
  }

  pub fn from_join<S: Into<String>>(dmid: S, join: Join) -> Result<Self, VOTableError> {
    let dmid = dmid.into();
    if dmid.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmid' in collection",
      )))
    } else {
      Ok(Self {
        dmid,
        elems: CollectionElems::Join(join),
      })
    }
  }

  /// Special case since we check that the Collection contains attribute...
  pub(crate) fn from_dmid_and_reading_sub_elems<R: BufRead>(
    dmid: String,
    _context: &(),
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Self, VOTableError> {
    let mut inst_or_ref_vec: Vec<InstanceOrRef> = Default::default();
    let mut join_vec: Vec<Join> = Default::default();

    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Reference::TAG_BYTES => inst_or_ref_vec.push(InstanceOrRef::Reference(
            from_event_start_by_ref!(Reference, reader, reader_buff, e),
          )),
          Instance::TAG_BYTES => inst_or_ref_vec.push(InstanceOrRef::Instance(
            from_event_start_by_ref!(Instance, reader, reader_buff, e),
          )),
          Join::TAG_BYTES => join_vec.push(from_event_start_by_ref!(Join, reader, reader_buff, e)),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Reference::TAG_BYTES => {
            inst_or_ref_vec.push(InstanceOrRef::Reference(Reference::from_event_empty(e)?))
          }
          Instance::TAG_BYTES => {
            inst_or_ref_vec.push(InstanceOrRef::Instance(Instance::from_event_empty(e)?))
          }
          Join::TAG_BYTES => join_vec.push(Join::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          return match (((!inst_or_ref_vec.is_empty()) as u8) << 1) + ((!join_vec.is_empty()) as u8)
          {
            2 => Self::from_instance_or_reference_elems(dmid, inst_or_ref_vec),
            1 if join_vec.len() == 1 => Self::from_join(dmid, join_vec.drain(..).next().unwrap()),
            1 => Err(VOTableError::Custom(
              "A collection cannot have more than one join".to_owned(),
            )),
            0 => Err(VOTableError::Custom(
              "In COLLECTION child of GLOBALS: must have at least one item".to_owned(),
            )),
            _ => Err(VOTableError::Custom(
              "A collection cannot have items of different types".to_owned(),
            )),
          };
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  impl_builder_mandatory_string_attr!(dmid);

  pub(crate) fn get_dmid_from_atttributes(attrs: Attributes) -> Result<String, VOTableError> {
    let mut dmid = String::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        match attr.key {
          b"dmid" => dmid.push_str(value),
          _ => return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)),
        }
      };
    }
    Ok(dmid)
  }

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor
      .visit_collection_childof_globals_start(self)
      .and_then(|()| self.elems.visit(visitor))
      .and_then(|()| visitor.visit_collection_childof_globals_ended(self))
  }
}

impl VOTableElement for Collection {
  const TAG: &'static str = "COLLECTION";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(_attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Err(VOTableError::Custom(format!(
      "Tag {} cannot be built directly from attributes",
      Self::TAG
    )))
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
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("dmid", self.dmid.as_str());
  }
}

impl HasSubElements for Collection {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    false
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(format!(
      "Tag {} cannot be built before reading sub-elements",
      Self::TAG
    )))
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self.elems.write(writer)
  }
}
