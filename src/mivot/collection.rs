use crate::Attributes;
use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::Reader;
use quick_xml::{
  events::{BytesStart, Event},
  Writer,
};
use std::{io::Write, str};

use super::instance::NoRoleInstance;
use super::{
    attribute_c::AttributePatC, join::Join, primarykey::PrimaryKey,
    reference::Reference, ElemImpl, ElemType,
};

/*
    enum Collection Elem
    Description
    *    Enum of the elements that can be children of the mivot <COLLECTION> tag in any order.
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum CollectionElem {
    Attribute(AttributePatC),
    Instance(NoRoleInstance),
    Reference(Reference),
    Collection(Collection),
    Join(Join),
}
impl ElemType for CollectionElem {
    fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
        match self {
            CollectionElem::Attribute(elem) => elem.write(writer, &()),
            CollectionElem::Instance(elem) => elem.write(writer, &()),
            CollectionElem::Reference(elem) => elem.write(writer, &()),
            CollectionElem::Collection(elem) => elem.write(writer, &()),
            CollectionElem::Join(elem) => elem.write(writer, &()),
        }
    }
  }

/*
    struct Collection => pattern a
    @elem dmtype String: Modeled node related => MAND
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Collection {
  // MANDATORY
  dmrole: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  dmid: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  primary_keys: Vec<PrimaryKey>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<CollectionElem>,
}
impl Collection {
    impl_non_empty_new!([dmrole], [dmid], [primary_keys, elems]);
    impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<CollectionElem> for Collection {
  fn push_to_elems(&mut self, elem: CollectionElem) {
    self.elems.push(elem)
  }
}
impl_quickrw_not_e!(
    [dmrole],     // MANDATORY ATTRIBUTES
    [dmid],       // OPTIONAL ATTRIBUTES
    "COLLECTION", // TAG, here : <COLLECTION>
    Collection,   // Struct on which to impl
    (),           // Context type
    [primary_keys],
    read_collection_sub_elem,
    [elems]
);

///////////////////////
// UTILITY FUNCTIONS //

fn read_collection_sub_elem<
  R: std::io::BufRead,
  T: QuickXmlReadWrite + ElemImpl<CollectionElem>,
>(
    collection: &mut T,
    _context: &(),
    mut reader: quick_xml::Reader<R>,
    mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
    loop {
        let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
            Event::Start(ref e) => match e.local_name() {
                AttributePatC::TAG_BYTES => collection.push_to_elems(CollectionElem::Attribute(
                    from_event_start!(AttributePatC, reader, reader_buff, e),
                )),
                NoRoleInstance::TAG_BYTES => collection.push_to_elems(CollectionElem::Instance(
                    from_event_start!(NoRoleInstance, reader, reader_buff, e),
                )),
                Reference::TAG_BYTES => collection.push_to_elems(CollectionElem::Reference(
                    from_event_start!(Reference, reader, reader_buff, e),
                )),
                Collection::TAG_BYTES => collection.push_to_elems(CollectionElem::Collection(
                    from_event_start!(Collection, reader, reader_buff, e),
                )),
                Join::TAG_BYTES => collection.push_to_elems(CollectionElem::Join(
                    from_event_start!(Join, reader, reader_buff, e),
                )),
                _ => {
                    return Err(VOTableError::UnexpectedStartTag(
                        e.local_name().to_vec(),
                        Collection::TAG,
                    ))
                }
            },
            Event::Empty(ref e) => match e.local_name() {
                AttributePatC::TAG_BYTES => collection.push_to_elems(CollectionElem::Attribute(
                    AttributePatC::from_event_empty(e)?,
                )),
                Reference::TAG_BYTES => collection
                    .push_to_elems(CollectionElem::Reference(Reference::from_event_empty(e)?)),
                _ => {
                    return Err(VOTableError::UnexpectedEmptyTag(
                        e.local_name().to_vec(),
                        T::TAG,
                    ))
                }
            },
            Event::Text(e) if is_empty(e) => {}
            Event::End(e) if e.local_name() == T::TAG_BYTES => return Ok(reader),
            Event::Eof => return Err(VOTableError::PrematureEOF(T::TAG)),
            _ => eprintln!("Discarded event in {}: {:?}", T::TAG, event),
        }
      }
    }
