use std::{
  error::Error,
  fmt::{Debug, Display, Formatter, Result},
  marker::Copy,
  slice::Iter,
  str::FromStr,
};

use clap::ValueEnum;
use votable::VOTableError;

pub mod colnames;
pub mod fieldarray;
pub mod update;
#[cfg(feature = "vizier")]
pub mod viz_org_names;
pub mod votstruct;

pub struct StringError(String);

impl From<String> for StringError {
  fn from(value: String) -> Self {
    Self(value)
  }
}

impl From<StringError> for String {
  fn from(value: StringError) -> Self {
    value.0
  }
}

impl Error for StringError {
  #[allow(deprecated)]
  fn description(&self) -> &str {
    &self.0
  }
}

impl Display for StringError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    Display::fmt(&self.0, f)
  }
}

// Purposefully skip printing "StringError(..)"
impl Debug for StringError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    Debug::fmt(&self.0, f)
  }
}

// Remark: only VALUES and DESCRIPTION are unique in a given tag
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Tag {
  // Elements possibly having sub-elements
  VOTABLE = 0,
  RESOURCE,
  TABLE,
  DATA,
  FIELD,
  PARAM,
  GROUP,
  VALUES,
  OPTION,
  COOSYS,
  DEFINITION,
  // Elements without sub-elements
  DESCRIPTION,
  TIMESYS,
  INFO,
  LINK,
  FIELDRef,
  PARAMRef,
  MIN,
  MAX,
  STREAM,
  // MIVOT related tags
  VODML,
  // REPORT
  // MODEL
  // GLOBAL
  // TEMPLATE
  // ATTRIBUTE / COLLECTION / INSTANCE / REFERENCE
  // JOIN / WHERE / PRIMARY_KEY / FOREIGN_KEY
}
impl Default for Tag {
  fn default() -> Self {
    Self::VOTABLE
  }
}
const TAGS: [Tag; Tag::len()] = [
  Tag::VOTABLE,
  Tag::RESOURCE,
  Tag::TABLE,
  Tag::DATA,
  Tag::FIELD,
  Tag::PARAM,
  Tag::GROUP,
  Tag::VALUES,
  Tag::OPTION,
  Tag::COOSYS,
  Tag::DEFINITION,
  Tag::DESCRIPTION,
  Tag::TIMESYS,
  Tag::INFO,
  Tag::LINK,
  Tag::FIELDRef,
  Tag::PARAMRef,
  Tag::MIN,
  Tag::MAX,
  Tag::STREAM,
  Tag::VODML,
];

impl Tag {
  pub const fn index(&self) -> usize {
    *self as usize
  }
  pub const fn len() -> usize {
    Self::VODML as usize + 1
  }
  pub const fn array() -> [Tag; Tag::len()] {
    TAGS
  }
  pub const fn new_array<T>(default: T) -> [T; Tag::len()]
  where
    T: Copy,
  {
    [default; Tag::len()]
  }
  pub const fn new_array_of_vec<T>() -> [Vec<T>; Tag::len()] {
    [
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
      Vec::new(),
    ]
  }
  pub fn iterator() -> Iter<'static, Tag> {
    TAGS.iter()
  }
  /// # Remark
  /// * elements that may contains sub-elements are in upper case
  /// * elements that *cannot* contains sub-elemnts are in lower case
  fn char(&self) -> u8 {
    match self {
      Self::VOTABLE => b'D', // 'D' for "Document"
      Self::RESOURCE => b'R',
      Self::TABLE => b'T',
      Self::DATA => b'A',
      Self::FIELD => b'F',
      Self::PARAM => b'P',
      Self::GROUP => b'G',
      Self::VALUES => b'V',
      Self::OPTION => b'O',
      Self::COOSYS => b'C',
      Self::DEFINITION => b'E',
      Self::DESCRIPTION => b'd',
      Self::TIMESYS => b't',
      Self::INFO => b'i',
      Self::LINK => b'l',
      Self::FIELDRef => b'f',
      Self::PARAMRef => b'p',
      Self::MIN => b'n', // Final letter since both min and max start be 'm'
      Self::MAX => b'x', // Final letter since both min and max start be 'm'
      Self::STREAM => b's',
      Self::VODML => b'M', // For 'M'ivot of vod'M'l
    }
  }
}

impl FromStr for Tag {
  type Err = VOTableError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "VOTABLE" => Ok(Self::VOTABLE),
      "RESOURCE" => Ok(Self::RESOURCE),
      "TABLE" => Ok(Self::TABLE),
      "DATA" => Ok(Self::DATA),
      "FIELD" => Ok(Self::FIELD),
      "PARAM" => Ok(Self::PARAM),
      "GROUP" => Ok(Self::GROUP),
      "VALUES" => Ok(Self::VALUES),
      "OPTION" => Ok(Self::OPTION),
      "COOSYS" => Ok(Self::COOSYS),
      "DEFINITION" => Ok(Self::DEFINITION),
      "DESCRIPTION" => Ok(Self::DESCRIPTION),
      "TIMESYS" => Ok(Self::TIMESYS),
      "INFO" => Ok(Self::INFO),
      "LINK" => Ok(Self::LINK),
      "FIELDRef" => Ok(Self::FIELDRef),
      "PARAMRef" => Ok(Self::PARAMRef),
      "MIN" => Ok(Self::MIN),
      "MAX" => Ok(Self::MAX),
      "STREAM" => Ok(Self::STREAM),
      "VODML" => Ok(Self::VODML),
      _ => Err(VOTableError::Custom(format!("Tag '{}' not recognized.", s))),
    }
  }
}

impl Display for Tag {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    Debug::fmt(self, f)
  }
}
