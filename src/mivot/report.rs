//! Module dedicated to the `REPORT` tag.

use std::str::{self, FromStr};

use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesText, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  mivot::VodmlVisitor,
  utils::{discard_comment, discard_event},
  QuickXmlReadWrite,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Status {
  OK,
  FAILED,
}
impl FromStr for Status {
  type Err = String;

  fn from_str(str: &str) -> Result<Self, Self::Err> {
    match str {
      "OK" => Ok(Self::OK),
      "FAILED" => Ok(Self::FAILED),
      _ => Err(format!(
        "Attribute 'status' error in 'REPORT'. Expected: either 'OK' or 'FAILED'. Actual: '{}'.",
        str
      )),
    }
  }
}

impl ToString for Status {
  fn to_string(&self) -> String {
    match self {
      Self::OK => "OK".to_owned(),
      Self::FAILED => "FAILED".to_owned(),
    }
  }
}

/// Structure storing the content of the `REPORT` tag.
/// Tells the client whether the annotation process succeeded or not.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Report {
  /// Status of the annotation process.
  pub status: Status,
  /// Report content.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
}

impl Report {
  pub fn new(status: Status) -> Self {
    Report {
      status,
      content: None,
    }
  }

  impl_builder_opt_string_attr!(content);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_report(self)
  }
}

impl QuickXmlReadWrite for Report {
  const TAG: &'static str = "REPORT";
  type Context = ();

  fn from_attributes(mut attrs: Attributes) -> Result<Self, VOTableError> {
    if let Some(attr_res) = attrs.next() {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = std::str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      return match attr.key {
        b"status" => Status::from_str(value)
          .map(Report::new)
          .map_err(VOTableError::Custom),
        _ => Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)),
      };
    }
    Err(VOTableError::Custom(format!(
      "Attribute 'status' is mandatory in tag '{}'",
      Self::TAG
    )))
  }

  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    // I dot not like the fact that we first create an empty String that we replace here... :o/
    read_content!(Self, self, reader, reader_buff)
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    read_content_by_ref!(Self, self, reader, reader_buff)
  }

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("status", self.status.to_string().as_str()));
    write_content!(self, elem_writer);
    Ok(())
  }
}
