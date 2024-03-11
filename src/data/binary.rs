//! Module dedicated to the `BINARY` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use log::warn;
use quick_xml::{
  events::{attributes::Attributes, BytesEnd, BytesStart, Event},
  Reader, Writer,
};

use super::{
  super::{
    error::VOTableError,
    table::TableElem,
    utils::{discard_comment, discard_event, is_empty},
    QuickXmlReadWrite, TableDataContent,
  },
  stream::Stream,
};

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Binary<C: TableDataContent> {
  pub stream: Stream<C>,
}

impl<C: TableDataContent> Binary<C> {
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
        //if self.stream.content.is_some() {
        self
          .stream
          .write_start(writer)
          .and_then(|()| writer.write(b"\n").map_err(VOTableError::Write))
        /*} else {
          self.stream.write(writer, &())
        }*/
      })
  }

  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    //if self.stream.content.is_some() {
    self
      .stream
      .write_end(writer)
      /*} else {
        self.stream.write(writer, &())
      }*/
      .and_then(|()| {
        writer
          .write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES)))
          .map_err(VOTableError::Write)
      })
  }
}

impl<C: TableDataContent> QuickXmlReadWrite for Binary<C> {
  const TAG: &'static str = "BINARY";
  type Context = Vec<TableElem>;

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let data = Self::new();
    if attrs.count() > 0 {
      warn!(
        "No attribute expected in {}: attribute(s) ignored.",
        Self::TAG
      );
    }
    Ok(data)
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
            content.read_binary_content(reader, reader_buff, context)?;
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
    /*writer
      .write_event(Event::Start(BytesStart::borrowed_name(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)?;
    if self.stream.content.is_some() {
      self.stream.write_start(writer)?;
      writer.write(b"\n").map_err(VOTableError::Write)?;
      let content = self.stream.content.as_mut().unwrap();
      content.write_in_binary(writer, context)?;
      // self.content.write_in_datatable(&mut writer)?;
      self.stream.write_end(writer)?;
    } else {
      self.stream.write(writer, &())?;
    }
    writer
      .write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)*/
    self
      .write_to_data_beginning(writer)
      .and_then(|()| {
        self
          .stream
          .content
          .as_mut()
          .unwrap()
          .write_in_binary(writer, context)
      })
      .and_then(|()| self.write_from_data_end(writer))
  }
}
