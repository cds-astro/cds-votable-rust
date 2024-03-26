//! Module dedicated to the `STREAM` tag.

use std::{
  fmt,
  io::{BufRead, Write},
  str::{self, FromStr},
};

use paste::paste;
use quick_xml::{
  events::{BytesStart, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  impls::mem::VoidTableDataContent,
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_warn},
  QuickXmlReadWrite, SpecialElem, TableDataContent, VOTableElement,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Type {
  Locator,
  Other,
}
impl FromStr for Type {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "locator" => Ok(Type::Locator),
      "other" => Ok(Type::Other),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s)),
    }
  }
}
impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Type::Locator => "locator",
        Type::Other => "other",
      }
    )
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Actuate {
  OnLoad,
  OnRequest,
  Other,
  None,
}
impl FromStr for Actuate {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "onLoad" => Ok(Actuate::OnLoad),
      "onRequest" => Ok(Actuate::OnRequest),
      "other" => Ok(Actuate::Other),
      "none" => Ok(Actuate::None),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s)),
    }
  }
}
impl fmt::Display for Actuate {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Actuate::OnLoad => "onLoad",
        Actuate::OnRequest => "onRequest",
        Actuate::Other => "other",
        Actuate::None => "none",
      }
    )
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EncodingType {
  Gzip,
  Base64,
  Dynamic,
  None,
}
impl FromStr for EncodingType {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "gzip" => Ok(EncodingType::Gzip),
      "base64" => Ok(EncodingType::Base64),
      "dynamic" => Ok(EncodingType::Dynamic),
      "none" => Ok(EncodingType::None),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s)),
    }
  }
}
impl fmt::Display for EncodingType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        EncodingType::Gzip => "gzip",
        EncodingType::Base64 => "base64",
        EncodingType::Dynamic => "dynamic",
        EncodingType::None => "none",
      }
    )
  }
}

// Make 1 Stream for binary, 1 for binary2 and 1 for FTS??
#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stream<C: TableDataContent> {
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_: Option<Type>, // Locator by default
  #[serde(skip_serializing_if = "Option::is_none")]
  pub href: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub actuate: Option<Actuate>, // OnRequest by default
  #[serde(skip_serializing_if = "Option::is_none")]
  pub encoding: Option<EncodingType>,
  // None by default, base64 if no href
  #[serde(skip_serializing_if = "Option::is_none")]
  pub expires: Option<String>,
  // date!
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rights: Option<String>,
  // content, if not parsed by binary of binary2
  #[serde(flatten)]
  pub content: Option<C>,
}

impl Stream<VoidTableDataContent> {
  pub(crate) fn open_stream<R: BufRead>(
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
  ) -> Result<Stream<VoidTableDataContent>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.name() {
          Stream::<VoidTableDataContent>::TAG_BYTES => {
            // We could detect if current stream.content.is_some() to prevent from multi-stream...
            return Stream::<VoidTableDataContent>::from_event_start(e);
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.name() {
          Stream::<VoidTableDataContent>::TAG_BYTES => {
            return Stream::<VoidTableDataContent>::from_event_empty(e);
          }
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
}

impl<C: TableDataContent> Stream<C> {
  pub fn new() -> Self {
    Self::default()
  }

  // attributes
  impl_builder_opt_attr!(type_, type, Type);
  impl_builder_opt_string_attr!(href);
  impl_builder_opt_attr!(actuate, Actuate);
  impl_builder_opt_attr!(encoding, EncodingType);
  impl_builder_opt_string_attr!(expires);
  impl_builder_opt_string_attr!(rights);

  pub fn set_content(mut self, content: C) -> Self {
    self.set_content_by_ref(content);
    self
  }
  pub fn set_content_by_ref(&mut self, content: C) {
    self.content = Some(content);
  }

  pub fn write_start<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    self.for_each_attribute(|k, v| tag.push_attribute((k, v)));
    // push2write_extra!(self, tag);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)
  }

  pub fn write_end<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    // Close tag
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}

impl<C: TableDataContent> VOTableElement for Stream<C> {
  const TAG: &'static str = "STREAM";

  type MarkerType = SpecialElem;

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
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "type" => self.set_type_by_ref(
          val
            .as_ref()
            .parse::<Type>()
            .map_err(VOTableError::Variant)?,
        ),
        "href" => self.set_href_by_ref(val),
        "actuate" => self.set_actuate_by_ref(
          val
            .as_ref()
            .parse::<Actuate>()
            .map_err(VOTableError::Variant)?,
        ),
        "encoding" => self.set_encoding_by_ref(
          val
            .as_ref()
            .parse::<EncodingType>()
            .map_err(VOTableError::Variant)?,
        ),
        "expires" => self.set_expires_by_ref(val),
        "rights" => self.set_rights_by_ref(val),
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    Ok(())
  }

  /// Calls a closure on each (key, value) attribute pairs.
  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(type_) = &self.type_ {
      f("type", type_.to_string().as_str());
    }
    if let Some(href) = &self.href {
      f("href", href.as_str());
    }
    if let Some(actuate) = &self.actuate {
      f("actuate", actuate.to_string().as_str());
    }
    if let Some(encoding) = &self.encoding {
      f("encoding", encoding.to_string().as_str());
    }
    if let Some(expires) = &self.expires {
      f("expires", expires.as_str());
    }
    if let Some(rights) = &self.rights {
      f("rights", rights.as_str());
    }
  }
}

impl<C: TableDataContent> QuickXmlReadWrite<SpecialElem> for Stream<C> {
  type Context = ();

  fn read_content_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    Err(VOTableError::Custom(String::from(
      "Reading STREAM with a content must be taken in charge by the parent \
    element (since the encoding depends on the parent: BINARY of BINARY2)",
    )))
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    assert!(
      self.content.is_none(),
      "Writing STREAM with a content must be taken in charge by \
    the parent element (since the encoding depends on the parent: BINARy of BINARY2"
    );
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    write_opt_tostring_attr!(self, elem_writer, type_, "type");
    write_opt_string_attr!(self, elem_writer, href);
    write_opt_tostring_attr!(self, elem_writer, actuate);
    write_opt_tostring_attr!(self, elem_writer, encoding);
    write_opt_string_attr!(self, elem_writer, expires);
    write_opt_string_attr!(self, elem_writer, rights);
    // write_extra!(self, elem_writer);
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}
