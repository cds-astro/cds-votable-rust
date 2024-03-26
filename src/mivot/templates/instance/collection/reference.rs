use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::{
    globals::collection::reference::Reference as ReferenceStatic,
    templates::instance::reference::foreign_key::ForeignKey, VodmlVisitor,
  },
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_err},
  HasSubElements, HasSubElems, QuickXmlReadWrite, SpecialElem, VOTableElement,
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

impl VOTableElement for Reference {
  const TAG: &'static str = "REFERENCE";

  type MarkerType = SpecialElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut dmref = String::new();
    let mut sourceref = String::new();
    for (key, val) in attrs {
      let key = key.as_ref();
      let val = val.as_ref();
      if !val.is_empty() {
        match key {
          "dmref" => dmref.push_str(val),
          "sourceref" => sourceref.push_str(val),
          _ => return Err(unexpected_attr_err(key, Self::TAG)),
        }
      }
    }
    match (dmref.as_str(), sourceref.as_str()) {
      (dmref, "") => Ok(Reference::Static(ReferenceStatic::new(dmref))),
      ("", sourceref) => Ok(Reference::Dynamic(ReferenceDyn::new(sourceref))),
      _ =>
        Err(VOTableError::Custom(format!(
          "Either attribute 'dmref' or 'sourceref', not both, are accepted in tag '{}' child of COLLECTION",
          Self::TAG
        ))),
    }
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    match self {
      Reference::Static(e) => e.set_attrs_by_ref(attrs),
      Reference::Dynamic(e) => e.set_attrs_by_ref(attrs),
    }
  }

  fn for_each_attribute<F>(&self, f: F)
  where
    F: FnMut(&str, &str),
  {
    match self {
      Reference::Static(e) => e.for_each_attribute(f),
      Reference::Dynamic(e) => e.for_each_attribute(f),
    }
  }
}

impl QuickXmlReadWrite<SpecialElem> for Reference {
  type Context = ();

  fn read_content_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    match self {
      Reference::Static(static_ref_mut) => {
        static_ref_mut.read_content_by_ref(reader, reader_buff, context)
      }
      Reference::Dynamic(dyn_ref_mut) => {
        dyn_ref_mut.read_content_by_ref(reader, reader_buff, context)
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
  pub fn new<S: Into<String>>(sourceref: S) -> Self {
    Self {
      sourceref: sourceref.into(),
      foreignkeys: Default::default(),
    }
  }

  impl_builder_mandatory_string_attr!(sourceref);

  impl_builder_push!(ForeignKey);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_reference_dynamic_childof_collection_in_templates_start(self)?;
    for fk in self.foreignkeys.iter_mut() {
      fk.visit(visitor)?;
    }
    visitor.visit_reference_dynamic_childof_collection_in_templates_ended(self)
  }
}

impl VOTableElement for ReferenceDyn {
  const TAG: &'static str = "REFERENCE";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("").set_attrs(attrs).and_then(|reference| {
      if reference.sourceref.is_empty() {
        Err(VOTableError::Custom(format!(
          "Attribute 'sourceref' is' mandatory and must be non-empty in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(reference)
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
        "sourceref" => self.set_sourceref_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("sourceref", self.sourceref.as_str());
  }
}

impl HasSubElements for ReferenceDyn {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.foreignkeys.is_empty()
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

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec!(self, foreignkeys, writer, context);
    Ok(())
  }
}
