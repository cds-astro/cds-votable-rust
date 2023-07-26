use crate::{error::VOTableError, is_empty, mivot::value_checker, QuickXmlReadWrite};
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
  dmref: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  sourceref: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  wheres: Vec<JoinWhereElem>,
}
impl Join {
  impl_non_empty_new!([dmref], [sourceref], [wheres]);
  impl_builder_opt_string_attr!(sourceref);
}
impl ElemImpl<JoinWhereElem> for Join {
  fn push_to_elems(&mut self, elem: JoinWhereElem) {
    self.wheres.push(elem)
  }
}
impl_quickrw_not_e!(
  [dmref],            // MANDATORY ATTRIBUTES
  [sourceref],        // OPTIONAL ATTRIBUTES
  "JOIN",             // TAG, here : <INSTANCE>
  Join,               // Struct on which to impl
  (),                 // Context type
  [],                 // Ordered elements
  read_join_sub_elem, // Sub elements reader
  [wheres]            // Empty context writables
);

/*
    struct Join => pattern A & B (cannot be determined from context)
    @elem dmref Option<String>: Modeled node related => OPT
    @elem sourceref Option<String>: Reference of the TEMPLATES or COLLECTION to be joined with. => OPT
    @elem wheres: Join conditions
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SrcJoin {
  #[serde(skip_serializing_if = "Option::is_none")]
  dmref: Option<String>,
  sourceref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  wheres: Vec<JoinWhereElem>,
}
impl SrcJoin {
  impl_non_empty_new!([sourceref], [dmref], [wheres]);
  impl_builder_opt_string_attr!(dmref);
}
impl ElemImpl<JoinWhereElem> for SrcJoin {
  fn push_to_elems(&mut self, elem: JoinWhereElem) {
    self.wheres.push(elem)
  }
}
impl_quickrw_not_e!(
  [sourceref],        // MANDATORY ATTRIBUTES
  [dmref],            // OPTIONAL ATTRIBUTES
  "JOIN",             // TAG, here : <INSTANCE>
  SrcJoin,            // Struct on which to impl
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
    @generic T: QuickXmlReadWrite + ElemImpl<JoinWhereElem>, a struct implementing QuickXmlReadWrite and ElemImpl on JoinWhereElems
    @param instance &mut T: an instance of T (here: Join or SrcJoin)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_join_sub_elem<R: std::io::BufRead, T: QuickXmlReadWrite + ElemImpl<JoinWhereElem>>(
  join: &mut T,
  _context: &(),
  mut reader: quick_xml::Reader<R>,
  reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) => {
        return Err(VOTableError::UnexpectedStartTag(
          e.local_name().to_vec(),
          T::TAG,
        ))
      }
      Event::Empty(ref e) => match e.local_name() {
        Where::TAG_BYTES => {
          if e
            .attributes()
            .any(|attribute| attribute.as_ref().unwrap().key == "foreignkey".as_bytes())
          {
            join.push_to_elems(JoinWhereElem::Where(Where::from_event_empty(e)?))
          } else {
            join.push_to_elems(JoinWhereElem::NoFkWhere(NoFkWhere::from_event_empty(e)?))
          }
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

#[cfg(test)]
mod tests {
  use crate::{
    mivot::{
      join::{Join, SrcJoin},
      test::{get_xml, test_error},
    },
    tests::test_read,
  };

  #[test]
  fn test_join_read() {
    // OK JOINS
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.1.xml");
    println!("testing 9.1");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.4.xml");
    println!("testing 9.4");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.5.xml");
    println!("testing 9.5");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.6.xml");
    println!("testing 9.6");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.9.xml");
    println!("testing 9.9");
    test_read::<SrcJoin>(&xml);

    // KO JOINS
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.2.xml");
    println!("testing 9.2"); // no dmref + sourceref must come with a dmref
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.3.xml");
    println!("testing 9.3"); // must have dmref or sourceref
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.3.xml");
    println!("testing 9.3"); // must have dmref or sourceref
    test_error::<SrcJoin>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.7.xml");
    println!("testing 9.7"); // dmref must not be empty
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.8.xml");
    println!("testing 9.8"); // sourceref must not be empty
    test_error::<SrcJoin>(&xml, false);
  }
}
