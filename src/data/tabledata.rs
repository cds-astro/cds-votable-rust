//! Module dedicated to the `TABLEDATA` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{
  events::{BytesEnd, BytesStart, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError, table::TableElem, utils::*, QuickXmlReadWrite, SpecialElem,
  TableDataContent, VOTableElement,
};

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TableData<C: TableDataContent> {
  #[serde(flatten)]
  pub content: C,
}

/// According to the context, we may parse a TABLEDATA row till we found `</TR>`,
/// or, if `</TR>` has already been parsed (iteration on pre-parsed rows), till `EOF` is found.
/// In the first case, finding `EOF` is an error while it is expected in the second case.
/// Hence this trait with two implementers to control the behaviours of the method `parse_fields`
/// when those two events are found.
pub trait IsRowEnd {
  /// Possibly raised an error if `EOF` is found.
  fn eof() -> Result<(), VOTableError>;
  /// Possibly raised an error if `</TR>` is found.
  fn tr_end() -> Result<(), VOTableError>;
}

/// Stands for 'End Of TR', i.e. we found the `</TR>` tag.
pub enum EOTR {}
impl IsRowEnd for EOTR {
  fn eof() -> Result<(), VOTableError> {
    Err(VOTableError::PrematureEOF("TR or TD"))
  }
  fn tr_end() -> Result<(), VOTableError> {
    Ok(())
  }
}

/// Stands for `End Of File`, i.e. no more data to ead.
pub enum EOF {}
impl IsRowEnd for EOF {
  fn eof() -> Result<(), VOTableError> {
    Ok(())
  }

  fn tr_end() -> Result<(), VOTableError> {
    Err(VOTableError::UnexpectedEndTag(b"TR".to_vec(), "TR or TD"))
  }
}

/// Parse oll successive '<TD></TD>' tag till either:
/// * we find the `EOF` event in case of `parse_field::<_, EOF>`
/// * we find the `</TR>` event in case of `parse_field::<_, EOTR>`
pub fn parse_fields<R: BufRead, I: IsRowEnd>(
  reader: &mut Reader<R>,
  reader_buff: &mut Vec<u8>,
  n_fields: usize,
) -> Result<Vec<String>, VOTableError> {
  let mut row: Vec<String> = Vec::with_capacity(n_fields);
  loop {
    let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
    match &mut event {
      Event::Start(ref e) if e.local_name() == b"TD" => {
        let mut field = String::new();
        loop {
          let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
          match &mut event {
            Event::Text(e) => {
              let raw_txt = e.unescaped().map_err(VOTableError::Read)?;
              field.push_str(unsafe { str::from_utf8_unchecked(raw_txt.as_ref()) });
            }
            Event::CData(e) => field.push_str(unsafe { str::from_utf8_unchecked(e.as_ref()) }),
            Event::End(e) if e.local_name() == b"TD" => {
              row.push(field);
              reader_buff.clear();
              break;
            }
            Event::Eof => return Err(VOTableError::PrematureEOF("TD")),
            Event::Comment(e) => discard_comment(e, reader, "TD"),
            _ => return Err(unexpected_event(event, "TD")),
          }
        }
      }
      Event::Empty(e) if e.local_name() == b"TD" => row.push(String::new()),
      Event::End(e) if e.local_name() == b"TR" => return I::tr_end().map(|_| row),
      Event::Eof => return I::eof().map(|_| row),
      Event::Text(e) if is_empty(e) => {}
      Event::Comment(e) => discard_comment(e, reader, "TD"),
      _ => return Err(unexpected_event(event, "TR")),
    }
  }
}

/// Iterator on the '<TD></TD>' content:
/// # Pre-requisite
/// * the tag '<TR>' must have already been parsed
/// # Stops when
/// * '</TR>' is found
pub struct FieldIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
}

impl<'a, R: BufRead> FieldIterator<'a, R> {
  pub fn new(reader: &'a mut Reader<R>, reader_buff: &'a mut Vec<u8>) -> Self {
    Self {
      reader,
      reader_buff,
    }
  }
}

