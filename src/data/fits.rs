//! Module dedicated to the `FITS` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use super::{
  super::{
    error::VOTableError,
    impls::mem::VoidTableDataContent,
    utils::{discard_comment, discard_event, unexpected_attr_warn},
    HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement,
    VOTableVisitor,
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
  pub fn set_stream(mut self, stream: Stream<VoidTableDataContent>) -> Self {
    self.set_stream_by_ref(stream);
    self
  }
  pub fn set_stream_by_ref(&mut self, stream: Stream<VoidTableDataContent>) {
    self.stream = stream;
  }

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

impl VOTableElement for Fits {
  const TAG: &'static str = "FITS";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new().set_attrs(attrs)
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "extnum" => self.set_extnum_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?),
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(extnum) = &self.extnum {
      f("extnum", extnum.to_string().as_str());
    }
  }
}

impl HasSubElements for Fits {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    false
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

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self.stream.write(writer, context)?;
    Ok(())
  }
}
