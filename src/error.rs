
use std::{
  str::ParseBoolError,
  num::{ParseIntError, ParseFloatError},
};

use quick_error::quick_error;

quick_error! {
  #[derive(Debug)]
  pub enum VOTableError {
    UnexpectedAttr(attr: Vec<u8>, tag: &'static str) {
      display("Unexpected attribute {} in tag {}", String::from_utf8_lossy(attr), tag)
    }
    UnexpectedEmptyTag(tag: Vec<u8>, context_tag: &'static str) {
      display("Unexpected empty tag {} in tag {}", String::from_utf8_lossy(tag), context_tag)
    }
    UnexpectedStartTag(tag: Vec<u8>, context_tag: &'static str) {
      display("Unexpected start tag {} in tag {}", &String::from_utf8_lossy(tag), context_tag)
    }
    Variant(err: String) {
      display("Error parsing variant: {}", err)
    }
    ParseBool(err: ParseBoolError) {
      display("Error parsing a boolean: {}", err)
    }
    ParseInt(err: ParseIntError) {
      display("Error parsing an integer: {}", err)
    }
    ParseFloat(err: ParseFloatError) {
      display("Error parsing a float: {}", err)
    }
    ParseYear(err: ParseFloatError) {
      display("Error parsing a Besselian or Julian year: {}", err)
    }
    ParseDatatype(err: String) {
      display("Error parsing Datatype: {}", err)
    }
    Read(err: quick_xml::Error) {
      display("Error while reading: {}", err)
    }
    Write(err: quick_xml::Error) {
      display("Error while writing: {}", err)
    }
    Attr(err: quick_xml::events::attributes::AttrError) {
      display("Attributes error: {}", err)
    }
    PrematureEOF(tag: &'static str) {
      display("Premature End Of File encountered in tag {}", tag)
    }
    Io(err: std::io::Error) {
      display("I/O error: {}", err)
    }
    Utf8(err: std::str::Utf8Error) {
      display("Utf8 error, valid up to {}", err.valid_up_to())
    }
    FromUtf8(err: std::string::FromUtf8Error) {
      display("Utf8 error, valid up to {}", err)
    }
    FromUCS2(err: ucs2::Error) {
      display("From UCS2 error: {:?}", err)
    }
    ToUCS2(err: ucs2::Error) {
      display("To UCS2 error: {:?}", err)
    }
    Json(err: serde_json::Error) {
      display("Serde JSON error: {:?}", err)
    }
    Yaml(err: serde_yaml::Error) {
      display("Serde Yaml error: {:?}", err)
    }
    TomlSer(err: toml::ser::Error) {
      display("Serde Toml error: {:?}", err)
    }
    TomlDe(err: toml::de::Error) {
      display("Serde Toml error: {:?}", err)
    }
    Custom(err: std::string::String) {
      display("Custom error: {}", err)
    }
  }
}

impl serde::de::Error for VOTableError {
  fn custom<T: std::fmt::Display>(desc: T) -> Self {
    VOTableError::Custom(desc.to_string())
  }
}

impl serde::ser::Error for VOTableError {
  fn custom<T: std::fmt::Display>(desc: T) -> Self {
    VOTableError::Custom(desc.to_string())
  }
}
