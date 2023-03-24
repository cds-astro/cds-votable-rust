
use std::{str, io::BufRead};

use quick_xml::{Reader, events::Event};

use crate::{
  is_empty,
  // table::Table,
  error::VOTableError,
  // impls::mem::VoidTableDataContent
};

pub struct RowStringIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
  // table: &'a mut Table<VoidTableDataContent>,
}

impl<'a, R: BufRead> RowStringIterator<'a, R> {

  pub fn new(
    reader: &'a mut Reader<R>,
    reader_buff: &'a mut Vec<u8>,
    // table: &'a mut Table<VoidTableDataContent>
  ) -> Self {
    Self {
      reader,
      reader_buff,
      // table,
    }
  }

}

impl<'a, R: BufRead> Iterator for RowStringIterator<'a, R> {
  type Item = Result<Vec<String>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff).map_err(VOTableError::Read);
      match event {
        Err(e) => return Some(Err(e)),
        Ok(mut event) =>
          match &mut event {
            Event::Start(ref e) =>
              match e.local_name() {
                b"TR" => {
                  let fit = FieldStringIterator { reader: self.reader, reader_buff: self.reader_buff};
                  let res = fit.collect::<Result<Vec<String>, VOTableError>>();
                  self.reader_buff.clear();
                  return Some(res);
                },
                _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Row Iterator: {:?}", event)))),
              }
            Event::End(e) =>
              match e.local_name() {
                b"TABLEDATA" => return None,
                _ => return Some(Err(VOTableError::Custom(format!("Unexpected end of tag in Row Iterator '{:?}'", str::from_utf8(e.local_name()))))),
              }
            Event::Eof => return Some(Err(VOTableError::Custom(String::from("Premature end of file in Row Iterator")))),
            Event::Text(e) if is_empty(e) => {},
            _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
          }
      }
    }
  }
}

pub struct FieldStringIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
}

impl<'a, R: BufRead> Iterator for FieldStringIterator<'a, R> {
  type Item = Result<String, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff).map_err(VOTableError::Read);
      match event {
        Err(e) => return Some(Err(e)),
        Ok(mut event) =>
          match &mut event {
            Event::Start(ref e) =>
              match e.local_name() {
                b"TD" => {
                  let event = self.reader.read_event(self.reader_buff).map_err(VOTableError::Read);
                  match event {
                    Err(e) => return Some(Err(e)),
                    Ok(Event::Text(e)) => return Some(e.unescape_and_decode(&self.reader).map_err(VOTableError::Read)),
                    _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
                  }
                }
                _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
              }
            Event::Empty(e) if e.local_name() == b"TD" => return Some(Ok(String::from(""))),
            Event::End(e) =>
              match e.local_name() {
                b"TD" => {},
                b"TR" => return None,
                _ => return Some(Err(VOTableError::Custom(format!("Unexpected end of tag in Field Iterator '{:?}'", str::from_utf8(e.local_name()))))),
              }
            Event::Eof => return Some(Err(VOTableError::Custom(String::from("Premature end of file in Field Iterator")))),
            Event::Text(e) if is_empty(e) => {},
            _ => return Some(Err(VOTableError::Custom(format!("Discarded event in Field Iterator: {:?}", event)))),
          }
      }
    }
  }
}
