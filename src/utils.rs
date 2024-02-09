use std::io::BufRead;

use log::{debug, info, warn};
use quick_xml::{
  events::{BytesText, Event},
  Reader,
};

pub(crate) fn is_empty(text: &BytesText) -> bool {
  for byte in text.escaped() {
    if *byte != b' ' && *byte != b'\n' && *byte != b'\t' && *byte != b'\r' {
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
