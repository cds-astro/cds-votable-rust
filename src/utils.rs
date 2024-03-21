use std::io::BufRead;

use crate::VOTableError;
use log::{debug, info, warn};
use quick_xml::{
  events::{BytesText, Event},
  Reader,
};

pub(crate) fn is_empty(text: &BytesText) -> bool {
  for byte in text.escaped() {
    if !u8::is_ascii_whitespace(byte) {
      return false;
    }
  }
  true
}

pub(crate) fn discard_comment<R: BufRead>(comment: &BytesText, reader: &Reader<R>, tag: &str) {
  if let Ok(comment) = comment.unescape_and_decode(reader) {
    info!("Discarded comment in tag {}: {}", tag, comment)
  } else {
    warn!("Discarded undecoded comment in tag {}: {:?}", tag, comment)
  }
}

pub(crate) fn discard_event(event: Event, tag: &str) {
  debug!("Discarded event in tag {}: {:?}", tag, event)
}

pub(crate) fn unexpected_event(event: Event, tag: &str) -> VOTableError {
  VOTableError::Custom(format!("Unexpected event in tag {}: {:?}", tag, event))
}

pub(crate) fn unexpected_attr_warn(attr_key: &str, tag: &str) {
  warn!(
    "Unexpected attribute in tag {}: '{:?}' is discarded",
    tag, attr_key
  )
}

#[cfg(feature = "mivot")]
pub(crate) fn unexpected_attr_err(attr_key: &str, tag: &'static str) -> VOTableError {
  VOTableError::UnexpectedAttr(attr_key.as_bytes().to_vec(), tag)
}
