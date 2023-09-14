//! The `GLOBALS` block contains model element(s)  having no reference to any table.
//! Thus, an element in a `GLOBALS` block cannot contains a `ref` attribute pointing to a table
//! (`FIELD` or `PARAM`), **but** it may contain a `ref` attribute pointing to a `PARAM` which is
//! not in a VOTable table.
//! For `PRIMARY_KEY`,

use std::{io::Write, str};

use paste::paste;

use quick_xml::{
  events::{BytesStart, Event},
  Reader, Writer,
};

use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};

pub mod collection;
use collection::Collection;

pub mod instance;
use instance::Instance;

/// The two sub-elements `GLOBALS` may contains (in any order).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum GlobalsElem {
  Instance(Instance),
  Collection(Collection),
}
impl GlobalsElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      GlobalsElem::Instance(elem) => elem.write(writer, &()),
      GlobalsElem::Collection(elem) => elem.write(writer, &()),
    }
  }
}

/// Structure storing the content of the `GLOABLS` tag.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Globals {
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<GlobalsElem>,
}

impl Globals {
  pub fn new() -> Self {
    Self {
      elems: Default::default(),
    }
  }

  impl_builder_push_elem!(Instance, GlobalsElem);
  impl_builder_push_elem!(Collection, GlobalsElem);
}

impl_quickrw_not_e_no_a!(
  "GLOBALS",
  Globals,
  (),
  [],
  read_globals_sub_elem_by_ref,
  [elems]
);

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
fn read_globals_sub_elem_by_ref<R: std::io::BufRead>(
  globals: &mut Globals,
  _context: &(),
  mut reader: &mut quick_xml::Reader<R>,
  mut reader_buff: &mut Vec<u8>,
) -> Result<(), crate::error::VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => match e.local_name() {
        Instance::TAG_BYTES => {
          globals.push_instance_by_ref(from_event_start_by_ref!(Instance, reader, reader_buff, e))
        }
        Collection::TAG_BYTES => {
          let dmid = collection::get_dmid_from_atttributes(e.attributes())?;
          let collection = collection::create_collection_from_dmid_and_reading_sub_elems(
            dmid,
            &(),
            reader,
            reader_buff,
          )?;
          globals.push_collection_by_ref(collection);
        }
        _ => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            Globals::TAG,
          ))
        }
      },
      Event::Empty(ref e) => match e.local_name() {
        Instance::TAG_BYTES => globals.push_instance_by_ref(Instance::from_event_empty(e)?),
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            Globals::TAG,
          ));
        }
      },
      Event::Text(e) if is_empty(e) => {}
      Event::End(e) if e.local_name() == Globals::TAG_BYTES => return Ok(()),
      Event::Eof => return Err(VOTableError::PrematureEOF(Globals::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", Globals::TAG, event),
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
    // Should npot be valid according to 4.14 and Table 28: Dynamic Primary key only in TEMPLATES
    //   let xml = get_xml("./resources/mivot/3/test_3_ok_3.1.xml");
    //   println!("testing 3.1");
    //   test_read::<Globals>(&xml);
    //  let xml = get_xml("./resources/mivot/3/test_3_ok_3.2.xml");
    //  println!("testing 3.2");
    //  test_read::<Globals>(&xml);
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.3.xml");
    println!("testing 3.3");
    test_read::<Globals>(&xml);
    //  let xml = get_xml("./resources/mivot/3/test_3_ok_3.4.xml");
    //  println!("testing 3.4");
    //  test_read::<Globals>(&xml);
    let xml = get_xml("./resources/mivot/3/test_3_ok_3.5.xml");
    println!("testing 3.5");
    test_read::<Globals>(&xml);
    // KO MODELS
    let xml = get_xml("./resources/mivot/3/test_3_ko_3.6.xml");
    println!("testing 3.6"); // Unexpected subnode.
    test_error::<Globals>(&xml, false);
  }
}