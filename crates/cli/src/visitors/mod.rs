use std::{
  error::Error,
  fmt::{Debug, Display, Formatter, Result},
};

pub mod colnames;
pub mod fieldarray;
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

#[derive(Debug, Copy, Clone)]
// Remark: only VALUES and DESCRIPTION are unique in a given tag
pub enum Tag {
  // Elements possibly having sub-elements
  VOTABLE,
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
}

impl Tag {
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
    }
  }
}

impl Display for Tag {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    /*f.write_str(match self {
      Self::VOTABLE => "VOTABLE",
      Self::RESOURCE => "RESOURCE",
      Self::TABLE => "TABLE",
      Self::FIELD => "FIELD",
      Self::PARAM => "PARAM",
      Self::GROUP => "GROUP",
      Self::VALUES => "VALUES",
      Self::OPTION => "OPTION",
      Self::COOSYS => "COOSYS",
      Self::DESCRIPTION => "DESCRIPTION",
      Self::TIMESYS => "TIMESYS",
      Self::INFO => "INFO",
      Self::LINK => "LINK",
      Self::FIELDRef => "FIELDRef",
      Self::PARAMRef => "PARAMRef",
      Self::MIN => "MIN",
      Self::MAX => "MAX",
    })*/
    Debug::fmt(self, f)
  }
}
