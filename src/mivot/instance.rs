use crate::{error::VOTableError, is_empty, mivot::value_checker, QuickXmlReadWrite};
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
  attribute_a::AttributePatA, collection::CollectionPatA, primarykey::PrimaryKeyA, ElemImpl,
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
  pub dmtype: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dmid: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub primary_keys: Vec<PrimaryKeyA>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<InstanceElem>,
}
impl NoRoleInstance {
  impl_non_empty_new!([dmtype], [dmid], [primary_keys, elems]);
  /*
      function setters, enable the setting of an optional or mandatory through self.set_"var"
  */
  impl_builder_opt_string_attr!(dmid);
  impl_builder_mand_string_attr!(dmtype);
}
impl InstanceType for NoRoleInstance {
  fn push_pk(&mut self, pk: PrimaryKeyA) {
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
  fn push_elems(&mut self, elem: InstanceElem) {
    self.elems.push(elem);
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

/////////////////////////////////
/////// PATTERN A mand pk ///////
/////////////////////////////////

/*
    struct Globals or templates instance => pattern a
    @elem dmtype String: Modeled node related => MAND
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MandPKInstance {
  // MANDATORY
  pub dmtype: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dmid: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub primary_keys: Vec<PrimaryKeyA>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<InstanceElem>,
}
impl MandPKInstance {
  impl_non_empty_new!([dmtype], [dmid], [primary_keys, elems]);
  /*
      function setters, enable the setting of an optional through self.set_"var"
  */
  impl_builder_opt_string_attr!(dmid);
  impl_builder_mand_string_attr!(dmtype);
}
impl InstanceType for MandPKInstance {
  fn push_pk(&mut self, pk: PrimaryKeyA) {
    self.primary_keys.push(pk);
  }
}
impl ElemImpl<InstanceElem> for MandPKInstance {
  /*
      function push_to_elems
      Description:
      *   pushes an InstanceElem to the elems contained in struct
      @param self &mut: function is used like : self."function"
      @param dmid InstanceElem: the elem that needs to be pushed
      #returns ()
  */
  fn push_elems(&mut self, elem: InstanceElem) {
    self.elems.push(elem);
  }
}
impl_quickrw_not_e!(
  [dmtype],                       // MANDATORY ATTRIBUTES
  [dmid],                         // OPTIONAL ATTRIBUTES
  [dmrole],                       // potential empty tag
  "INSTANCE",                     // TAG, here : <INSTANCE>
  MandPKInstance,                 // Struct on which to impl
  (),                             // Context type
  [primary_keys],                 // Ordered elements
  read_mand_pk_instance_sub_elem, // Sub elements reader
  [elems]                         // Empty context writables
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
  pub dmrole: String,
  pub dmtype: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dmid: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub primary_keys: Vec<PrimaryKeyA>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<InstanceElem>,
}
impl Instance {
  impl_non_empty_new!([dmrole, dmtype], [dmid], [primary_keys, elems]);
  impl_builder_opt_string_attr!(dmid);
  impl_builder_mand_string_attr!(dmrole);
  impl_builder_mand_string_attr!(dmtype);
}
impl InstanceType for Instance {
  fn push_pk(&mut self, pk: PrimaryKeyA) {
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
  fn push_elems(&mut self, elem: InstanceElem) {
    self.elems.push(elem);
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
fn read_mand_pk_instance_sub_elem<R: std::io::BufRead>(
  instance: &mut MandPKInstance,
  _context: &(),
  reader: quick_xml::Reader<R>,
  reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
  let reader = read_instance_sub_elem(instance, _context, reader, reader_buff)?;
  if instance.primary_keys.is_empty() {
    Err(VOTableError::Custom(
      "When a collection is child of globals then its instances need a primary key.".to_owned(),
    ))
  } else {
    Ok(reader)
  }
}

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
        AttributePatA::TAG_BYTES => instance.push_elems(InstanceElem::AttributePatA(
          from_event_start!(AttributePatA, reader, reader_buff, e),
        )),
        Instance::TAG_BYTES => instance.push_elems(InstanceElem::Instance(from_event_start!(
          Instance,
          reader,
          reader_buff,
          e
        ))),
        DynRef::TAG_BYTES => {
          if e
            .attributes()
            .any(|attribute| attribute.as_ref().unwrap().key == "sourceref".as_bytes())
          {
            instance.push_elems(InstanceElem::DynRef(from_event_start!(
              DynRef,
              reader,
              reader_buff,
              e
            )))
          } else {
            instance.push_elems(InstanceElem::StaticRef(from_event_start!(
              StaticRef,
              reader,
              reader_buff,
              e
            )))
          }
        }
        CollectionPatA::TAG_BYTES => instance.push_elems(InstanceElem::Collection(
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
        Instance::TAG_BYTES => {
          instance.push_elems(InstanceElem::Instance(Instance::from_event_empty(e)?))
        }
        AttributePatA::TAG_BYTES => instance.push_elems(InstanceElem::AttributePatA(
          AttributePatA::from_event_empty(e)?,
        )),
        DynRef::TAG_BYTES => {
          if e
            .attributes()
            .any(|attribute| attribute.as_ref().unwrap().key == "sourceref".as_bytes())
          {
            instance.push_elems(InstanceElem::DynRef(DynRef::from_event_empty(e)?))
          } else {
            instance.push_elems(InstanceElem::StaticRef(StaticRef::from_event_empty(e)?))
          }
        }
        CollectionPatA::TAG_BYTES => instance.push_elems(InstanceElem::Collection(
          CollectionPatA::from_event_empty(e)?,
        )),
        PrimaryKeyA::TAG_BYTES => instance.push_pk(PrimaryKeyA::from_event_empty(e)?),
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

#[cfg(test)]
mod tests {
  use crate::{
    mivot::test::test_error,
    mivot::{
      instance::{Instance, MandPKInstance, NoRoleInstance},
      test::get_xml,
    },
    tests::test_read,
  };

  #[test]
  fn test_no_role_instances_read() {
    // OK INSTANCES
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.3.xml");
    println!("testing 5.3");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.4.xml");
    println!("testing 5.4");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.5.xml");
    println!("testing 5.5");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.6.xml");
    println!("testing 5.6");
    test_read::<NoRoleInstance>(&xml);
    // KO INSTANCES
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.1.xml");
    println!("testing 5.1"); // dmid + dmrole + dmtype; must have no or empty dmrole. (parser can overlook this and write it correctly later)
    test_read::<NoRoleInstance>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.2.xml");
    println!("testing 5.2"); // no dmid + dmrole + dmtype; must have no or empty dmrole. (parser can overlook this and write it correctly later)
    test_read::<NoRoleInstance>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.7.xml");
    println!("testing 5.7"); // empty dmid + valid dmrole + valid dmtype; If present, dmid must not be empty.
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.8.xml");
    println!("testing 5.8"); // valid dmid + valid dmrole + no dmtype; must have dmtype
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.9.xml");
    println!("testing 5.9"); // valid dmid + valid dmrole + empty dmtype; dmtype must not be empty
    test_error::<NoRoleInstance>(&xml, false);
  }

  #[test]
  fn test_instance_child_read() {
    // No role instances with a child instance (represented by Instance struct) or child instance directly
    // OK INSTANCES
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.40.xml");
    println!("testing 5.40");
    test_read::<Instance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.41.xml");
    println!("testing 5.41");
    test_read::<Instance>(&xml);
    // KO INSTANCES
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.42.xml");
    println!("testing 5.42"); // dmid + empty dmrole + dmtype; must have non-empty dmrole.
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.43.xml");
    println!("testing 5.43"); // no dmid + empty dmrole + dmtype; must have non-empty dmrole.
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.44.xml");
    println!("testing 5.44"); // dmid + no dmrole + dmtype; (must have non-empty dmrole)
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.45.xml");
    println!("testing 5.45"); // no dmid + no dmrole + dmtype; (must have non-empty dmrole)
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.46.xml");
    println!("testing 5.46"); // empty dmid + valid dmrole + valid dmtype; If present, dmid must not be empty.
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.47.xml");
    println!("testing 5.47"); // valid dmid + valid dmrole + no dmtype; must have dmtype
    test_error::<NoRoleInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.48.xml");
    println!("testing 5.48"); // valid dmid + valid dmrole + empty dmtype; dmtype must not be empty
    test_error::<NoRoleInstance>(&xml, false);
  }

  #[test]
  fn generic_test_instances_read() {
    // OK INSTANCES
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.50.xml");
    println!("testing 5.50");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.51.xml");
    println!("testing 5.51");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.52.xml");
    println!("testing 5.52");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.53.xml");
    println!("testing 5.53");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.54.xml");
    println!("testing 5.54");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.55.xml");
    println!("testing 5.55");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.56.xml");
    println!("testing 5.56");
    test_read::<NoRoleInstance>(&xml);
    let xml = get_xml("./resources/mivot/5/test_5_ok_5.58.xml");
    println!("testing 5.58");
    test_read::<NoRoleInstance>(&xml);
    // KO INSTANCES
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.57.xml");
    println!("testing 5.57"); // with PRIMARY_KEY not first (parser can overlook this and write it correctly later)
    test_read::<NoRoleInstance>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.59.xml");
    println!("testing 5.59"); // contains subnode other than (PK,A,I,R,C)
    test_error::<NoRoleInstance>(&xml, false);
  }

  //TODO test CollectionPatB with a MANDPKINSTANCE missing a PK
  #[test]
  fn test_2_instances_read() {
    let xml = get_xml("./resources/mivot/5/test_5_ko_5.29.xml");
    println!("testing 5.29"); // no PRIMARY_KEY; must have PRIMARY_KEY in this context
    test_read::<MandPKInstance>(&xml); // ! fixed when reading from a parent collection
  }
}
