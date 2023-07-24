use crate::Attributes;
use crate::{error::VOTableError, is_empty, mivot::value_checker, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::Reader;
use quick_xml::{
  events::{BytesStart, Event},
  Writer,
};
use std::{io::Write, str};

use super::instance::{MandPKInstance, NoRoleInstance};
use super::join::SrcJoin;
use super::reference::{DynRef, StaticRef};
use super::CollectionType;
use super::{attribute_c::AttributePatC, join::Join, primarykey::PrimaryKeyB, ElemImpl, ElemType};

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
  PKInstance(MandPKInstance),
  StaticRef(StaticRef),
  DynRef(DynRef),
  Collection(CollectionPatC),
  Join(Join),
  SrcJoin(SrcJoin),
}
impl ElemType for CollectionElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      CollectionElem::Attribute(elem) => elem.write(writer, &()),
      CollectionElem::Instance(elem) => elem.write(writer, &()),
      CollectionElem::PKInstance(elem) => elem.write(writer, &()),
      CollectionElem::StaticRef(elem) => elem.write(writer, &()),
      CollectionElem::DynRef(elem) => elem.write(writer, &()),
      CollectionElem::Collection(elem) => elem.write(writer, &()),
      CollectionElem::Join(elem) => elem.write(writer, &()),
      CollectionElem::SrcJoin(elem) => elem.write(writer, &()),
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
  primary_keys: Vec<PrimaryKeyB>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<CollectionElem>,
  checker: Vec<String>,
}
impl CollectionPatA {
  impl_non_empty_new!([dmrole], [dmid], [primary_keys, elems, checker]);
  impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<CollectionElem> for CollectionPatA {
  fn push_to_elems(&mut self, elem: CollectionElem) {
    self.elems.push(elem)
  }
}
impl CollectionType for CollectionPatA {
  fn push_to_checker(&mut self, str: String) {
    self.checker.push(str);
  }
  fn check_elems(&mut self) -> bool {
    let first = self.checker.get(0);
    let mut res = false;
    self.checker.iter().for_each(|s| {
      if first != Some(s) {
        res = true;
      }
    });
    res
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
  primary_keys: Vec<PrimaryKeyB>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<CollectionElem>,
  checker: Vec<String>,
}
impl CollectionPatB {
  impl_non_empty_new!([dmid], [], [primary_keys, elems, checker]);
}
impl ElemImpl<CollectionElem> for CollectionPatB {
  fn push_to_elems(&mut self, elem: CollectionElem) {
    self.elems.push(elem)
  }
}
impl CollectionType for CollectionPatB {
  fn push_to_checker(&mut self, str: String) {
    self.checker.push(str);
  }
  fn check_elems(&mut self) -> bool {
    let first = self.checker.get(0);
    let mut res = false;
    if self.checker.contains(&"join".to_owned()) && self.checker.len() > 1 {
      return true;
    }
    self.checker.iter().for_each(|s| {
      if first != Some(s) {
        res = true
      }
    });
    res
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
  read_collection_b_sub_elem,
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
  primary_keys: Vec<PrimaryKeyB>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<CollectionElem>,
  checker: Vec<String>,
}
impl CollectionPatC {
  impl_non_empty_new!([], [dmid], [primary_keys, elems, checker]);
  impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<CollectionElem> for CollectionPatC {
  fn push_to_elems(&mut self, elem: CollectionElem) {
    self.elems.push(elem)
  }
}
impl CollectionType for CollectionPatC {
  fn push_to_checker(&mut self, str: String) {
    self.checker.push(str);
  }
  fn check_elems(&mut self) -> bool {
    let first = self.checker.get(0);
    let mut res = false;
    self.checker.iter().for_each(|s| {
      if first != Some(s) {
        res = true;
      }
    });
    res
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
  T: QuickXmlReadWrite + ElemImpl<CollectionElem> + CollectionType,
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
        AttributePatC::TAG_BYTES => {
          collection.push_to_checker("attribute".to_owned());
          collection.push_to_elems(CollectionElem::Attribute(from_event_start!(
            AttributePatC,
            reader,
            reader_buff,
            e
          )))
        }
        NoRoleInstance::TAG_BYTES => {
          collection.push_to_checker("instance".to_owned());
          collection.push_to_elems(CollectionElem::Instance(from_event_start!(
            NoRoleInstance,
            reader,
            reader_buff,
            e
          )))
        }
        DynRef::TAG_BYTES => {
          collection.push_to_checker("reference".to_owned());
          if e
            .attributes()
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
            return Err(VOTableError::Custom(
              "A static reference should be empty".to_owned(),
            ));
          }
        }
        CollectionPatC::TAG_BYTES => {
          collection.push_to_checker("collection".to_owned());
          collection.push_to_elems(CollectionElem::Collection(from_event_start!(
            CollectionPatC,
            reader,
            reader_buff,
            e
          )))
        }
        Join::TAG_BYTES => {
          collection.push_to_checker("join".to_owned());
          collection.push_to_elems(CollectionElem::Join(from_event_start!(
            Join,
            reader,
            reader_buff,
            e
          )))
        }
        _ => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            CollectionPatA::TAG,
          ))
        }
      },
      Event::Empty(ref e) => match e.local_name() {
        AttributePatC::TAG_BYTES => {
          collection.push_to_checker("attribute".to_owned());
          collection.push_to_elems(CollectionElem::Attribute(AttributePatC::from_event_empty(
            e,
          )?))
        }
        StaticRef::TAG_BYTES => {
          collection.push_to_checker("reference".to_owned());
          if e
            .attributes()
            .find(|attribute| attribute.as_ref().unwrap().key == "dmref".as_bytes())
            .is_some()
          {
            collection.push_to_elems(CollectionElem::StaticRef(StaticRef::from_event_empty(e)?))
          } else {
            return Err(VOTableError::Custom(
              "A dynamic reference shouldn't be empty".to_owned(),
            ));
          }
        }
        CollectionPatC::TAG_BYTES => {
          collection.push_to_checker("collection".to_owned());
          collection.push_to_elems(CollectionElem::Collection(
            CollectionPatC::from_event_empty(e)?,
          ))
        }
        NoRoleInstance::TAG_BYTES => {
          collection.push_to_checker("instance".to_owned());
          collection.push_to_elems(CollectionElem::Instance(NoRoleInstance::from_event_empty(
            e,
          )?))
        }
        Join::TAG_BYTES => {
          collection.push_to_checker("join".to_owned());
          collection.push_to_elems(CollectionElem::Join(Join::from_event_empty(e)?))
        }
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            T::TAG,
          ))
        }
      },
      Event::Text(e) if is_empty(e) => {}
      Event::End(e) if e.local_name() == T::TAG_BYTES => {
        if collection.check_elems() {
          return Err(VOTableError::Custom(
            "A collection cannot have items of diffrent types".to_owned(),
          ));
        } else {
          return Ok(reader);
        }
      }
      Event::Eof => return Err(VOTableError::PrematureEOF(T::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", T::TAG, event),
    }
  }
}

