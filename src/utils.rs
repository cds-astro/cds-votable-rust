use std::io::BufRead;

use crate::VOTableError;
use log::{debug, info, warn};
use quick_xml::{
  Reader,
  escape::{EscapeError, resolve_predefined_entity},
  events::{BytesRef, BytesText, Event},
};

pub(crate) fn is_empty(text: &BytesText) -> bool {
  text.bytes().all(|byte| byte.is_ascii_whitespace())
}

/// Push the text corresponding to a reference (e.g. `&lt;` or `&#38;`) into the given string.
/// Only character references and the XML predefined entities are supported.
pub(crate) fn push_general_ref(content: &mut String, e: &BytesRef) -> Result<(), VOTableError> {
  if let Some(c) = e.resolve_char_ref().map_err(VOTableError::Read)? {
    content.push(c);
  } else if let Some(s) = resolve_predefined_entity(e) {
    content.push_str(s);
  } else {
    return Err(VOTableError::Read(quick_xml::Error::Escape(
      EscapeError::UnrecognizedEntity(0..e.len(), e.to_string()),
    )));
  }
  Ok(())
}

pub(crate) fn discard_comment<R: BufRead>(comment: &BytesText, _reader: &Reader<R>, tag: &str) {
  info!("Discarded comment in tag {}: {}", tag, comment.as_ref())
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
