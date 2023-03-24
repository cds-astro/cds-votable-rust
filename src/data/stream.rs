use std::{
  fmt,
  io::{BufRead, Write},
  str::{self, FromStr},
};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use paste::paste;

use serde;
use crate::{QuickXmlReadWrite, TableDataContent, impls::mem::VoidTableDataContent, error::VOTableError, is_empty};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Type {
  Locator,
  Other,
}
impl FromStr for Type {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "locator" => Ok(Type::Locator),
      "other" =>  Ok(Type::Other),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s))
    }
  }
}
impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}",
           match self {
             Type::Locator => "locator",
             Type::Other => "other",
           }
    )
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
      "onRequest" =>  Ok(Actuate::OnRequest),
      "other" => Ok(Actuate::Other),
      "none" =>  Ok(Actuate::None),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s))
    }
  }
}
impl fmt::Display for Actuate {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}",
           match self {
             Actuate::OnLoad => "onLoad",
             Actuate::OnRequest => "onRequest",
             Actuate::Other => "other",
             Actuate::None => "none",
           }
    )
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
      "base64" =>  Ok(EncodingType::Base64),
      "dynamic" => Ok(EncodingType::Dynamic),
      "none" =>  Ok(EncodingType::None),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s))
    }
  }
}
impl fmt::Display for EncodingType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}",
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
#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
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
  // extra attributes
  // #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  // pub extra: HashMap<String, Value>,
  // content, if not parsed by binary of binary2
  //#[serde(skip_serializing_if = "Option::is_none")]
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
        Event::Start(ref e) =>
          match e.name() {
            Stream::<VoidTableDataContent>::TAG_BYTES => {
              // We could detect if current stream.content.is_some() to prevent from multi-stream...
              let stream = Stream::<VoidTableDataContent>::from_attributes(e.attributes())?;
              return Ok(stream);
            },
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        Event::Empty(ref e) =>
          match e.name() {
            Stream::<VoidTableDataContent>::TAG_BYTES => {
              let stream = Stream::<VoidTableDataContent>::from_event_empty(e)?;
              return Ok(stream);
            },
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        Event::Text(e) if is_empty(e) => { },
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
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
  // extra attributes
  // impl_builder_insert_extra!();

  pub fn write_start<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_tostring_attr!(self, tag, type_, type);
    push2write_opt_string_attr!(self, tag, href);
    push2write_opt_tostring_attr!(self, tag, actuate);
    push2write_opt_tostring_attr!(self, tag, encoding);
    push2write_opt_string_attr!(self, tag, expires);
    push2write_opt_string_attr!(self, tag, rights);
    // push2write_extra!(self, tag);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)
  }

  pub fn write_end<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    // Close tag
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}


impl<C: TableDataContent> QuickXmlReadWrite for Stream<C> {
  const TAG: &'static str = "STREAM";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut stream = Self::default();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      stream = match attr.key {
        b"type" => stream.set_type(value.parse::<Type>().map_err(VOTableError::Variant)?),
        b"href" => stream.set_href(value),
        b"actuate" => stream.set_actuate(value.parse::<Actuate>().map_err(VOTableError::Variant)?),
        b"encoding" => stream.set_encoding(value.parse::<EncodingType>().map_err(VOTableError::Variant)?),
        b"expires" => stream.set_expires(value),
        b"rights" => stream.set_rights(value),
        _ => stream /*stream.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        )*/,
      }
    }
    Ok(stream)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut _reader: Reader<R>,
    mut _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    // read_content!(Self, self, reader, reader_buff)
    Err(
      VOTableError::Custom(
        String::from("Reading STREAM with a content must be taken in charge by the parent \
    element (since the encoding depends on the parent: BINARY of BINARY2")
      )
    )
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut _reader: &mut Reader<R>,
    mut _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    todo!()
  }
  
  
  fn write<W: Write>(&mut self, writer: &mut Writer<W>, _context: &Self::Context) -> Result<(), VOTableError> {
    assert!(self.content.is_none(), "Writing STREAM with a content must be taken in charge by \
    the parent element (since the encoding depends on the parent: BINARy of BINARY2");
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