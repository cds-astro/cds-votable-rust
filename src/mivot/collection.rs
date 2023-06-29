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
use super::reference::{DynRef, StaticRef};
use super::{attribute_c::AttributePatC, join::Join, primarykey::PrimaryKey, ElemImpl, ElemType};

/*
    enum CollectionElem
    Description
    *    Enum of the elements that can be children of the mivot <COLLECTION> tag in any order.
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum CollectionElem {
    Attribute(AttributePatC),
    Instance(NoRoleInstance),
    StaticRef(StaticRef),
    DynRef(DynRef),
    Collection(CollectionPatC),
    Join(Join),
}
impl ElemType for CollectionElem {
    fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
        match self {
            CollectionElem::Attribute(elem) => elem.write(writer, &()),
            CollectionElem::Instance(elem) => elem.write(writer, &()),
            CollectionElem::StaticRef(elem) => elem.write(writer, &()),
            CollectionElem::DynRef(elem) => elem.write(writer, &()),
            CollectionElem::Collection(elem) => elem.write(writer, &()),
            CollectionElem::Join(elem) => elem.write(writer, &()),
        }
    }
  }

/////////////////////////
/////// PATTERN A ///////
/////////////////////////

/*
    struct Collection => pattern A valid in Instance
    @elem dmrole String: Modeled node related => MAND
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionPatA {
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
impl CollectionPatA {
    impl_non_empty_new!([dmrole], [dmid], [primary_keys, elems]);
    impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<CollectionElem> for CollectionPatA {
    fn push_to_elems(&mut self, elem: CollectionElem) {
        self.elems.push(elem)
    }
}
impl_quickrw_not_e!(
    [dmrole],       // MANDATORY ATTRIBUTES
    [dmid],         // OPTIONAL ATTRIBUTES
    "COLLECTION",   // TAG, here : <COLLECTION>
    CollectionPatA, // Struct on which to impl
    (),             // Context type
    [primary_keys],
    read_collection_sub_elem,
    [elems]
);

/////////////////////////
/////// PATTERN B ///////
/////////////////////////

/*
    struct Collection => pattern B valid in Globals
    @elem dmid String: Mapping element identification => MAND
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionPatB {
    // MANDATORY
    dmid: String,
    // OPTIONAL
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<CollectionElem>,
}
impl CollectionPatB {
    impl_non_empty_new!([dmid], [], [primary_keys, elems]);
}
impl ElemImpl<CollectionElem> for CollectionPatB {
    fn push_to_elems(&mut self, elem: CollectionElem) {
        self.elems.push(elem)
    }
}
impl_quickrw_not_e!(
    [dmid],         // MANDATORY ATTRIBUTES
    [],             // OPTIONAL ATTRIBUTES
    [dmrole],       // Potential empty attributes
    "COLLECTION",   // TAG, here : <COLLECTION>
    CollectionPatB, // Struct on which to impl
    (),             // Context type
    [primary_keys],
    read_collection_sub_elem,
    [elems]
);

/////////////////////////
/////// PATTERN C ///////
/////////////////////////

/*
    struct Collection => pattern C Valid in collections
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionPatC {
    // OPTIONAL
    #[serde(skip_serializing_if = "Option::is_none")]
    dmid: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<CollectionElem>,
}
impl CollectionPatC {
    impl_non_empty_new!([], [dmid], [primary_keys, elems]);
    impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<CollectionElem> for CollectionPatC {
    fn push_to_elems(&mut self, elem: CollectionElem) {
        self.elems.push(elem)
    }
}
impl_quickrw_not_e!(
    [],             // MANDATORY ATTRIBUTES
    [dmid],         // OPTIONAL ATTRIBUTES
    [dmrole],       //Potential empty attributes
    "COLLECTION",   // TAG, here : <COLLECTION>
    CollectionPatC, // Struct on which to impl
    (),             // Context type
    [primary_keys],
    read_collection_sub_elem,
    [elems]
);

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_collection_sub_elem
    Description:
    *   reads the children of Collection
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<CollectionElem>; a struct that implements the quickXMLReadWrite and ElemImpl for CollectionElem traits.
    @param instance &mut T: an instance of T (here CollectionPatA, CollectionPatB or CollectionPatC)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/

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
                DynRef::TAG_BYTES => {
                    if e.attributes()
                        .find(|attribute| attribute.as_ref().unwrap().key == "sourceref".as_bytes())
                        .is_some()
                    {
                        collection.push_to_elems(CollectionElem::DynRef(from_event_start!(
                            DynRef,
                            reader,
                            reader_buff,
                            e
                        )))
                    } else {
                        collection.push_to_elems(CollectionElem::StaticRef(from_event_start!(
                            StaticRef,
                            reader,
                            reader_buff,
                            e
                        )))
                    }
                }
                CollectionPatA::TAG_BYTES => collection.push_to_elems(CollectionElem::Collection(
                    from_event_start!(CollectionPatC, reader, reader_buff, e),
                )),
                Join::TAG_BYTES => collection.push_to_elems(CollectionElem::Join(
                    from_event_start!(Join, reader, reader_buff, e),
                )),
                _ => {
                    return Err(VOTableError::UnexpectedStartTag(
                        e.local_name().to_vec(),
                        CollectionPatA::TAG,
                    ))
                }
            },
            Event::Empty(ref e) => match e.local_name() {
                AttributePatC::TAG_BYTES => collection.push_to_elems(CollectionElem::Attribute(
                    AttributePatC::from_event_empty(e)?,
                )),
                DynRef::TAG_BYTES => {
                    if e.attributes()
                        .find(|attribute| attribute.as_ref().unwrap().key == "sourceref".as_bytes())
                        .is_some()
                    {
                        collection
                            .push_to_elems(CollectionElem::DynRef(DynRef::from_event_empty(e)?))
                    } else {
                        collection.push_to_elems(CollectionElem::StaticRef(
                            StaticRef::from_event_empty(e)?,
                        ))
                    }
                }
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
