use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::Reader;
use quick_xml::{
  events::{BytesStart, Event},
  Writer,
};
use std::{io::Write, str};

use super::reference::{DynRef, StaticRef};
use super::InstanceType;
use super::{
    attribute_a::AttributePatA, collection::CollectionPatA, primarykey::PrimaryKey, ElemImpl,
    ElemType,
};
use quick_xml::events::attributes::Attributes;

/*
    enum Instance Elem
    Description
    *    Enum of the elements that can be children of the mivot <INSTANCE> tag in any order.
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum InstanceElem {
    AttributePatA(AttributePatA),
    Instance(Instance),
    StaticRef(StaticRef),
    DynRef(DynRef),
    Collection(CollectionPatA),
}
impl ElemType for InstanceElem {
    /*
        function Write
        Description:
        *   function that writes the elements as mivot TAGS
        @generic W: Write; a struct that implements the std::io::Write trait.
        @param self &mut: function is used like : self."function"
        @param writer &mut Writer<W>: the writer used to write the elements
        #returns Result<(), VOTableError>: returns an error if writing doesn't work
    */
    fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
        match self {
            InstanceElem::AttributePatA(elem) => elem.write(writer, &()),
            InstanceElem::Instance(elem) => elem.write(writer, &()),
            InstanceElem::StaticRef(elem) => elem.write(writer, &()),
            InstanceElem::DynRef(elem) => elem.write(writer, &()),
            InstanceElem::Collection(elem) => elem.write(writer, &()),
        }
    }
  }

/////////////////////////
/////// PATTERN A ///////
/////////////////////////

/*
    struct Globals or templates instance => pattern a
    @elem dmtype String: Modeled node related => MAND
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NoRoleInstance {
    // MANDATORY
    dmtype: String,
    // OPTIONAL
    #[serde(skip_serializing_if = "Option::is_none")]
    dmid: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<InstanceElem>,
}
impl NoRoleInstance {
    impl_non_empty_new!([dmtype], [dmid], [primary_keys, elems]);
    /*
        function setters, enable the setting of an optional through self.set_"var"
    */
    impl_builder_opt_string_attr!(dmid);
}
impl InstanceType for NoRoleInstance {
    fn push2_pk(&mut self, pk: PrimaryKey) {
        self.primary_keys.push(pk);
    }
}
impl ElemImpl<InstanceElem> for NoRoleInstance {
    /*
        function push_to_elems
        Description:
        *   pushes an InstanceElem to the elems contained in struct
        @param self &mut: function is used like : self."function"
        @param dmid InstanceElem: the elem that needs to be pushed
        #returns ()
    */
    fn push_to_elems(&mut self, elem: InstanceElem) {
        self.elems.push(elem)
    }
}
impl_quickrw_not_e!(
    [dmtype],               // MANDATORY ATTRIBUTES
    [dmid],                 // OPTIONAL ATTRIBUTES
    [dmrole],               // potential empty tag
    "INSTANCE",             // TAG, here : <INSTANCE>
    NoRoleInstance,         // Struct on which to impl
    (),                     // Context type
    [primary_keys],         // Ordered elements
    read_instance_sub_elem, // Sub elements reader
    [elems]                 // Empty context writables
);

/////////////////////////
/////// PATTERN B ///////
/////////////////////////

/*
    struct Instance => pattern b
    @elem dmrole String: Modeled node related => MAND
    @elem dmtype String: Modeled node related => MAND
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Instance {
  // MANDATORY
  dmrole: String,
  dmtype: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  dmid: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  primary_keys: Vec<PrimaryKey>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<InstanceElem>,
}
impl Instance {
    impl_non_empty_new!([dmrole, dmtype], [dmid], [primary_keys, elems]);
    impl_builder_opt_string_attr!(dmid);
}
impl InstanceType for Instance {
    fn push2_pk(&mut self, pk: PrimaryKey) {
        self.primary_keys.push(pk);
    }
}
impl ElemImpl<InstanceElem> for Instance {
  /*
      function push_to_elems
      Description:
      *   pushes an InstanceElem to the elems contained in struct
      @param self &mut: function is used like : self."function"
      @param dmid InstanceElem: the elem that needs to be pushed
      #returns ()
  */
  fn push_to_elems(&mut self, elem: InstanceElem) {
    self.elems.push(elem)
  }
}
impl_quickrw_not_e!(
    [dmrole, dmtype], // MANDATORY ATTRIBUTES
    [dmid],           // OPTIONAL ATTRIBUTES
    "INSTANCE",       // TAG, here : <INSTANCE>
    Instance,         // Struct on which to impl
    (),               // Context type
    [primary_keys],
    read_instance_sub_elem,
    [elems]
);

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_instance_sub_elem
    Description:
    *   reads the children of Instance
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<InstanceElem>; a struct that implements the quickXMLReadWrite and ElemImpl for InstanceElem traits.
    @param instance &mut T: an instance of T (here either NoRoleInstance or Instance)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_instance_sub_elem<
    R: std::io::BufRead,
    T: QuickXmlReadWrite + ElemImpl<InstanceElem> + InstanceType,
>(
    instance: &mut T,
    _context: &(),
    mut reader: quick_xml::Reader<R>,
    mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
    loop {
        let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
            Event::Start(ref e) => match e.local_name() {
                AttributePatA::TAG_BYTES => instance.push_to_elems(InstanceElem::AttributePatA(
                    from_event_start!(AttributePatA, reader, reader_buff, e),
                )),
                Instance::TAG_BYTES => instance.push_to_elems(InstanceElem::Instance(
                    from_event_start!(Instance, reader, reader_buff, e),
                )),
                DynRef::TAG_BYTES => {
                    if e.attributes()
                        .find(|attribute| attribute.as_ref().unwrap().key == "sourceref".as_bytes())
                        .is_some()
                    {
                        instance.push_to_elems(InstanceElem::DynRef(from_event_start!(
                            DynRef,
                            reader,
                            reader_buff,
                            e
                        )))
                    } else {
                        instance.push_to_elems(InstanceElem::StaticRef(from_event_start!(
                            StaticRef,
                            reader,
                            reader_buff,
                            e
                        )))
                    }
                }
                CollectionPatA::TAG_BYTES => instance.push_to_elems(InstanceElem::Collection(
                    from_event_start!(CollectionPatA, reader, reader_buff, e),
                )),
                _ => {
                    return Err(VOTableError::UnexpectedStartTag(
                        e.local_name().to_vec(),
                        Instance::TAG,
                    ))
                }
              },
            Event::Empty(ref e) => match e.local_name() {
                AttributePatA::TAG_BYTES => instance.push_to_elems(InstanceElem::AttributePatA(
                    AttributePatA::from_event_empty(e)?,
                )),
                DynRef::TAG_BYTES => {
                    if e.attributes()
                        .find(|attribute| attribute.as_ref().unwrap().key == "sourceref".as_bytes())
                        .is_some()
                    {
                        instance.push_to_elems(InstanceElem::DynRef(DynRef::from_event_empty(e)?))
                    } else {
                        instance
                            .push_to_elems(InstanceElem::StaticRef(StaticRef::from_event_empty(e)?))
                    }
                }
                PrimaryKey::TAG_BYTES => instance.push2_pk(PrimaryKey::from_event_empty(e)?),
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
