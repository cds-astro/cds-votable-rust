use std::any::TypeId;
use std::fmt::{self, Display, Formatter, Write};
use std::num::ParseIntError;
use std::string::FromUtf8Error;

use serde::de::{DeserializeSeed, Visitor, Error};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::{
  error::VOTableError,
  field::Field,
  datatype::Datatype,
};

pub mod visitors;
pub mod b64;
pub mod mem;

use visitors::{/*get_visitor,*/ CharVisitor, StringVisitor, BytesVisitor};
use crate::impls::visitors::{FixedLengthArrayVisitor, VariableLengthArrayVisitor};

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum VOTableValue {
  Null,
  Bool(bool),
  // Bit(Vec<u8>),
  Byte(u8),
  Short(i16),
  Int(i32),
  Long(i64),
  Float(f32),
  Double(f64),
  ComplexFloat((f32, f32)),
  ComplexDouble((f64, f64)),
  CharASCII(char),
  CharUnicode(char),
  String(String),
  Bytes(Vec<u8>),
}

impl Serialize for VOTableValue {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    match self {
      VOTableValue::Null =>
        // Hack to be able to serialize in TOML (TOML does not support NULL values :o/)
        // but then, deserializing from TOML will lead to a VOTableValue::String("null") instead of
        // a VOTableValue::Null ... TODO find a solution to TOML deserialization of null values!
        // In VOTable, we are supposed to know the value coding for NULL, so we should forbid
        // 'null' values in JSON/YAML/TOML!
        if std::any::type_name::<S>() == "&mut toml::ser::Serializer" {
          serializer.serialize_str("")
        } else {
          serializer.serialize_none()
        }
      VOTableValue::Bool(v) => serializer.serialize_bool(*v),
      VOTableValue::Byte(v) => serializer.serialize_u8(*v),
      VOTableValue::Short(v) => serializer.serialize_i16(*v),
      VOTableValue::Int(v) => serializer.serialize_i32(*v),
      VOTableValue::Long(v) => serializer.serialize_i64(*v),
      VOTableValue::Float(v) => serializer.serialize_f32(*v),
      VOTableValue::Double(v) => serializer.serialize_f64(*v),
      VOTableValue::ComplexFloat((l, r)) => [l, r].serialize(serializer),
      VOTableValue::ComplexDouble((l, r)) => [l, r].serialize(serializer),
      VOTableValue::CharASCII(v) => serializer.serialize_char(*v as char),
      VOTableValue::CharUnicode(v) => serializer.serialize_char(*v),
      VOTableValue::String(v) => v.serialize(serializer), //serializer.serialize_str(v.as_str()),
      VOTableValue::Bytes(v) => serializer.serialize_bytes(v.as_slice())
    }
  }
}

