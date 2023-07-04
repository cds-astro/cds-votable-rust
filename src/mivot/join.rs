use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::io::Write;
use std::str;

use super::r#where::{NoFkWhere, Where};
use super::{ElemImpl, ElemType};

/*
    enum JoinWhereElem
    Description
    *    Enum of the elements that can be children of the mivot <COLLECTION> tag in any order.
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum JoinWhereElem {
  Where(Where),
  NoFkWhere(NoFkWhere),
}
impl ElemType for JoinWhereElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      JoinWhereElem::Where(elem) => elem.write(writer, &()),
      JoinWhereElem::NoFkWhere(elem) => elem.write(writer, &()),
    }
  }
}

/*
    struct Join => pattern A & B (cannot be determined from context)
    @elem dmref Option<String>: Modeled node related => OPT
    @elem sourceref Option<String>: Reference of the TEMPLATES or COLLECTION to be joined with. => OPT
    @elem wheres: Join conditions
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Join {
  #[serde(skip_serializing_if = "Option::is_none")]
  dmref: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  sourceref: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  wheres: Vec<JoinWhereElem>,
}
impl Join {
  impl_non_empty_new!([], [sourceref, dmref], [wheres]);
  impl_builder_opt_string_attr!(sourceref);
  impl_builder_opt_string_attr!(dmref);
}
impl ElemImpl<JoinWhereElem> for Join {
  fn push_to_elems(&mut self, elem: JoinWhereElem) {
    self.wheres.push(elem)
  }
}
impl_quickrw_not_e!(
  [],                 // MANDATORY ATTRIBUTES
  [sourceref, dmref], // OPTIONAL ATTRIBUTES
  "JOIN",             // TAG, here : <INSTANCE>
  Join,               // Struct on which to impl
  (),                 // Context type
  [],                 // Ordered elements
  read_join_sub_elem, // Sub elements reader
  [wheres]            // Empty context writables
);

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_join_sub_elem
    Description:
    *   reads the children of Join
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @param instance &mut Join: an instance of Join
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_join_sub_elem<R: std::io::BufRead>(
  join: &mut Join,
  _context: &(),
  mut reader: quick_xml::Reader<R>,
  reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => match e.local_name() {
        _ => {
          return Err(VOTableError::UnexpectedStartTag(
            e.local_name().to_vec(),
            Join::TAG,
          ))
        }
      },
      Event::Empty(ref e) => match e.local_name() {
        Where::TAG_BYTES => {
          if e
            .attributes()
            .find(|attribute| attribute.as_ref().unwrap().key == "foreignkey".as_bytes())
            .is_some()
          {
            join.push_to_elems(JoinWhereElem::Where(Where::from_event_empty(e)?))
          } else {
            join.push_to_elems(JoinWhereElem::NoFkWhere(NoFkWhere::from_event_empty(e)?))
          }
        }
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            Join::TAG,
          ))
        }
      },
      Event::Text(e) if is_empty(e) => {}
      Event::End(e) if e.local_name() == Join::TAG_BYTES => return Ok(reader),
      Event::Eof => return Err(VOTableError::PrematureEOF(Join::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", Join::TAG, event),
    }
  }
}
