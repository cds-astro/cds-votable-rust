use super::{foreignkey::ForeignKey, ReferenceType};
use crate::{error::VOTableError, mivot::value_checker, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};
use std::str;

/*
    struct StaticRef
    @elem dmrole String: role of the referenced INSTANCE or COLLECTION in the DM => MAND
    @elem dmref Option<String>: @dmid of the referenced INSTANCE or COLLECTION => NO or MAND mutually exclusive with sourceref
    @elem sourceref Option<String>: @dmid of the COLLECTION or TEMPLATES to be searched in dynamic reference case => NO or MAND mutually exclusive with dmref
    @elem foreign_keys: Foreign key to be used to resolve a dynamic reference => MAND
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StaticRef {
  pub dmrole: String,
  pub dmref: String,
}
impl StaticRef {
  impl_empty_new!([dmrole, dmref], []);
  impl_builder_mand_string_attr!(dmrole);
  impl_builder_mand_string_attr!(dmref);
}
impl_quickrw_e! {
  [dmrole, dmref],
  [],
  "REFERENCE",
  StaticRef,
  ()
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
  pub dmrole: String,
  pub sourceref: String,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub foreign_keys: Vec<ForeignKey>,
}
impl DynRef {
  impl_non_empty_new!([dmrole, sourceref], [], [foreign_keys]);
  impl_builder_mand_string_attr!(dmrole);
  impl_builder_mand_string_attr!(sourceref);
}
impl ReferenceType for DynRef {
  fn push_fk(&mut self, fk: ForeignKey) {
    self.foreign_keys.push(fk);
  }
}
impl QuickXmlReadWrite for DynRef {
  const TAG: &'static str = "REFERENCE";
  type Context = ();

  impl_builder_from_attr!([dmrole, sourceref], []);

  non_empty_read_sub!(read_dynref_sub_elem);

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
    function read_dynref_sub_elem
    Description:
    *   reads the children of a dynamic reference
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @param instance &mut Reference: an instance of Reference
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_dynref_sub_elem<R: std::io::BufRead>(
  reference: &mut DynRef,
  _context: &(),
  mut reader: quick_xml::Reader<R>,
  reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Empty(ref e) => match e.local_name() {
        ForeignKey::TAG_BYTES => reference.push_fk(ForeignKey::from_event_empty(e)?),
        _ => {
          return Err(VOTableError::UnexpectedEmptyTag(
            e.local_name().to_vec(),
            DynRef::TAG,
          ))
        }
      },
      Event::End(e) if e.local_name() == DynRef::TAG_BYTES => break,
      Event::Eof => return Err(VOTableError::PrematureEOF(DynRef::TAG)),
      _ => eprintln!("Discarded event in {}: {:?}", DynRef::TAG, event),
    }
  }
  if reference.foreign_keys.is_empty() {
    Err(VOTableError::Custom(
      "A dynamic reference should have 1 or more foreign keys".to_owned(),
    ))
  } else {
    Ok(reader)
  }
}

#[cfg(test)]
mod tests {
  use std::str::from_utf8;

  use crate::{
    mivot::test::test_error,
    mivot::{reference::DynRef, reference::StaticRef, test::get_xml},
    tests::test_read,
  };

  #[test]
  fn test_staticref_read() {
    println!(
      "{:?}",
      from_utf8(&[115, 111, 117, 114, 99, 101, 114, 101, 102])
    );
    // OK VODMLS
    let xml = get_xml("./resources/mivot/6/test_6_ok_6.1.xml");
    println!("testing 6.1");
    test_read::<StaticRef>(&xml);

    // KO VODMLS
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.3.xml");
    println!("testing 6.3"); // valid dmrole + no dmref + no tableref; must have one of (dmref, sourceref)
    test_error::<StaticRef>(&xml, false);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.4.xml");
    println!("testing 6.4"); // valid dmrole + dmref + sourceref; must have dmref or sourceref, not both
    test_error::<StaticRef>(&xml, true);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.5.xml");
    println!("testing 6.5"); // no dmrole + dmref; must have non-empty dmrole
    test_error::<StaticRef>(&xml, false);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.7.xml");
    println!("testing 6.7"); // valid dmrole + dmref + FK; with dmref, must not contain FOREIGN_KEY
    test_error::<StaticRef>(&xml, false);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.9.xml");
    println!("testing 6.9"); // empty dmrole + dmref; must have non-empty dmrole in this context
    test_error::<StaticRef>(&xml, false);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.11.xml");
    println!("testing 6.11"); // dmrole + empty dmref; if dmref, must not be empty
    test_error::<StaticRef>(&xml, false);
  }

  #[test]
  fn test_dynref_read() {
    // OK VODMLS
    let xml = get_xml("./resources/mivot/6/test_6_ok_6.2.xml");
    println!("testing 6.2");
    test_read::<DynRef>(&xml);
    let xml = get_xml("./resources/mivot/6/test_6_ok_6.8.xml");
    println!("testing 6.8");
    test_read::<DynRef>(&xml);

    // KO VODMLS
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.3.xml");
    println!("testing 6.3"); // valid dmrole + no dmref + no tableref; must have one of (dmref, sourceref)
    test_error::<DynRef>(&xml, false);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.4.xml");
    println!("testing 6.4"); // valid dmrole + dmref + sourceref; must have dmref or sourceref, not both
    test_error::<DynRef>(&xml, true);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.6.xml");
    println!("testing 6.6"); // valid dmrole + sourceref + no FK;  with sourceref, must have one or more FOREIGN_KEY
    test_error::<DynRef>(&xml, true);
    let xml = get_xml("./resources/mivot/6/test_6_ko_6.10.xml");
    println!("testing 6.10"); // dmrole + empty sourceref; if sourceref, must not be empty
    test_error::<DynRef>(&xml, true);
  }
}
