use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::str;

use super::{instance::NoRoleInstance, r#where::Where};

/*
    struct Templates
    @elem tableref Option<String>:  => OPT
    @elem wheres:
    @elem instances:
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Templates {
    #[serde(skip_serializing_if = "Option::is_none")]
    tableref: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    wheres: Vec<Where>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    instances: Vec<NoRoleInstance>,
}
impl Templates {
  fn new() -> Self {
    Self {
      tableref: None,
      wheres: vec![],
      instances: vec![],
    }
  }
  impl_builder_opt_string_attr!(tableref);
}
impl_quickrw_not_e!(
    [],                     // MANDATORY ATTRIBUTES
    [tableref],             // OPTIONAL ATTRIBUTES
    "TEMPLATES",            // TAG, here : <INSTANCE>
    Templates,              // Struct on which to impl
    (),                     // Context type
    [wheres, instances],    // Ordered elements
    read_template_sub_elem, // Sub elements reader
    []                      // Empty context writables
);

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_template_sub_elem
    Description:
    *   reads the children of Templates
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @param instance &mut Templates: an instance of Templates
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_template_sub_elem<R: std::io::BufRead>(
    template: &mut Templates,
    _context: &(),
    mut reader: quick_xml::Reader<R>,
    mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
    loop {
        let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
            Event::Start(ref e) => match e.local_name() {
                NoRoleInstance::TAG_BYTES => template.instances.push(from_event_start!(
                    NoRoleInstance,
                    reader,
                    reader_buff,
                    e
                )),
                _ => {
                    return Err(VOTableError::UnexpectedStartTag(
                        e.local_name().to_vec(),
                        Templates::TAG,
                    ))
                }
            },
            Event::Empty(ref e) => match e.local_name() {
                Where::TAG_BYTES => template.wheres.push(Where::from_event_empty(e)?),
                _ => {
                    return Err(VOTableError::UnexpectedEmptyTag(
                        e.local_name().to_vec(),
                        Templates::TAG,
                    ))
                }
            },
            Event::Text(e) if is_empty(e) => {}
            Event::End(e) if e.local_name() == Templates::TAG_BYTES => return Ok(reader),
            Event::Eof => return Err(VOTableError::PrematureEOF(Templates::TAG)),
            _ => eprintln!("Discarded event in {}: {:?}", Templates::TAG, event),
        }
    }
}
