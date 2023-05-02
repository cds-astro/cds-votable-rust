
use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use paste::paste;

use super::{
  QuickXmlReadWrite,
  coosys::CooSys,
  param::Param,
  error::VOTableError,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "def_type")]
pub enum DefinitionsElem {
  CooSys(CooSys),
  Param(Param),
}

impl DefinitionsElem {
  
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      DefinitionsElem::CooSys(elem) => elem.write(writer, &()),
      DefinitionsElem::Param(elem) => elem.write(writer, &()),
    }
  }
}

/// Deprecated since VOTable 1.1, see 
/// [IVOA doc](https://www.ivoa.net/documents/VOTable/20040811/REC-VOTable-1.1-20040811.html#ToC19)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Definitions {
  // no attributes
  // sub-elems
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  elems: Vec<DefinitionsElem>,
}

impl Default for Definitions {
  
  fn default() -> Self {
    Definitions::new()
  }
  
}

impl Definitions {

  pub fn new() -> Self {
    Self {
      elems: Default::default()
    }
  }

  impl_builder_push_elem!(Param, DefinitionsElem);
  impl_builder_push_elem!(CooSys, DefinitionsElem);
  
}



impl QuickXmlReadWrite for Definitions {
  const TAG: &'static str = "DEFINITIONS";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let definitions = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      /*let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      definitions = match attr.key {
        b"ID" => definitions.set_id(value),
        _ => { return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)); }
      }*/
      return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
    }
    Ok(definitions)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            CooSys::TAG_BYTES => self.elems.push(DefinitionsElem::CooSys(from_event_start!(CooSys, reader, reader_buff, e))),
            Param::TAG_BYTES => self.elems.push(DefinitionsElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            CooSys::TAG_BYTES => self.elems.push(DefinitionsElem::CooSys(CooSys::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(DefinitionsElem::Param(Param::from_event_empty(e)?)),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    todo!()
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    if self.elems.is_empty() {
      let elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag
      writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem_vec_no_context!(self, elems, writer);
      // Close tag
      writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
    }
  }
}
