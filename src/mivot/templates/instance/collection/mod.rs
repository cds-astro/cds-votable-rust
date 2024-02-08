//! Here `COLLECTION` is a **child of** `INSTANCE` in `TEMPLATES`, hence:
//! * **must have** a `dmrole`

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
  mivot::{
    attribute::AttributeChildOfCollection as Attribute, join::Join, templates::instance::Instance,
    VodmlVisitor,
  },
  utils::{discard_comment, discard_event, is_empty},
  QuickXmlReadWrite,
};

pub mod collection;
use collection::{Collection as CollectionChildOfCollection, CollectionElems, InstanceOrRef};
pub mod reference;
use reference::Reference;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Collection {
  pub dmrole: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dmid: Option<String>,
  pub elems: CollectionElems,
}

impl Collection {
  pub fn from_attribute<S: Into<String>>(
    dmrole: S,
    attributes: Vec<Attribute>,
  ) -> Result<Self, VOTableError> {
    let dmrole = dmrole.into();
    if dmrole.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmrole' in collection",
      )))
    } else if attributes.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of attribute in collection",
      )))
    } else {
      Ok(Self {
        dmrole,
        dmid: None,
        elems: CollectionElems::Attribute(attributes),
      })
    }
  }

  pub fn from_collections<S: Into<String>>(
    dmrole: S,
    collections: Vec<CollectionChildOfCollection>,
  ) -> Result<Self, VOTableError> {
    let dmrole = dmrole.into();
    if dmrole.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmrole' in collection",
      )))
    } else if collections.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of collection in collection",
      )))
    } else {
      Ok(Self {
        dmrole,
        dmid: None,
        elems: CollectionElems::Collection(collections),
      })
    }
  }

  pub fn from_instances<S: Into<String>>(
    dmrole: S,
    mut instances: Vec<Instance>,
  ) -> Result<Self, VOTableError> {
    let dmrole = dmrole.into();
    if dmrole.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmrole' in collection",
      )))
    } else if instances.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of instance in collection",
      )))
    } else {
      Ok(Self {
        dmrole,
        dmid: None,
        elems: CollectionElems::InstanceOrRef(
          instances.drain(..).map(InstanceOrRef::Instance).collect(),
        ),
      })
    }
  }

  pub fn from_instance_or_reference_elems<S: Into<String>>(
    dmrole: S,
    instance_or_reference_elems: Vec<InstanceOrRef>,
  ) -> Result<Self, VOTableError> {
    let dmrole = dmrole.into();
    if dmrole.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmrole' in collection",
      )))
    } else if instance_or_reference_elems.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of instance/reference in collection",
      )))
    } else {
      Ok(Self {
        dmrole,
        dmid: None,
        elems: CollectionElems::InstanceOrRef(instance_or_reference_elems),
      })
    }
  }

  pub fn from_join<S: Into<String>>(dmrole: S, join: Join) -> Result<Self, VOTableError> {
    let dmrole = dmrole.into();
    if dmrole.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmrole' in collection",
      )))
    } else {
      Ok(Self {
        dmrole,
        dmid: None,
        elems: CollectionElems::Join(join),
      })
    }
  }

  impl_builder_opt_string_attr!(dmid);

  pub(crate) fn get_dmrole_opt_dmid_from_atttributes(
    attrs: Attributes,
  ) -> Result<(String, Option<String>), VOTableError> {
    let mut dmrole = String::new();
    let mut dmid: Option<String> = None;
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        match attr.key {
          b"dmrole" => dmrole.push_str(value),
          b"dmid" => dmid = Some(value.into()),
          _ => return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)),
        }
      };
    }
    Ok((dmrole, dmid))
  }

  /// Special case since we check that the Collection contains attribute...
  pub(crate) fn from_dmrole_and_reading_sub_elems<R: BufRead>(
    dmrole: String,
    dmid: Option<String>,
    _context: &(),
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Self, VOTableError> {
    let mut attr_vec: Vec<Attribute> = Default::default();
    let mut col_vec: Vec<CollectionChildOfCollection> = Default::default();
    let mut inst_or_ref_vec: Vec<InstanceOrRef> = Default::default();
    let mut join_vec: Vec<Join> = Default::default();

    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Attribute::TAG_BYTES => {
            attr_vec.push(from_event_start_by_ref!(Attribute, reader, reader_buff, e))
          }
          Reference::TAG_BYTES => inst_or_ref_vec.push(InstanceOrRef::Reference(
            from_event_start_by_ref!(Reference, reader, reader_buff, e),
          )),
          Self::TAG_BYTES => {
            let opt_dmid =
              CollectionChildOfCollection::get_opt_dmid_from_atttributes(e.attributes())?;
            let col = CollectionChildOfCollection::from_opt_dmid_and_reading_sub_elems(
              opt_dmid,
              &(),
              reader,
              reader_buff,
            )?;
            col_vec.push(col)
          }
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
          Attribute::TAG_BYTES => attr_vec.push(Attribute::from_event_empty(e)?),
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
          return match (((!attr_vec.is_empty()) as u8) << 3)
            + (((!col_vec.is_empty()) as u8) << 2)
            + (((!inst_or_ref_vec.is_empty()) as u8) << 1)
            + ((!join_vec.is_empty()) as u8)
          {
            8 => Self::from_attribute(dmrole, attr_vec),
            4 => Self::from_collections(dmrole, col_vec),
            2 => Self::from_instance_or_reference_elems(dmrole, inst_or_ref_vec),
            1 if join_vec.len() == 1 => Self::from_join(dmrole, join_vec.drain(..).next().unwrap()),
            1 => Err(VOTableError::Custom(
              "A collection cannot have more than one join".to_owned(),
            )),
            0 => Err(VOTableError::Custom(
              "In COLLECTION child of INSTANCE child of TEMPLATES: must have at least one item"
                .to_owned(),
            )),
            _ => Err(VOTableError::Custom(
              "A collection cannot have items of different types".to_owned(),
            )),
          } // Set dmid if any
          .map(|c| {
            if let Some(dmid) = dmid {
              c.set_dmid(dmid)
            } else {
              c
            }
          });
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_collection_childof_instance_in_templates(self)?;
    self.elems.visit(visitor)
  }
}

impl QuickXmlReadWrite for Collection {
  const TAG: &'static str = "COLLECTION";
  type Context = ();

  fn from_attributes(_attrs: Attributes) -> Result<Self, VOTableError> {
    Err(VOTableError::Custom(format!(
      "Tag {} cannot be built directly from attributes",
      Self::TAG
    )))
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
    mut _reader: &mut Reader<R>,
    mut _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(format!(
      "Tag {} cannot be built before reading sub-elements",
      Self::TAG
    )))
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    tag.push_attribute(("dmrole", self.dmrole.as_str()));
    push2write_opt_string_attr!(self, tag, dmid);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    self.elems.write(writer)?;
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}
