use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::Reader;
use quick_xml::{
  events::{BytesStart, Event},
  Writer,
};
use std::{io::Write, str};

use super::{
  attribute_a::AttributePatA, attribute_b::AttributePatB, attribute_c::AttributePatC,
  collection::Collection, primarykey::PrimaryKey, reference::Reference, ElemImpl, ElemType,
};
use quick_xml::events::attributes::Attributes;

/*
    enum Instance context
    Description
    *   Enum of contexts available for Instance, these will influence the children attributes of said Instance
*/
#[derive(Clone, Debug)]
pub enum InstanceContexts {
  A,       // for Templates
  B,       // for Globals
  C,       // for Collection
  Writing, // for Writing which does not require a context
}

/*
    enum Instance Elem
    Description
    *    Enum of the elements that can be children of the mivot <INSTANCE> tag in any order.
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum InstanceElem {
  AttributePatA(AttributePatA),
  AttributePatB(AttributePatB),
  AttributePatC(AttributePatC),
  Instance(Instance),
  Reference(Reference),
  Collection(Collection),
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
      InstanceElem::AttributePatB(elem) => elem.write(writer, &()),
      InstanceElem::AttributePatC(elem) => elem.write(writer, &()),
      InstanceElem::Instance(elem) => elem.write(writer, &InstanceContexts::Writing),
      InstanceElem::Reference(elem) => elem.write(writer, &()),
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
pub struct GlobOrTempInstance {
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
impl GlobOrTempInstance {
  /*
      function New
      Description:
      *   creates a new Instance
      @generic N: Into<String>; a struct implementing the Into<String> trait
      @param dmtype N: a placeholder for the MANDATORY dmtype
      #returns Self: returns an instance of the GlobOrTempInstance struct
  */
  fn new<N: Into<String>>(dmtype: N) -> Self {
    Self {
      // MANDATORY
      dmtype: dmtype.into(),
      // OPTIONAL
      dmid: None,
      primary_keys: vec![],
      elems: vec![],
    }
  }
  /*
      function setters, enable the setting of an optional through self.set_"var"
  */
  impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<InstanceElem> for GlobOrTempInstance {
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
impl QuickXmlReadWrite for GlobOrTempInstance {
  // The TAG name here : <INSTANCE>
  const TAG: &'static str = "INSTANCE";
  // Potential context, here : InstanceContexts
  type Context = InstanceContexts;

  /*
      function from_attributes
      Description:
      *   creates Self from deserialized attributes contained inside the passed XML
      @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
      #returns Result<Self, VOTableError>: returns an instance of GlobOrTempInstance built using attributes or an error if reading doesn't work
  */
  fn from_attributes(
    attrs: quick_xml::events::attributes::Attributes,
  ) -> Result<Self, VOTableError> {
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

  /*
      function read_sub_elements
      Description:
      *   see function read_sub_elem
  */
  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    reader: quick_xml::Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
    read_sub_elem(self, context, reader, reader_buff)
  }

  /*
      function read_sub_elements
      todo UNIMPLEMENTED
  */
  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut quick_xml::Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    todo!()
  }

  /*
      function Write
      Description:
      *   function that writes the TAG
      @generic W: Write; a struct that implements the std::io::Write trait.
      @param self &mut: function is used like : self."function"
      @param writer &mut Writer<W>: the writer used to write the elements
      @param context &Self::Context: the context used for writing UNUSED
      #returns Result<(), VOTableError>: returns an error if writing doesn't work
  */
  fn write<W: Write>(
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
  /*
      function New
      Description:
      *   creates a new Instance
      @generic N: Into<String>; a struct implementing the Into<String> trait
      @param dmtype N: a placeholder for the MANDATORY dmtype
      @param dmrole N: a placeholder for the MANDATORY dmrole
      #returns Self: returns an instance of the Instance struct
  */
  fn new<N: Into<String>>(dmtype: N, dmrole: N) -> Self {
    Self {
      // MANDATORY
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      // OPTIONAL
      dmid: None,
      primary_keys: vec![],
      elems: vec![],
    }
  }
  impl_builder_opt_string_attr!(dmid);
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
// impl QuickXmlReadWrite for Instance {
//     // The TAG name here : <INSTANCE>
//     const TAG: &'static str = "INSTANCE";
//     // Potential context, here : InstanceContexts
//     type Context = InstanceContexts;

//     /*
//         function from_attributes
//         Description:
//         *   creates Self from deserialized attributes contained inside the passed XML
//         @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
//         #returns Result<Self, VOTableError>: returns an instance of Instance built using attributes or an error if reading doesn't work
//     */
//     impl_builder_from_attr!([dmrole, dmtype], [dmid]);

//     /*
//         function read_sub_elements
//         Description:
//         *   see function read_sub_elem
//     */
//     fn read_sub_elements<R: std::io::BufRead>(
//         &mut self,
//         reader: quick_xml::Reader<R>,
//         reader_buff: &mut Vec<u8>,
//         context: &Self::Context,
//     ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
//         read_sub_elem(self, context, reader, reader_buff)
//     }

//     /*
//         function read_sub_elements
//         todo UNIMPLEMENTED
//     */
//     fn read_sub_elements_by_ref<R: std::io::BufRead>(
//         &mut self,
//         _reader: &mut quick_xml::Reader<R>,
//         _reader_buff: &mut Vec<u8>,
//         _context: &Self::Context,
//     ) -> Result<(), crate::error::VOTableError> {
//         todo!()
//     }

//     /*
//         function Write
//         Description:
//         *   function that writes the TAG
//         @generic W: Write; a struct that implements the std::io::Write trait.
//         @param self &mut: function is used like : self."function"
//         @param writer &mut Writer<W>: the writer used to write the elements
//         @param context &Self::Context: the context used for writing UNUSED
//         #returns Result<(), VOTableError>: returns an error if writing doesn't work
//     */
//     impl_write_not_e!([dmrole, dmtype], [dmid], elems, [primary_keys]);
// }
impl_quickrw_not_e!(
  [dmrole, dmtype], // MANDATORY ATTRIBUTES
  [dmid],           // OPTIONAL ATTRIBUTES
  "INSTANCE",       // TAG, here : <ATTRIBUTE>
  Instance,         // Struct on which to impl
  InstanceContexts, // Context type
  elems,
  [primary_keys],
  read_sub_elem
);

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_sub_elem
    Description:
    *   reads the children of Instance
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<InstanceElem>; a struct that implements the quickXMLReadWrite and ElemImpl for InstanceElem traits.
    @param instance &mut T: an instance of T (here either GlobOrTempInstance or Instance)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_sub_elem<R: std::io::BufRead, T: QuickXmlReadWrite + ElemImpl<InstanceElem>>(
  instance: &mut T,
  context: &InstanceContexts,
  mut reader: quick_xml::Reader<R>,
  mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => {
        match e.local_name() {
          AttributePatB::TAG_BYTES => match context {
            InstanceContexts::A => instance.push_to_elems(InstanceElem::AttributePatA(
              from_event_start!(AttributePatA, reader, reader_buff, e),
            )),
            InstanceContexts::B => instance.push_to_elems(InstanceElem::AttributePatB(
              from_event_start!(AttributePatB, reader, reader_buff, e),
            )),
            InstanceContexts::C => instance.push_to_elems(InstanceElem::AttributePatC(
              from_event_start!(AttributePatC, reader, reader_buff, e),
            )),
            InstanceContexts::Writing => unreachable!(),
          },
          Instance::TAG_BYTES => instance.push_to_elems(InstanceElem::Instance(from_event_start!(
            Instance,
            reader,
            reader_buff,
            e,
            context
          ))),
          Reference::TAG_BYTES => instance.push_to_elems(InstanceElem::Reference(
            from_event_start!(Reference, reader, reader_buff, e),
          )),
          Collection::TAG_BYTES => instance.push_to_elems(InstanceElem::Collection(
            from_event_start!(Collection, reader, reader_buff, e),
          )),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Collection::TAG,
            ))
          }
        }
      }
      Event::Empty(ref e) => match e.local_name() {
        AttributePatB::TAG_BYTES => match context {
          InstanceContexts::A => instance.push_to_elems(InstanceElem::AttributePatA(
            AttributePatA::from_event_empty(e)?,
          )),
          InstanceContexts::B => instance.push_to_elems(InstanceElem::AttributePatB(
            AttributePatB::from_event_empty(e)?,
          )),
          InstanceContexts::C => instance.push_to_elems(InstanceElem::AttributePatC(
            AttributePatC::from_event_empty(e)?,
          )),
          InstanceContexts::Writing => unreachable!(),
        },
        Reference::TAG_BYTES => {
          instance.push_to_elems(InstanceElem::Reference(Reference::from_event_empty(e)?))
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
