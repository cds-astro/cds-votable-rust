use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{
    events::{BytesStart, Event},
    Writer,
};
use std::{io::Write, str};

use super::{
    attribute::Attribute, instance::Instance, join::Join, primarykey::PrimaryKey,
    reference::Reference,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum CollectionElem {
    Attribute(Attribute),
    Instance(Instance),
    Reference(Reference),
    Collection(Collection),
    Join(Join),
}
impl CollectionElem {
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Collection {
    #[serde(skip_serializing_if = "Option::is_none")]
    dmid: Option<String>,
    dmtype: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<CollectionElem>,
}
impl Collection {
    fn new<N: Into<String>>(dmtype: N) -> Self {
        Self {
            dmid: None,
            dmtype: dmtype.into(),
            primary_keys: vec![],
            elems: vec![],
        }
    }
    impl_builder_opt_string_attr!(dmid);
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
                b"dmtype" => {
                    collection.dmtype = value.to_string();
                    collection
                }
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if collection.dmtype.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attribute 'dmtype' is mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(collection)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        mut reader: quick_xml::Reader<R>,
        mut reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
        loop {
            let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
            match &mut event {
                Event::Start(ref e) => {
                    match e.local_name() {
                        Attribute::TAG_BYTES => self.elems.push(CollectionElem::Attribute(
                            from_event_start!(Attribute, reader, reader_buff, e),
                        )),
                        Instance::TAG_BYTES => self.elems.push(CollectionElem::Instance(
                            from_event_start!(Instance, reader, reader_buff, e),
                        )),
                        Reference::TAG_BYTES => self.elems.push(CollectionElem::Reference(
                            from_event_start!(Reference, reader, reader_buff, e),
                        )),
                        Collection::TAG_BYTES => {
                            self.elems
                                .push(CollectionElem::Collection(from_event_start!(
                                    Collection,
                                    reader,
                                    reader_buff,
                                    e
                                )))
                        }
                        Join::TAG_BYTES => self.elems.push(CollectionElem::Join(
                            from_event_start!(Join, reader, reader_buff, e),
                        )),
                        _ => {
                            return Err(VOTableError::UnexpectedStartTag(
                                e.local_name().to_vec(),
                                Self::TAG,
                            ))
                        }
                    }
                }
                Event::Empty(ref e) => match e.local_name() {
                    Attribute::TAG_BYTES => self
                        .elems
                        .push(CollectionElem::Attribute(Attribute::from_event_empty(e)?)),
                    Reference::TAG_BYTES => self
                        .elems
                        .push(CollectionElem::Reference(Reference::from_event_empty(e)?)),
                    _ => {
                        return Err(VOTableError::UnexpectedEmptyTag(
                            e.local_name().to_vec(),
                            Self::TAG,
                        ))
                    }
                },
                Event::Text(e) if is_empty(e) => {}
                Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
                Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
                _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
            }
        }
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
        context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
        //MANDATORY
        tag.push_attribute(("dmtype", self.dmtype.as_str()));
        //OPTIONAL
        push2write_opt_string_attr!(self, tag, dmid);
        writer
            .write_event(Event::Start(tag.to_borrowed()))
            .map_err(VOTableError::Write)?;
        write_elem_vec!(self, primary_keys, writer, context);
        write_elem_vec_no_context!(self, elems, writer);
        writer
            .write_event(Event::End(tag.to_end()))
            .map_err(VOTableError::Write)
    }
}
