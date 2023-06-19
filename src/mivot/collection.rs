use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{
    events::{BytesStart, Event},
    Writer,
};
use std::{io::Write, str};

use super::{
    attribute_c::AttributePatC,
    instance::{Instance, InstanceContexts},
    join::Join,
    primarykey::PrimaryKey,
    reference::Reference,
    ElemImpl, ElemType,
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
    Instance(Instance),
    Reference(Reference),
    Collection(Collection),
    Join(Join),
}
impl ElemType for CollectionElem {
    fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
        match self {
            CollectionElem::Attribute(elem) => elem.write(writer, &()),
            CollectionElem::Instance(elem) => elem.write(writer, &InstanceContexts::Writing),
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
    fn new<N: Into<String>>(dmrole: N) -> Self {
        Self {
            // MANDATORY
            dmrole: dmrole.into(),
            // OPTIONAL
            dmid: None,
            primary_keys: vec![],
            elems: vec![],
        }
    }
    impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<CollectionElem> for Collection {
    fn push_to_elems(&mut self, elem: CollectionElem) {
        self.elems.push(elem)
    }
}
impl QuickXmlReadWrite for Collection {
    const TAG: &'static str = "COLLECTION";
    type Context = ();

    fn from_attributes(
        attrs: quick_xml::events::attributes::Attributes,
    ) -> Result<Self, crate::error::VOTableError> {
        const NULL: &str = "@TBD";
        let mut collection = Self::new(NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            collection = match attr.key {
                b"dmid" => collection.set_dmid(value),
                b"dmrole" => {
                    collection.dmrole = value.to_string();
                    collection
                }
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if collection.dmrole.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attribute 'dmrole' is mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(collection)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        reader: quick_xml::Reader<R>,
        reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
        read_collection_sub_elem(self, reader, reader_buff)
    }

    fn read_sub_elements_by_ref<R: std::io::BufRead>(
        &mut self,
        _reader: &mut quick_xml::Reader<R>,
        _reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        todo!()
    }

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut quick_xml::Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
        //MANDATORY
        tag.push_attribute(("dmrole", self.dmrole.as_str()));
        write_non_empty_mandatory_attributes!(tag, self, dmrole);
        //OPTIONAL4
        write_non_empty_optional_attributes!(tag, self, dmid);
        writer
            .write_event(Event::Start(tag.to_borrowed()))
            .map_err(VOTableError::Write)?;
        write_elem_vec_empty_context!(self, primary_keys, writer);
        write_elem_vec_no_context!(self, elems, writer);
        writer
            .write_event(Event::End(tag.to_end()))
            .map_err(VOTableError::Write)
    }
}

fn read_collection_sub_elem<
    R: std::io::BufRead,
    T: QuickXmlReadWrite + ElemImpl<CollectionElem>,
>(
    collection: &mut T,
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
                Instance::TAG_BYTES => collection.push_to_elems(CollectionElem::Instance(
                    from_event_start!(Instance, reader, reader_buff, e, InstanceContexts::C),
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
