use super::foreignkey::ForeignKey;
use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Reference {
    dmrole: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dmref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sourceref: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    foreign_keys: Vec<ForeignKey>,
}
impl Reference {
    impl_non_empty_new!([dmrole], [sourceref, dmref], [foreign_keys]);
    impl_builder_opt_string_attr!(sourceref);
    impl_builder_opt_string_attr!(dmref);
}

impl QuickXmlReadWrite for Reference {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

    impl_builder_from_attr!([dmrole], [sourceref, dmref]);

    non_empty_read_sub!(read_ref_sub_elem);

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        if self.foreign_keys.is_empty() {
            let mut elem_writer = writer.create_element(Self::TAG_BYTES);
            elem_writer = elem_writer.with_attribute(("dmrole", self.dmrole.as_str()));
            write_empty_optional_attributes!(elem_writer, self, sourceref, dmref);
            elem_writer.write_empty().map_err(VOTableError::Write)?;
            Ok(())
        } else {
            let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
            // Write tag + attributes
            tag.push_attribute(("dmrole", self.dmrole.as_str()));
            write_non_empty_optional_attributes!(tag, self, sourceref, dmref);
            writer
                .write_event(Event::Start(tag.to_borrowed()))
                .map_err(VOTableError::Write)?;
            // Write sub-elems
            write_elem_vec!(self, foreign_keys, writer, context);
            // Close tag
            writer
                .write_event(Event::End(tag.to_end()))
                .map_err(VOTableError::Write)
        }
    }
  }

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_ref_sub_elem
    Description:
    *   reads the children of Reference
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<InstanceElem>; a struct that implements the quickXMLReadWrite and ElemImpl for InstanceElem traits.
    @param instance &mut T: an instance of T (here either GlobOrTempInstance or Reference)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_ref_sub_elem<R: std::io::BufRead>(
    reference: &mut Reference,
    _context: &(),
    mut reader: quick_xml::Reader<R>,
    reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
    loop {
        let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
            Event::Empty(ref e) => match e.local_name() {
                ForeignKey::TAG_BYTES => reference
                    .foreign_keys
                    .push(ForeignKey::from_event_empty(e)?),
                _ => {
                    return Err(VOTableError::UnexpectedEmptyTag(
                        e.local_name().to_vec(),
                        Reference::TAG,
                    ))
                }
            },
            Event::End(e) if e.local_name() == Reference::TAG_BYTES => return Ok(reader),
            Event::Eof => return Err(VOTableError::PrematureEOF(Reference::TAG)),
            _ => eprintln!("Discarded event in {}: {:?}", Reference::TAG, event),
        }
    }
}
