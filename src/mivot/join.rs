use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::events::attributes::Attributes;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use std::str;

use super::r#where::Where;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Join {
  #[serde(skip_serializing_if = "Option::is_none")]
  sourceref: Option<String>,
  dmref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  wheres: Vec<Where>,
}
impl Join {
    impl_non_empty_new!([dmref], [sourceref], [wheres]);
    impl_builder_opt_string_attr!(sourceref);
}
impl_quickrw_not_e!(
    [dmref],            // MANDATORY ATTRIBUTES
    [sourceref],        // OPTIONAL ATTRIBUTES
    "JOIN",             // TAG, here : <INSTANCE>
    Join,               // Struct on which to impl
    (),                 // Context type
    [wheres],           // Ordered elements
    read_join_sub_elem, // Sub elements reader
    []                  // Empty context writables
);

///////////////////////
// UTILITY FUNCTIONS //

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
                Where::TAG_BYTES => join.wheres.push(Where::from_event_empty(e)?),
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
