use super::{foreignkey::ForeignKey, ReferenceType};
use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};
use std::str;

trait Referencial {
  fn push2foreign(&mut self, _fk: ForeignKey) {}
}

/*
    struct StaticRef
    @elem dmrole String: role of the referenced INSTANCE or COLLECTION in the DM => MAND
    @elem dmref Option<String>: @dmid of the referenced INSTANCE or COLLECTION => NO or MAND mutually exclusive with sourceref
    @elem sourceref Option<String>: @dmid of the COLLECTION or TEMPLATES to be searched in dynamic reference case => NO or MAND mutually exclusive with dmref
    @elem foreign_keys: Foreign key to be used to resolve a dynamic reference => MAND
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StaticRef {
  dmrole: String,
  dmref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  foreign_keys: Vec<ForeignKey>,
}
impl StaticRef {
  impl_non_empty_new!([dmrole, dmref], [], [foreign_keys]);
}
impl ReferenceType for StaticRef {
  fn push2_fk(&mut self, fk: ForeignKey) {
    self.foreign_keys.push(fk);
  }
}
impl QuickXmlReadWrite for StaticRef {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

  impl_builder_from_attr!([dmrole, dmref], []);

  non_empty_read_sub!(read_ref_sub_elem);

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    if self.foreign_keys.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer = elem_writer.with_attribute(("dmrole", self.dmrole.as_str()));
      elem_writer = elem_writer.with_attribute(("dmref", self.dmref.as_str()));
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      tag.push_attribute(("dmrole", self.dmrole.as_str()));
      tag.push_attribute(("dmref", self.dmref.as_str()));
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

/*
    struct DynRef
    @elem dmrole String: role of the referenced INSTANCE or COLLECTION in the DM => MAND
    @elem dmref Option<String>: @dmid of the referenced INSTANCE or COLLECTION => NO or MAND mutually exclusive with sourceref
    @elem sourceref Option<String>: @dmid of the COLLECTION or TEMPLATES to be searched in dynamic reference case => NO or MAND mutually exclusive with dmref
    @elem foreign_keys: Foreign key to be used to resolve a dynamic reference => MAND
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DynRef {
  dmrole: String,
  sourceref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  foreign_keys: Vec<ForeignKey>,
}
impl DynRef {
  impl_non_empty_new!([dmrole, sourceref], [], [foreign_keys]);
}
impl ReferenceType for DynRef {
  fn push2_fk(&mut self, fk: ForeignKey) {
    self.foreign_keys.push(fk);
  }
}
impl QuickXmlReadWrite for DynRef {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

  impl_builder_from_attr!([dmrole, sourceref], []);

  non_empty_read_sub!(read_ref_sub_elem);

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    if self.foreign_keys.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer = elem_writer.with_attribute(("dmrole", self.dmrole.as_str()));
      elem_writer = elem_writer.with_attribute(("sourceref", self.sourceref.as_str()));
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      tag.push_attribute(("dmrole", self.dmrole.as_str()));
      tag.push_attribute(("sourceref", self.sourceref.as_str()));
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
    @param instance &mut Reference: an instance of Reference
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_ref_sub_elem<R: std::io::BufRead, T: QuickXmlReadWrite + ReferenceType>(
  reference: &mut T,
  _context: &(),
  mut reader: quick_xml::Reader<R>,
  reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Empty(ref e) => match e.local_name() {
        ForeignKey::TAG_BYTES => reference.push2_fk(ForeignKey::from_event_empty(e)?),
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            T::TAG,
          ))
        }
      },
      Event::End(e) if e.local_name() == T::TAG_BYTES => return Ok(reader),
      Event::Eof => return Err(VOTableError::PrematureEOF(T::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", T::TAG, event),
    }
  }
}
