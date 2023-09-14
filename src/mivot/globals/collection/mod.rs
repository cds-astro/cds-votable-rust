//! Here `COLLECTION` is a **child of** `GLOBALS`.
//! Hence, it has **no** `dmrole`.

use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use crate::{error::VOTableError, is_empty, mivot::join::Join, QuickXmlReadWrite};

pub mod reference;
use reference::Reference;

pub mod instance;
use instance::Instance;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type", content = "content")]
enum CollectionElems {
  /// Reference here is **child of** `COLLECTION` in `GLOBALS`
  Reference(Vec<Reference>),
  Instance(Vec<Instance>),
  Join(Join),
}

impl CollectionElems {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      CollectionElems::Reference(elems) => {
        for elem in elems {
          elem.write(writer, &())?;
        }
        Ok(())
      }
      CollectionElems::Instance(elems) => {
        for elem in elems {
          elem.write(writer, &())?;
        }
        Ok(())
      }
      CollectionElems::Join(elem) => elem.write(writer, &()),
    }
  }
}

/// `COLLECTION` is a **child of** `GLOBALS` (no `dmrole`, like `COLLECTION` **child of** `COLLECTION`).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Collection {
  dmid: String,
  elems: CollectionElems,
}
impl Collection {
  pub fn from_references<S: Into<String>>(
    dmid: S,
    references: Vec<Reference>,
  ) -> Result<Self, VOTableError> {
    let dmid = dmid.into();
    if dmid.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty 'dmid' in collection",
      )))
    } else if references.is_empty() {
      Err(VOTableError::Custom(String::from(
        "Empty list of reference in collection",
      )))
    } else {
      Ok(Self {
        dmid,
        elems: CollectionElems::Reference(references),
      })
    }
  }

  pub fn from_instances<S: Into<String>>(
    dmid: S,
    instances: Vec<Instance>,
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
        elems: CollectionElems::Instance(instances),
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
}

pub(crate) fn get_dmid_from_atttributes(attrs: Attributes) -> Result<String, VOTableError> {
  let mut dmid = String::new();
  for attr_res in attrs {
    let attr = attr_res.map_err(VOTableError::Attr)?;
    let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
    let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
    if !value.is_empty() {
      match attr.key {
        b"dmid" => dmid.push_str(value),
        _ => {
          return Err(VOTableError::UnexpectedAttr(
            attr.key.to_vec(),
            Collection::TAG,
          ))
        }
      }
    };
  }
  Ok(dmid)
}

/// Special case since we check that the Collection contains attribute...
pub(crate) fn create_collection_from_dmid_and_reading_sub_elems<R: BufRead>(
  dmid: String,
  _context: &(),
  mut reader: &mut Reader<R>,
  mut reader_buff: &mut Vec<u8>,
) -> Result<Collection, VOTableError> {
  let mut ref_vec: Vec<Reference> = Default::default();
  let mut inst_vec: Vec<Instance> = Default::default();
  let mut join_vec: Vec<Join> = Default::default();

  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => match e.local_name() {
        Reference::TAG_BYTES => {
          ref_vec.push(from_event_start_by_ref!(Reference, reader, reader_buff, e))
        }
        Instance::TAG_BYTES => {
          inst_vec.push(from_event_start_by_ref!(Instance, reader, reader_buff, e))
        }
        Join::TAG_BYTES => join_vec.push(from_event_start_by_ref!(Join, reader, reader_buff, e)),
        _ => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            Collection::TAG,
          ))
        }
      },
      Event::Empty(ref e) => match e.local_name() {
        Reference::TAG_BYTES => ref_vec.push(Reference::from_event_empty(e)?),
        Instance::TAG_BYTES => inst_vec.push(Instance::from_event_empty(e)?),
        Join::TAG_BYTES => join_vec.push(Join::from_event_empty(e)?),
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            Collection::TAG,
          ))
        }
      },
      Event::Text(e) if is_empty(e) => {}
      Event::End(e) if e.local_name() == Collection::TAG_BYTES => {
        return match (((!ref_vec.is_empty()) as u8) << 2)
          + (((!inst_vec.is_empty()) as u8) << 1)
          + ((!join_vec.is_empty()) as u8)
        {
          4 => Collection::from_references(dmid, ref_vec),
          2 => Collection::from_instances(dmid, inst_vec),
          1 if join_vec.len() == 1 => {
            Collection::from_join(dmid, join_vec.drain(..).next().unwrap())
          }
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
      Event::Eof => return Err(VOTableError::PrematureEOF(Collection::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", Collection::TAG, event),
    }
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
    tag.push_attribute(("name", self.dmid.as_str()));
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