/*
    function read_collectionB_sub_elem
    Description:
    *   reads the children of Collection
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<CollectionElem>; a struct that implements the quickXMLReadWrite and ElemImpl for CollectionElem traits.
    @param instance &mut T: an instance of CollectionPatB
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/

fn read_collection_b_sub_elem<R: std::io::BufRead>(
  collection: &mut CollectionPatB,
  _context: &(),
  mut reader: quick_xml::Reader<R>,
  mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => match e.local_name() {
        MandPKInstance::TAG_BYTES => {
          collection.push_to_checker("instance".to_owned());
          collection.push_to_elems(CollectionElem::PKInstance(from_event_start!(
            MandPKInstance,
            reader,
            reader_buff,
            e
          )))
        }
        DynRef::TAG_BYTES => {
          collection.push_to_checker("reference".to_owned());
          if e
            .attributes()
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
            return Err(VOTableError::Custom(
              "A static reference should be empty".to_owned(),
            ));
          }
        }
        CollectionPatC::TAG_BYTES => {
          collection.push_to_checker("collection".to_owned());
          collection.push_to_elems(CollectionElem::Collection(from_event_start!(
            CollectionPatC,
            reader,
            reader_buff,
            e
          )))
        }
        SrcJoin::TAG_BYTES => {
          collection.push_to_checker("join".to_owned());
          collection.push_to_elems(CollectionElem::SrcJoin(from_event_start!(
            SrcJoin,
            reader,
            reader_buff,
            e
          )))
        }
        _ => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            CollectionPatA::TAG,
          ))
        }
      },
      Event::Empty(ref e) => match e.local_name() {
        StaticRef::TAG_BYTES => {
          collection.push_to_checker("reference".to_owned());
          if e
            .attributes()
            .find(|attribute| attribute.as_ref().unwrap().key == "dmref".as_bytes())
            .is_some()
          {
            collection.push_to_elems(CollectionElem::StaticRef(StaticRef::from_event_empty(e)?))
          } else {
            return Err(VOTableError::Custom(
              "A dynamic reference shouldn't be empty".to_owned(),
            ));
          }
        }
        SrcJoin::TAG_BYTES => {
          collection.push_to_checker("join".to_owned());
          collection.push_to_elems(CollectionElem::SrcJoin(SrcJoin::from_event_empty(e)?))
        }
        CollectionPatC::TAG_BYTES => {
          collection.push_to_checker("collection".to_owned());
          collection.push_to_elems(CollectionElem::Collection(
            CollectionPatC::from_event_empty(e)?,
          ))
        }
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            CollectionPatB::TAG,
          ))
        }
      },
      Event::Text(e) if is_empty(e) => {}
      Event::End(e) if e.local_name() == CollectionPatB::TAG_BYTES => {
        if collection.check_elems() {
          return Err(VOTableError::Custom(
            "A collection cannot have items of diffrent types".to_owned(),
          ));
        } else {
          return Ok(reader);
        }
      }
      Event::Eof => return Err(VOTableError::PrematureEOF(CollectionPatB::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", CollectionPatB::TAG, event),
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::{
      collection::{CollectionPatA, CollectionPatB},
      test::{get_xml, test_error},
    },
    tests::test_read,
  };

  #[test]
  fn test_collection_b_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.1.xml");
    println!("testing 8.1");
    test_read::<CollectionPatB>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.4.xml");
    println!("testing 8.4");
    test_read::<CollectionPatB>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.6.xml");
    println!("testing 8.6");
    test_read::<CollectionPatB>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.7.xml");
    println!("testing 8.7");
    test_read::<CollectionPatB>(&xml);
    // KO MODELS
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.2.xml");
    println!("testing 8.2"); // must have dmid
    test_error::<CollectionPatB>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.3.xml");
    println!("testing 8.3"); // dmid must not be empty
    test_error::<CollectionPatB>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.5.xml");
    println!("testing 8.5"); // must have empty or no dmrole. (parser can overlook this and write it correctly later)
    test_read::<CollectionPatB>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.8.xml");
    println!("testing 8.8"); // contains other than INSTANCE.
    test_error::<CollectionPatB>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.9.xml");
    println!("testing 8.9"); // with invalid child
    test_error::<CollectionPatB>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.25.xml");
    println!("testing 8.25"); // Collection of globals cannot have an instance without a PK
    test_error::<CollectionPatB>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.25.xml");
    println!("testing 8.26"); // Collection of globals cannot have an instance without a PK
    test_error::<CollectionPatB>(&xml, false);
  }

  #[test]
  fn test_collection_a_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.15.xml");
    println!("testing 8.15");
    test_read::<CollectionPatA>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.16.xml");
    println!("testing 8.16");
    test_read::<CollectionPatA>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.17.xml");
    println!("testing 8.17");
    test_read::<CollectionPatA>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.18.xml");
    println!("testing 8.18");
    test_read::<CollectionPatA>(&xml);
    let xml = get_xml("./resources/mivot/8/test_8_ok_8.19.xml");
    println!("testing 8.19");
    test_read::<CollectionPatA>(&xml);
    // KO MODELS
    // let xml = get_xml("./resources/mivot/8/test_8_ko_8.10.xml");
    // println!("testing 8.10"); // must not have dmid
    // test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.11.xml");
    println!("testing 8.11"); // dmid if present shouldn't be empty
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.12.xml");
    println!("testing 8.12"); // must have dmrole
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.13.xml");
    println!("testing 8.13"); // dmrole must not be empty.
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.13.xml");
    println!("testing 8.14"); // dmrole must not be blank.
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.20.xml");
    println!("testing 8.20"); // COLLECTION of INSTANCE + JOIN (local and external instances)
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.21.xml");
    println!("testing 8.21"); // COLLECTION of ATTRIBUTE + (other)
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.22.xml");
    println!("testing 8.22"); // COLLECTION of REFERENCE + (other)
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.23.xml");
    println!("testing 8.23"); // COLLECTION of INSTANCE + (other)
    test_error::<CollectionPatA>(&xml, false);
    let xml = get_xml("./resources/mivot/8/test_8_ko_8.24.xml");
    println!("testing 8.24"); // COLLECTION of other
    test_error::<CollectionPatA>(&xml, false);
  }
}
