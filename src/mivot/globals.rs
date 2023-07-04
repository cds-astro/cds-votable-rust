use paste::paste;
use quick_xml::Reader;
use quick_xml::{
  events::{BytesStart, Event},
  Writer,
};

use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};

use super::{collection::CollectionPatB, instance::NoRoleInstance, ElemImpl, ElemType};
use std::{io::Write, str};

/*
    enum GlobalsElem
    Description
    *    Enum of the elements that can be children of the mivot <GLOBALS> tag in any order.
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum GlobalsElem {
  Instance(NoRoleInstance),
  Collection(CollectionPatB),
}
impl ElemType for GlobalsElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      GlobalsElem::Instance(elem) => elem.write(writer, &()),
      GlobalsElem::Collection(elem) => elem.write(writer, &()),
    }
  }
}

/*
    struct Globals
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Globals {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  elems: Vec<GlobalsElem>,
}
impl ElemImpl<GlobalsElem> for Globals {
  fn push_to_elems(&mut self, elem: GlobalsElem) {
    self.elems.push(elem)
  }
}
impl_quickrw_not_e_no_a!("GLOBALS", Globals, (), [], read_globals_sub_elem, [elems]);

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_globals_sub_elem
    Description:
    *   reads the children of Globals
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<GlobalsElem>; a struct that implements the quickXMLReadWrite and ElemImpl for GlobalsElem traits.
    @param instance &mut T: an instance of T (here Globals)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_globals_sub_elem<R: std::io::BufRead, T: QuickXmlReadWrite + ElemImpl<GlobalsElem>>(
  globals: &mut T,
  _context: &(),
  mut reader: quick_xml::Reader<R>,
  mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => match e.local_name() {
        NoRoleInstance::TAG_BYTES => globals.push_to_elems(GlobalsElem::Instance(
          from_event_start!(NoRoleInstance, reader, reader_buff, e),
        )),
        CollectionPatB::TAG_BYTES => globals.push_to_elems(GlobalsElem::Collection(
          from_event_start!(CollectionPatB, reader, reader_buff, e),
        )),
        _ => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            T::TAG,
          ))
        }
      },
      Event::Empty(ref e) => match e.local_name() {
        NoRoleInstance::TAG_BYTES => {
          globals.push_to_elems(GlobalsElem::Instance(NoRoleInstance::from_event_empty(e)?))
        }
        CollectionPatB::TAG_BYTES => globals.push_to_elems(GlobalsElem::Collection(
          CollectionPatB::from_event_empty(e)?,
        )),
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            T::TAG,
          ));
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
    mivot::globals::Globals,
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  #[test]
  fn test_globals_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.1.xml");
    println!("testing 3.1");
    test_read::<Globals>(&xml);
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.2.xml");
    println!("testing 3.2");
    test_read::<Globals>(&xml);
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.3.xml");
    println!("testing 3.3");
    test_read::<Globals>(&xml);
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.4.xml");
    println!("testing 3.4");
    test_read::<Globals>(&xml);
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.5.xml");
    println!("testing 3.5");
    test_read::<Globals>(&xml);
    // KO MODELS
    let xml = get_xml("./resources/mivot/3/test_3_ko_3.6.xml");
    println!("testing 3.6"); // Unexpected subnode.
    test_error::<Globals>(&xml, false);
  }
}
