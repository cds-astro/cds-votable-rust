use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{
    events::{BytesStart, Event},
    Writer,
};
use std::{io::Write, str};

use super::{
    attribute::Attribute, collection::Collection, primarykey::PrimaryKey, reference::Reference,
    ElemImpl, ElemType,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum InstanceElem {
    Attribute(Attribute),
    Instance(Instance),
    Reference(Reference),
    Collection(Collection),
}
impl ElemType for InstanceElem {
    fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
        match self {
            InstanceElem::Attribute(elem) => elem.write(writer, &()),
            InstanceElem::Instance(elem) => elem.write(writer, &()),
            InstanceElem::Reference(elem) => elem.write(writer, &()),
            InstanceElem::Collection(elem) => elem.write(writer, &()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobOrTempInstance {
    #[serde(skip_serializing_if = "Option::is_none")]
    dmid: Option<String>,
    dmtype: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<InstanceElem>,
}
impl GlobOrTempInstance {
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
impl ElemImpl<InstanceElem> for GlobOrTempInstance {
    fn push_to_elems(&mut self, elem: InstanceElem) {
        self.elems.push(elem)
    }
}
impl QuickXmlReadWrite for GlobOrTempInstance {
    const TAG: &'static str = "INSTANCE";
    type Context = ();

    fn from_attributes(
        attrs: quick_xml::events::attributes::Attributes,
    ) -> Result<Self, crate::error::VOTableError> {
        const NULL: &str = "@TBD";
        let mut instance = Self::new(NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            instance = match attr.key {
                b"dmid" => instance.set_dmid(value),
                b"dmtype" => {
                    instance.dmtype = value.to_string();
                    instance
                }
                b"dmrole" => instance, // * This is in case of an empty dmrole which shouldn't be taken into account
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if instance.dmtype.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attribute 'dmtype' is mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(instance)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        reader: quick_xml::Reader<R>,
        reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
        read_instance_sub_elem(self, reader, reader_buff)
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
        tag.push_attribute(("dmtype", self.dmtype.as_str()));
        //OPTIONAL
        push2write_opt_string_attr!(self, tag, dmid);
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

//////////////////////////////////////

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Instance {
    //Mandatory
    dmtype: String,
    dmrole: String,
    //Optional
    #[serde(skip_serializing_if = "Option::is_none")]
    dmid: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<InstanceElem>,
}
impl Instance {
    fn new<N: Into<String>>(dmtype: N, dmrole: N) -> Self {
        Self {
            //Mandatory
            dmtype: dmtype.into(),
            dmrole: dmrole.into(),
            //Optional
            dmid: None,
            primary_keys: vec![],
            elems: vec![],
        }
    }
    impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<InstanceElem> for Instance {
    fn push_to_elems(&mut self, elem: InstanceElem) {
        self.elems.push(elem)
    }
}
impl QuickXmlReadWrite for Instance {
    const TAG: &'static str = "INSTANCE";
    type Context = ();

    fn from_attributes(
        attrs: quick_xml::events::attributes::Attributes,
    ) -> Result<Self, crate::error::VOTableError> {
        const NULL: &str = "@TBD";
        let mut instance = Self::new(NULL, NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            instance = match attr.key {
                b"dmid" => instance.set_dmid(value),
                b"dmrole" => {
                    instance.dmrole = value.to_string();
                    instance
                }
                b"dmtype" => {
                    instance.dmtype = value.to_string();
                    instance
                }
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if instance.dmtype.as_str() == NULL || instance.dmrole.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attributes 'dmtype' and 'dmrole' are mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(instance)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        reader: quick_xml::Reader<R>,
        reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
        read_instance_sub_elem(self, reader, reader_buff)
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
        tag.push_attribute(("dmtype", self.dmtype.as_str()));
        tag.push_attribute(("dmrole", self.dmrole.as_str()));
        //OPTIONAL
        push2write_opt_string_attr!(self, tag, dmid);
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

fn read_instance_sub_elem<R: std::io::BufRead, T: QuickXmlReadWrite + ElemImpl<InstanceElem>>(
    collection: &mut T,
    mut reader: quick_xml::Reader<R>,
    mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
    loop {
        let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
            Event::Start(ref e) => match e.local_name() {
                Attribute::TAG_BYTES => collection.push_to_elems(InstanceElem::Attribute(
                    from_event_start!(Attribute, reader, reader_buff, e),
                )),
                Instance::TAG_BYTES => collection.push_to_elems(InstanceElem::Instance(
                    from_event_start!(Instance, reader, reader_buff, e),
                )),
                Reference::TAG_BYTES => collection.push_to_elems(InstanceElem::Reference(
                    from_event_start!(Reference, reader, reader_buff, e),
                )),
                Collection::TAG_BYTES => collection.push_to_elems(InstanceElem::Collection(
                    from_event_start!(Collection, reader, reader_buff, e),
                )),
                _ => {
                    return Err(VOTableError::UnexpectedStartTag(
                        e.local_name().to_vec(),
                        Collection::TAG,
                    ))
                }
            },
            Event::Empty(ref e) => match e.local_name() {
                Attribute::TAG_BYTES => collection
                    .push_to_elems(InstanceElem::Attribute(Attribute::from_event_empty(e)?)),
                Reference::TAG_BYTES => collection
                    .push_to_elems(InstanceElem::Reference(Reference::from_event_empty(e)?)),
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