impl Display for VOTableValue {
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    match self {
      VOTableValue::Null => fmt.write_str(""),
      VOTableValue::Bool(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Byte(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Short(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Int(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Long(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Float(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Double(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::ComplexFloat((l, r)) => fmt.write_fmt(format_args!("{} {}", l, r)),
      VOTableValue::ComplexDouble((l, r)) => fmt.write_fmt(format_args!("{} {}", l, r)),
      VOTableValue::CharASCII(v) => fmt.write_char(*v as char),
      VOTableValue::CharUnicode(v) => fmt.write_char(*v),
      VOTableValue::String(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::Bytes(v) => todo!(),
    }
  }
}






pub enum Schema {
  Bool, 
  // Bit,
  Byte  { null: Option<u8> },
  Short { null: Option<i16> },
  Int   { null: Option<i32> },
  Long  { null: Option<i64> },
  Float,
  Double,
  ComplexFloat,
  ComplexDouble,
  CharASCII,
  CharUnicode,
  FixedLengthStringASCII { n_chars: usize },
  FixedLengthStringUnicode { n_chars: usize },
  VariableLengthStringASCII,
  VariableLengthStringUnicode,
  FixedLengthArray { n_elems: usize, elem_schema: Box<Schema> },
  VariableLengthArray { elem_schema: Box<Schema> },
}

impl Schema {
  // For VOTable DATATABLE field deserialization
  pub fn value_from_str(&self, s: &str) -> Result<VOTableValue, VOTableError> {
    Ok(
      if s.is_empty() {
        VOTableValue::Null
      } else {
        match self {
          Schema::Bool => {
            if s == "?" {
              VOTableValue::Null  
            } else {
              VOTableValue::Bool(s.parse().map_err(VOTableError::ParseBool)?)
            }
          },
          // Schema::Bit => VOTableValue::Bit(s.parse().map_err(VOTableError)?),
          Schema::Byte { null: None } => VOTableValue::Byte(s.parse().map_err(VOTableError::ParseInt)?),
          Schema::Byte { null: Some(null) } => {
            let val = s.parse().map_err(VOTableError::ParseInt)?;
            if val == *null {
              VOTableValue::Null
            } else {
              VOTableValue::Byte(val)
            }
          },
          Schema::Short { null: None } => VOTableValue::Short(s.parse().map_err(VOTableError::ParseInt)?),
          Schema::Short { null: Some(null) } => {
            let val = s.parse().map_err(VOTableError::ParseInt)?;
            if val == *null {
              VOTableValue::Null
            } else {
              VOTableValue::Short(val)
            }
          },
          Schema::Int { null: None } => VOTableValue::Int(s.parse().map_err(VOTableError::ParseInt)?),
          Schema::Int { null: Some(null) } =>{
            let val = s.parse().map_err(VOTableError::ParseInt)?;
            if val == *null {
              VOTableValue::Null
            } else {
              VOTableValue::Int(val)
            }
          },
          Schema::Long { null: None } => VOTableValue::Long(s.parse().map_err(VOTableError::ParseInt)?),
          Schema::Long { null: Some(null) } => {
            let val = s.parse().map_err(VOTableError::ParseInt)?;
            if val == *null {
              VOTableValue::Null
            } else {
              VOTableValue::Long(val)
            }
          },
          Schema::Float => VOTableValue::Float(s.parse().map_err(VOTableError::ParseFloat)?),
          Schema::Double => VOTableValue::Double(s.parse().map_err(VOTableError::ParseFloat)?),
          Schema::ComplexFloat =>
            match s.split_once(' ') {
              Some((l, r)) => VOTableValue::ComplexFloat(
                (l.parse().map_err(VOTableError::ParseFloat)?,
                 r.parse().map_err(VOTableError::ParseFloat)?
                )
              ),
              None => return Err(VOTableError::Custom(format!("Unable to parse complex float value: {}", s)))
            }
          Schema::ComplexDouble =>
            match s.split_once(' ') {
              Some((l, r)) => VOTableValue::ComplexDouble(
                (l.parse().map_err(VOTableError::ParseFloat)?,
                 r.parse().map_err(VOTableError::ParseFloat)?
                )
              ),
              None => return Err(VOTableError::Custom(format!("Unable to parse complex double value: {}", s)))
            }
          Schema::CharASCII => VOTableValue::CharASCII(s.chars().next().unwrap()), // unwrap ok since we already checked for empty string
          Schema::CharUnicode => VOTableValue::CharUnicode(s.chars().next().unwrap()), // unwrap ok since we already checked for empty string
          Schema::FixedLengthStringASCII { n_chars: _ } => VOTableValue::String(s.to_owned()),
          Schema::VariableLengthStringASCII => VOTableValue::String(s.to_owned()),
          Schema::FixedLengthStringUnicode { n_chars: _ } => VOTableValue::String(s.to_owned()),
          Schema::VariableLengthStringUnicode => VOTableValue::String(s.to_owned()),
          // Schema::Bytes => unreachable!() // only in binary mode?
          _ => todo!()
        }
      }
    )
  }

  fn serialize_seed<S>(&self, value: VOTableValue, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    /*match value {
      VOTableValue::Null =>
        match self {
          Schema::Bool => serializer.serialize_u8(b'?'),
          Schema::Byte  { null } => serializer.serialize_u8(null.unwrap_or(u8::MAX)),
          Schema::Short { null } => serializer.serialize_i16(null.unwrap_or(i16::MIN)),
          Schema::Int   { null } => serializer.serialize_i32(null.unwrap_or(i32::MIN)),
          Schema::Long  { null } => serializer.serialize_i64(null.unwrap_or(i64::MIN)),
          Schema::Float => serializer.serialize_f32(f32::NAN),
          Schema::Double => serializer.serialize_f64(f64::NAN),
          Schema::ComplexFloat => [f32::NAN, f32::NAN].serialize(serializer),
          Schema::ComplexDouble => [f64::NAN, f64::NAN].serialize(serializer),
          Schema::CharASCII => serializer.serialize_u8(b'\0'),
          Schema::CharUnicode => serializer.serialize_char('\0'),
          Schema::FixedLengthStringASCII { n_chars: usize } => todo!(), // taillefixe tableau!!
          Schema::FixedLengthStringUnicode { n_chars: usize } => todo!(),
          Schema::VariableLengthStringASCII => todo!(),
          Schema::VariableLengthStringUnicode => todo!(),
          // Schema::Bytes => serializer.serialize_bytes([0_u8; 0].as_slice())
        }
      VOTableValue::Bool(v) => serializer.serialize_bool(*v),
      VOTableValue::Byte(v) => serializer.serialize_u8(*v),
      VOTableValue::Short(v) => serializer.serialize_i16(*v),
      VOTableValue::Int(v) => serializer.serialize_i32(*v),
      VOTableValue::Long(v) => serializer.serialize_i64(*v),
      VOTableValue::Float(v) => serializer.serialize_f32(*v),
      VOTableValue::Double(v) => serializer.serialize_f64(*v),
      VOTableValue::ComplexFloat((l, r)) => [l, r].serialize(serializer),
      VOTableValue::ComplexDouble((l, r)) => [l, r].serialize(serializer),
      VOTableValue::CharASCII(v) => serializer.serialize_char(*v as char),
      VOTableValue::CharUnicode(v) => serializer.serialize_char(*v),
      VOTableValue::String(v) => v.serialize(serializer), //serializer.serialize_str(v.as_str()),
      VOTableValue::Bytes(v) => serializer.serialize_bytes(v.as_slice())
    }*/
    todo!()
  }
  
  /*pub fn serialize_seed<S>(&self, value: VOTableValue, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
  SERIALIZER TO BE IMPLEMENTED FOR BINARY!!
    match self {
      Schema::Bool => {},
      Schema::Bit => {}
      Schema::Byte => {}
      Schema::Short => {}
      Schema::Int => {}
      Schema::Long => {}
      Schema::Float => {}
      Schema::Double => {}
      Schema::ComplexFloat => {}
      Schema::ComplexDouble => {}
      Schema::CharASCII => {}
      Schema::CharUnicode => {}
      Schema::String => {}
      Schema::Bytes => {}
    }
  }*/
  
}

impl From<&Field> for Schema {
  fn from(field: &Field) -> Self {
    match (field.datatype, field.arraysize.as_ref().map(|string_ref| string_ref.as_str())) {
      (Datatype::Logical, None) => Schema::Bool,
      // (Datatype::Bit, None) => Schema::Bit,
      (Datatype::Byte, None) => Schema::Byte { 
        null: field.null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default() // i.e. null = None
      },
      (Datatype::ShortInt, None) => Schema::Short {
        null: field.null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default() // i.e. null = None
      },
      (Datatype::Int, None) => Schema::Int {
        null: field.null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default() // i.e. null = None
      },
      (Datatype::LongInt, None) => Schema::Long {
        null: field.null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default() // i.e. null = None
      },
      (Datatype::CharACII, None) => Schema::CharASCII,
      (Datatype::CharUnicode, None) => Schema::CharUnicode,
      (Datatype::Float, None) => Schema::Float,
      (Datatype::Double, None) => Schema::Double,
      (Datatype::ComplexFloat, None) => Schema::ComplexFloat,
      (Datatype::ComplexDouble, None) => Schema::ComplexDouble,
      // Char/String
      (Datatype::CharACII, Some("1")) => Schema::CharASCII,
      (Datatype::CharACII, Some(size)) => 
        match fixed_length_array(size) {
          Err(e) => {
            eprintln!("Error parsing arraysize: {:?}. Set to variable length.", e);
            Schema::VariableLengthStringASCII
          },
          Ok(Ok(n_chars)) => Schema::FixedLengthStringASCII { n_chars },
          Ok(Err(_)) => Schema::VariableLengthStringASCII,
        }
      (Datatype::CharUnicode, Some("1")) => Schema::CharUnicode,
      (Datatype::CharUnicode, Some(size)) =>
        match fixed_length_array(size) {
          Err(e) => {
            eprintln!("Error parsing arraysize: {:?}. Set to variable length.", e);
            Schema::VariableLengthStringUnicode
          },
          Ok(Ok(n_chars)) => Schema::FixedLengthStringUnicode { n_chars },
          Ok(Err(_)) => Schema::VariableLengthStringUnicode,
        }
      // Other
      (_, _) => todo!(),
    }
  }
}

/// If the result is:
/// * `Ok(usize)` => fixed length array of given size
/// * `Err(usize)` => variable size array of at least the given size
pub fn fixed_length_array(arraysize: &str) -> Result<Result<usize, usize>, ParseIntError> {
  let (arraysize, is_variable) = if arraysize.ends_with('*') {
    (arraysize.strip_suffix('*').unwrap_or(""), true)
  } else {
    (arraysize, false)
  };
  let elems = arraysize.split('x')
      .map(|v| v.parse::<usize>())
      .collect::<Result<Vec<usize>, ParseIntError>>()?;
  let n_tot = elems.into_iter().reduce(|acc, n| acc * n).unwrap_or(0);
  Ok(if is_variable {
    Err(n_tot)
  } else {
    Ok(n_tot)
  })
}

impl<'de> DeserializeSeed<'de> for &Schema {
  
  type Value = VOTableValue;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
      D: Deserializer<'de>
  {
    match self {
      Schema::Bool => <bool>::deserialize(deserializer).map(VOTableValue::Bool),
      // Schema::Bit => todo!(),
      Schema::Byte { null: None } => <u8>::deserialize(deserializer).map(VOTableValue::Byte),
      Schema::Short { null: None } =>  <i16>::deserialize(deserializer).map(VOTableValue::Short),
      Schema::Int { null: None } =>  <i32>::deserialize(deserializer).map(VOTableValue::Int),
      Schema::Long { null: None } =>  <i64>::deserialize(deserializer).map(VOTableValue::Long),
      Schema::Byte { null: Some(null) } => {
        let v = <u8>::deserialize(deserializer)?;
        Ok(if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Byte(v)
        })
      }
      Schema::Short { null: Some(null) } => {
        let v = <i16>::deserialize(deserializer)?;
        Ok(if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Short(v)
        })
      }
      Schema::Int { null: Some(null) } => {
        let v = <i32>::deserialize(deserializer)?;
        Ok(if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Int(v)
        })
      }
      Schema::Long { null: Some(null) } => {
        let v = <i64>::deserialize(deserializer)?;
        Ok(if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Long(v)
        })
      }
      Schema::Float => {
        let v = <f32>::deserialize(deserializer)?;
        Ok(if v.is_finite() {
          VOTableValue::Float(v)
        } else {
          VOTableValue::Null
        })
      }
      Schema::Double => {
        let v: f64 = <f64>::deserialize(deserializer)?;
        Ok(if v.is_finite() {
          VOTableValue::Double(v)
        } else {
          VOTableValue::Null
        })
      }
      Schema::ComplexFloat => {
        let (real, img): (f32, f32) = <(f32, f32)>::deserialize(deserializer)?;
        Ok(if real.is_finite() && img.is_finite() {
          VOTableValue::ComplexFloat((real, img))
        } else {
          VOTableValue::Null
        })
      }
      Schema::ComplexDouble => {
        let (real, img): (f64, f64) = <(f64, f64)>::deserialize(deserializer)?;
        Ok(if real.is_finite() && img.is_finite() {
          VOTableValue::ComplexDouble((real, img))
        } else {
          VOTableValue::Null
        })
      }
      Schema::CharASCII => deserializer.deserialize_u8(CharVisitor).map(VOTableValue::CharASCII),
      Schema::CharUnicode => deserializer.deserialize_u16(CharVisitor).map(VOTableValue::CharUnicode),
      Schema::FixedLengthStringASCII { n_chars } => {
        let n_bytes = *n_chars;
        let visitor = FixedLengthArrayVisitor::new(n_bytes);
        let bytes: Vec<u8> = deserializer.deserialize_tuple(n_bytes, visitor)?;
        String::from_utf8(bytes).map_err(D::Error::custom).map(VOTableValue::String)
      },
      Schema::FixedLengthStringUnicode { n_chars } => {
        let visitor = FixedLengthArrayVisitor::new(*n_chars);
        let bytes: Vec<u16> = deserializer.deserialize_tuple(*n_chars, visitor)?;
        decode_ucs2(bytes).map_err(D::Error::custom).map(VOTableValue::String)
      },
      Schema::VariableLengthStringASCII => {
        let visitor = VariableLengthArrayVisitor::new();
        let bytes: Vec<u8> = deserializer.deserialize_seq(visitor)?;
        String::from_utf8(bytes).map_err(D::Error::custom).map(VOTableValue::String)
      },
      Schema::VariableLengthStringUnicode => {
        let visitor = VariableLengthArrayVisitor::new();
        let bytes: Vec<u16> = deserializer.deserialize_seq(visitor)?;
        decode_ucs2(bytes).map_err(D::Error::custom).map(VOTableValue::String)
      },
      // Schema::Bytes => deserializer.deserialize_bytes(BytesVisitor).map(VOTableValue::Bytes),
      _ => todo!()
    }
  }
}

pub fn decode_ucs2(bytes: Vec<u16>) -> Result<String, VOTableError> {
  let mut bytes_utf8 = vec![0_u8;  bytes.len() << 1];
  let n = ucs2::decode(&bytes, &mut bytes_utf8)
    .map_err(VOTableError::FromUCS2)?;
  bytes_utf8.truncate(n);
  String::from_utf8(bytes_utf8).map_err(VOTableError::FromUtf8)
}