
use std::{
  fmt,
  str::FromStr
};

/// See [IVOA spec](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html#primitives)
#[derive(PartialEq, Eq, Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Datatype {
  #[serde(rename = "boolean")]
  Logical,
  #[serde(rename = "bit")]
  Bit,
  #[serde(rename = "unsignedByte")]
  Byte,
  #[serde(rename = "short")]
  ShortInt,
  #[serde(rename = "int")]
  Int,
  #[serde(rename = "long")]
  LongInt,
  #[serde(rename = "char")]
  CharASCII,
  #[serde(rename = "unicodeChar")]
  CharUnicode,
  #[serde(rename = "float")]
  Float,
  #[serde(rename = "double")]
  Double,
  #[serde(rename = "floatComplex")]
  ComplexFloat,
  #[serde(rename = "doubleComplex")]
  ComplexDouble,
}

impl FromStr for Datatype {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "boolean" => Ok(Datatype::Logical),
      "bit" =>  Ok(Datatype::Bit),
      "unsignedByte" =>  Ok(Datatype::Byte),
      "short" =>  Ok(Datatype::ShortInt),
      "int" =>  Ok(Datatype::Int),
      "long" =>  Ok(Datatype::LongInt),
      "char" =>  Ok(Datatype::CharASCII),
      "unicodeChar" =>  Ok(Datatype::CharUnicode),
      "float" =>  Ok(Datatype::Float),
      "double" =>  Ok(Datatype::Double),
      "floatComplex" =>  Ok(Datatype::ComplexFloat),
      "doubleComplex" =>  Ok(Datatype::ComplexDouble),
      _ => Err(format!("Unknown 'datatype' variant. Actual: '{}'.", s))
    }
  }
}

impl fmt::Display for Datatype {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}",
      match self {
        Datatype::Logical => "boolean",
        Datatype::Bit => "bit",
        Datatype::Byte => "unsignedByte",
        Datatype::ShortInt => "short",
        Datatype::Int => "int",
        Datatype::LongInt => "long",
        Datatype::CharASCII => "char",
        Datatype::CharUnicode => "unicodeChar",
        Datatype::Float => "float",
        Datatype::Double => "double",
        Datatype::ComplexFloat => "floatComplex",
        Datatype::ComplexDouble => "doubleComplex",
      }
    )
  }
}