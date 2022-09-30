
use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use serde;
use paste::paste;


use super::{
  stream::Stream,
  super::{
    QuickXmlReadWrite, TableDataContent,
    error::VOTableError,
    table::TableElem
  }
};

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Void;

impl TableDataContent for Void {
  fn read_datatable_content<R: BufRead>(&mut self, _reader: Reader<R>, _reader_buff: &mut Vec<u8>, _context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    unreachable!()
  }

  fn read_binary_content<R: BufRead>(&mut self, _reader: Reader<R>, _reader_buff: &mut Vec<u8>, _context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    unreachable!()
  }

  fn read_binary2_content<R: BufRead>(&mut self, _reader: Reader<R>, _reader_buff: &mut Vec<u8>, _context: &[TableElem]) -> Result<Reader<R>, VOTableError> {
    unreachable!()
  }

  fn write_in_datatable<W: Write>(&mut self, _writer: &mut Writer<W>, _context: &[TableElem]) -> Result<(), VOTableError> {
    unreachable!()
  }

  fn write_in_binary<W: Write>(&mut self, _writer: &mut Writer<W>, _context: &[TableElem]) -> Result<(), VOTableError> {
    unreachable!()
  }

  fn write_in_binary2<W: Write>(&mut self, _writer: &mut Writer<W>, _context: &[TableElem]) -> Result<(), VOTableError> {
    unreachable!()
  }
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Fits {
  // attribute
  #[serde(skip_serializing_if = "Option::is_none")]
  extnum: Option<u32>,
  // I assume (so far) that for FITS there is no STREAM content (but a link pointing to the FITS file)
  stream: Stream<Void>,
}

impl Fits {
  pub fn new() -> Self {
    Self::default()
  }

  impl_builder_opt_attr!(extnum, u32);

}


impl QuickXmlReadWrite for Fits {
  const TAG: &'static str = "FITS";
  type Context = Vec<TableElem>;

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut fits = Self::default();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      fits = match attr.key {
        b"extnum" => fits.set_extnum(value.parse().map_err(VOTableError::ParseInt)?),
        _ => { return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)); },
      }
    }
    Ok(fits)
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
          match e.name() {
            Stream::<Void>::TAG_BYTES => self.stream = from_event_start!(Stream, reader, reader_buff, e),
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.name() {
            Stream::<Void>::TAG_BYTES => self.stream = Stream::from_event_empty(e)?,
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_tostring_attr!(self, tag, extnum);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-elements
    self.stream.write(writer, &())?;
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}