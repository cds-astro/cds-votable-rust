
use std::{str, io::{BufRead, Write}};

use quick_xml::{
  Reader, Writer,
  events::{Event, attributes::Attributes}
};
use quick_xml::events::{BytesEnd, BytesStart};

use serde;

use super::{
  super::{
    is_empty,
    QuickXmlReadWrite, TableDataContent,
    error::VOTableError,
    table::TableElem
  },
  stream::Stream,
};

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Binary2<C: TableDataContent> {
  stream: Stream<C>
}

impl<C: TableDataContent> Binary2<C> {
  
  pub fn new() -> Self {
    Self::default()
  }

  pub fn from_stream(stream: Stream<C>) -> Self {
    Self { stream }
  }
  
}

impl<C: TableDataContent> QuickXmlReadWrite for Binary2<C> {
  const TAG: &'static str = "BINARY2";
  type Context = Vec<TableElem>;

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let binary2 = Self::new();
    if attrs.count() > 0 {
      eprintln!("No attribute expected in {}: attribute(s) ignored.", Self::TAG);
    }
    Ok(binary2)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) =>
          match e.name() {
            Stream::<C>::TAG_BYTES => {
              // We could detect if current stream.content.is_some() to prevent from multi-stream...
              let mut stream = Stream::<C>::from_attributes(e.attributes())?;
              let mut content = C::new();
              reader = content.read_binary2_content(reader, reader_buff, context)?;
              stream.content = Some(content);
              self.stream = stream;
              // the next call is a failure (because we consume </STREAM> in read_binary_content)
              let tmp_reader = reader.check_end_names(false);
              loop {
                let mut event = tmp_reader.read_event(reader_buff).map_err(VOTableError::Read)?;
                match &mut event {
                  Event::Text(e) if is_empty(e) => { },
                  Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(reader),
                  _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
                }
              }
            },
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        Event::Empty(ref e) =>
          match e.name() {
            Stream::<C>::TAG_BYTES => self.stream = Stream::<C>::from_event_empty(e)?,
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        Event::Text(e) if is_empty(e) => { },
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
  
  fn write<W: Write>(&mut self, writer: &mut Writer<W>, context: &Self::Context) -> Result<(), VOTableError> {
    writer.write_event(Event::Start(BytesStart::borrowed_name(Self::TAG_BYTES))).map_err(VOTableError::Write)?;
    if self.stream.content.is_some() {
      self.stream.write_start(writer)?;
      writer.write(b"\n").map_err(VOTableError::Write)?;
      let content = self.stream.content.as_mut().unwrap();
      content.write_in_binary2(writer, context)?;
      // self.content.write_in_datatable(&mut writer)?;     
      self.stream.write_end(writer)?;
    } else {
      self.stream.write(writer, &())?;
    }
    writer.write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES))).map_err(VOTableError::Write)
  }
}