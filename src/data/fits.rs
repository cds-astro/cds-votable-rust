//! Module dedicated to the `FITS` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use crate::impls::mem::VoidTableDataContent;

use super::{
  super::{
    error::VOTableError,
    utils::{discard_comment, discard_event},
    QuickXmlReadWrite, TableDataContent, VOTableVisitor,
  },
  stream::Stream,
};

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Fits {
  // attribute
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extnum: Option<u32>,
  // I assume (so far) that for FITS there is no STREAM content (but a link pointing to the FITS file)
  pub stream: Stream<VoidTableDataContent>,
}

impl Fits {
  pub fn new() -> Self {
    Self::default()
  }

  impl_builder_opt_attr!(extnum, u32);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_fits_start(self)?;
    visitor.visit_fits_stream(&mut self.stream)?;
    visitor.visit_fits_ended(self)
  }
}

impl QuickXmlReadWrite for Fits {
  const TAG: &'static str = "FITS";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut fits = Self::default();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      fits = match attr.key {
        b"extnum" => fits.set_extnum(value.parse().map_err(VOTableError::ParseInt)?),
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    Ok(fits)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, context)
      .map(|()| reader)
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.name() {
          Stream::<VoidTableDataContent>::TAG_BYTES => {
            self.stream = from_event_start_by_ref!(Stream, reader, reader_buff, e)
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.name() {
          Stream::<VoidTableDataContent>::TAG_BYTES => self.stream = Stream::from_event_empty(e)?,
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(()),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_tostring_attr!(self, tag, extnum);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    self.stream.write(writer, &())?;
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}
