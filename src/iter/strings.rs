//! Iterator on `TABLEDATA` rows in which a row is a `Vector` of `String`.
use std::io::BufRead;

use quick_xml::{events::Event, Reader};

use crate::{
  data::tabledata::FieldIterator,
  error::VOTableError,
  utils::{discard_comment, is_empty, unexpected_event},
};

pub struct RowStringIterator<'a, R: BufRead> {
  reader: &'a mut Reader<R>,
  reader_buff: &'a mut Vec<u8>,
}

impl<'a, R: BufRead> RowStringIterator<'a, R> {
  pub fn new(reader: &'a mut Reader<R>, reader_buff: &'a mut Vec<u8>) -> Self {
    Self {
      reader,
      reader_buff,
    }
  }
}

impl<'a, R: BufRead> Iterator for RowStringIterator<'a, R> {
  type Item = Result<Vec<String>, VOTableError>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      let event = self.reader.read_event(self.reader_buff);
      match event {
        Err(e) => return Some(Err(VOTableError::Read(e))),
        Ok(mut event) => match &mut event {
          Event::Start(ref e) if e.local_name() == b"TR" => {
            // TODO: we could avoid allocations by adding as attribute the number of FIELDs in the VOTable
            // (and using 'for' with 'push' in a pre-allocated Vec instead of collecting)!
            let res = FieldIterator::new(self.reader, self.reader_buff)
              .collect::<Result<Vec<String>, VOTableError>>();
            self.reader_buff.clear();
            return Some(res);
          }
          Event::End(e) if e.local_name() == b"TABLEDATA" => return None,
          Event::Eof => return Some(Err(VOTableError::PrematureEOF("TABLEDATA"))),
          Event::Text(e) if is_empty(e) => {}
          Event::Comment(e) => discard_comment(e, self.reader, "TABLEDATA"),
          _ => return Some(Err(unexpected_event(event, "TABLEDATA"))),
        },
      }
    }
  }
}