impl<'a, R: BufRead> Iterator for FieldIterator<'a, R> {
  type Item = Result<String, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff);
      match event {
        Err(e) => return Some(Err(VOTableError::Read(e))),
        Ok(mut event) => match &mut event {
          Event::Start(ref e) if e.local_name() == b"TD" => {
            let mut field = String::new();
            loop {
              let event = self.reader.read_event(self.reader_buff);
              match event {
                Err(e) => return Some(Err(VOTableError::Read(e))),
                Ok(mut event) => match &mut event {
                  Event::Text(e) => match e.unescaped() {
                    Err(e) => return Some(Err(VOTableError::Read(e))),
                    Ok(raw_txt) => {
                      field.push_str(unsafe { str::from_utf8_unchecked(raw_txt.as_ref()) })
                    }
                  },
                  Event::CData(e) => {
                    field.push_str(unsafe { str::from_utf8_unchecked(e.as_ref()) })
                  }
                  Event::End(e) if e.local_name() == b"TD" => {
                    self.reader_buff.clear();
                    return Some(Ok(field));
                  }
                  Event::Eof => return Some(Err(VOTableError::PrematureEOF("TD"))),
                  Event::Comment(e) => discard_comment(e, self.reader, "TD"),
                  _ => return Some(Err(unexpected_event(event, "TD"))),
                },
              }
            }
          }
          Event::Empty(e) if e.local_name() == b"TD" => return Some(Ok(String::new())),
          Event::End(e) if e.local_name() == b"TR" => return None,
          Event::Eof => return Some(Err(VOTableError::PrematureEOF("TR"))),
          Event::Text(e) if is_empty(e) => {}
          Event::Comment(e) => discard_comment(e, self.reader, "TD"),
          _ => return Some(Err(unexpected_event(event, "TR"))),
        },
      }
    }
  }
}

/// Same as 'FieldIterator' but from a slice (hence, no buffering needed) contaniing niehter '<TR>'
/// nor '</TR>' (so ends with EOF).
pub struct FieldIteratorUnbuffered<'a> {
  reader: Reader<&'a [u8]>,
}

impl<'a> FieldIteratorUnbuffered<'a> {
  /// Slice going from '<TR>' (exclusive) to '</TR>' (exclusive).
  pub fn new(raw_row: &'a [u8]) -> Self {
    Self {
      reader: Reader::from_bytes(raw_row),
    }
  }
}

impl<'a> Iterator for FieldIteratorUnbuffered<'a> {
  type Item = Result<String, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event_unbuffered();
      match event {
        Err(e) => return Some(Err(VOTableError::Read(e))),
        Ok(mut event) => match &mut event {
          Event::Start(ref e) if e.local_name() == b"TD" => {
            let mut field = String::new();
            loop {
              let event = self.reader.read_event_unbuffered();
              match event {
                Err(e) => return Some(Err(VOTableError::Read(e))),
                Ok(mut event) => match &mut event {
                  Event::Text(e) => match e.unescaped() {
                    Err(e) => return Some(Err(VOTableError::Read(e))),
                    Ok(raw_txt) => {
                      field.push_str(unsafe { str::from_utf8_unchecked(raw_txt.as_ref()) })
                    }
                  },
                  Event::CData(e) => {
                    field.push_str(unsafe { str::from_utf8_unchecked(e.as_ref()) })
                  }
                  Event::End(e) if e.local_name() == b"TD" => {
                    return Some(Ok(field));
                  }
                  Event::Eof => return Some(Err(VOTableError::PrematureEOF("TD"))),
                  Event::Comment(e) => discard_comment(e, &self.reader, "TD"),
                  _ => return Some(Err(unexpected_event(event, "TD"))),
                },
              }
            }
          }
          Event::Empty(e) if e.local_name() == b"TD" => return Some(Ok(String::new())),
          Event::Eof => return None,
          Event::Text(e) if is_empty(e) => {}
          Event::Comment(e) => discard_comment(e, &self.reader, "TD"),
          _ => return Some(Err(unexpected_event(event, "TR"))),
        },
      }
    }
  }
}

impl<C: TableDataContent> TableData<C> {
  pub fn new(content: C) -> Self {
    Self { content }
  }

  pub(crate) fn ensures_consistency(&mut self, context: &[TableElem]) -> Result<(), String> {
    self.content.ensures_consistency(context)
  }

  pub(crate) fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    writer
      .write_event(Event::Start(BytesStart::borrowed_name(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)
  }

  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
  ) -> Result<(), VOTableError> {
    writer
      .write_event(Event::End(BytesEnd::borrowed(Self::TAG_BYTES)))
      .map_err(VOTableError::Write)
  }
}

impl<C: TableDataContent> VOTableElement for TableData<C> {
  const TAG: &'static str = "TABLEDATA";

  type MarkerType = SpecialElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::default().set_attrs(attrs)
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
}

impl<C: TableDataContent> QuickXmlReadWrite<SpecialElem> for TableData<C> {
  type Context = Vec<TableElem>;

  fn read_content_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self
      .content
      .read_datatable_content(reader, reader_buff, context)
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    self
      .write_to_data_beginning(writer)
      .and_then(|()| self.content.write_in_datatable(writer, context))
      .and_then(|()| self.write_from_data_end(writer))
  }
}
