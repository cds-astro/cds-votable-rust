//! Module dedicated to the `BINARY2` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{
  events::{BytesEnd, BytesStart, Event},
  Reader, Writer,
};

use super::{
  super::{
    error::VOTableError,
    table::TableElem,
    utils::{discard_comment, discard_event, is_empty, unexpected_attr_warn},
    QuickXmlReadWrite, TableDataContent, VOTableElement,
  },
  stream::Stream,
};

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Binary2<C: TableDataContent> {
  pub stream: Stream<C>,
}

impl<C: TableDataContent> Binary2<C> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_stream(stream: Stream<C>) -> Self {
    Self { stream }
  }

  pub(crate) fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    writer
      .write_event(Event::Start(BytesStart::borrowed_name(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)
      .and_then(|()| {
        self
          .stream
          .write_start(writer)
          .and_then(|()| writer.write(b"\n").map_err(VOTableError::Write))
      })
  }

  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    //if self.stream.content.is_some() {
    self.stream.write_end(writer).and_then(|()| {
      writer
        .write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES)))
        .map_err(VOTableError::Write)
    })
  }
}

impl<C: TableDataContent> VOTableElement for Binary2<C> {
  const TAG: &'static str = "BINARY2";

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
    for (k, _) in attrs {
      unexpected_attr_warn(k.as_ref(), Self::TAG);
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, _f: F)
  where
    F: FnMut(&str, &str),
  {
  }

  fn has_no_sub_elements(&self) -> bool {
    false
  }
}

impl<C: TableDataContent> QuickXmlReadWrite for Binary2<C> {
  type Context = Vec<TableElem>;

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.name() {
          Stream::<C>::TAG_BYTES => {
            // We could detect if current stream.content.is_some() to prevent from multi-stream...
            let mut stream = Stream::<C>::from_attributes(e.attributes())?;
            let mut content = C::new();
            content.read_binary2_content(reader, reader_buff, context)?;
            stream.content = Some(content);
            self.stream = stream;
            // the next call is a failure (because we consume </STREAM> in read_binary_content)
            let tmp_reader = reader.check_end_names(false);
            loop {
              let mut event = tmp_reader
                .read_event(reader_buff)
                .map_err(VOTableError::Read)?;
              match &mut event {
                Event::Text(e) if is_empty(e) => {}
                Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(()),
                Event::Comment(e) => discard_comment(e, tmp_reader, Self::TAG),
                _ => discard_event(event, Self::TAG),
              }
            }
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.name() {
          Stream::<C>::TAG_BYTES => self.stream = Stream::<C>::from_event_empty(e)?,
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self
      .write_to_data_beginning(writer)
      .and_then(|()| {
        self
          .stream
          .content
          .as_mut()
          .unwrap()
          .write_in_binary2(writer, context)
      })
      .and_then(|()| self.write_from_data_end(writer))
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    _writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    unreachable!()
  }
}
