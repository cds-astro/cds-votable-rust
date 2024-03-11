use std::{
  fmt::{self, Display, Formatter, Write},
  mem::size_of,
  slice::Iter,
};

use bitvec::{order::Msb0, vec::BitVec as BV};

use serde::{
  de::{DeserializeSeed, Error as DeError},
  ser::{Error as SerError, SerializeSeq, SerializeTuple},
  Deserialize, Deserializer, Serialize, Serializer,
};

use crate::{
  datatype::Datatype,
  error::VOTableError,
  field::ArraySize,
  field::Field,
  impls::visitors::{FixedLengthArrayVisitor, VariableLengthArrayVisitor},
  table::TableElem,
};

pub mod b64;
pub mod mem;
pub mod visitors;
use visitors::CharVisitor;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BitVec(BV<u8, Msb0>);
/*impl Serialize for BitVec {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    self.0.serialize(serializer) // TODO: check if the serialisation is in the right order, else do it ourself!
  }
}*/

#[derive(Debug, Clone)]
pub struct TableSchema(Vec<Schema>);
impl TableSchema {
  pub fn unwrap(self) -> Vec<Schema> {
    self.0
  }
  pub fn as_slice(&self) -> &[Schema] {
    self.0.as_slice()
  }
  pub fn iter(&self) -> Iter<'_, Schema> {
    self.0.iter()
  }
}
impl<'a> From<&'a [TableElem]> for TableSchema {
  fn from(context: &[TableElem]) -> Self {
    let schema: Vec<Schema> = context
      .iter()
      .filter_map(|table_elem| match table_elem {
        TableElem::Field(field) => Some(field.into()),
        _ => None,
      })
      .collect();
    Self(schema)
  }
}

// WARNING: THE ORDER IS IMPORTANT WHEN DESERIALIZING JSON, NOT TO LOOSE SIGNIFICANT DIGITS!!
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(untagged)]
pub enum VOTableValue {
  Null,
  Bool(bool), // <=> Bit(bool)
  Byte(u8),
  Short(i16),
  Int(i32),
  Long(i64),
  Double(f64),
  Float(f32),
  ComplexDouble((f64, f64)),
  ComplexFloat((f32, f32)),
  CharASCII(char),
  CharUnicode(char),
  String(String),
  BitArray(BitVec),
  ByteArray(Vec<u8>),
  ShortArray(Vec<i16>),
  IntArray(Vec<i32>),
  LongArray(Vec<i64>),
  DoubleArray(Vec<f64>),
  FloatArray(Vec<f32>),
  ComplexDoubleArray(Vec<(f64, f64)>),
  ComplexFloatArray(Vec<(f32, f32)>),
}
impl AsRef<VOTableValue> for VOTableValue {
  fn as_ref(&self) -> &Self {
    self
  }
}
impl Serialize for VOTableValue {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      VOTableValue::Null =>
      // Hack to be able to serialize in TOML (TOML does not support NULL values :o/)
      // but then, deserializing from TOML will lead to a VOTableValue::String("null") instead of
      // a VOTableValue::Null ... TODO find a solution to TOML deserialization of null values!
      // In VOTable, we are supposed to know the value coding for NULL, so we should forbid
      // 'null' values in JSON/YAML/TOML!
      {
        let ser_name = std::any::type_name::<S>();
        if ser_name == "&mut toml::ser::Serializer"
          || ser_name == "toml_edit::ser::value::ValueSerializer"
        {
          serializer.serialize_str("")
        } else {
          serializer.serialize_none()
        }
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
      VOTableValue::CharASCII(v) => serializer.serialize_char(*v),
      VOTableValue::CharUnicode(v) => serializer.serialize_char(*v),
      VOTableValue::String(v) => v.serialize(serializer), //serializer.serialize_str(v.as_str()),
      VOTableValue::BitArray(v) => v.serialize(serializer),
      VOTableValue::ByteArray(v) => serializer.serialize_bytes(v.as_slice()),
      VOTableValue::ShortArray(v) => v.serialize(serializer),
      VOTableValue::IntArray(v) => v.serialize(serializer),
      VOTableValue::LongArray(v) => v.serialize(serializer),
      VOTableValue::FloatArray(v) => v.serialize(serializer),
      VOTableValue::DoubleArray(v) => v.serialize(serializer),
      VOTableValue::ComplexFloatArray(v) => v.serialize(serializer),
      VOTableValue::ComplexDoubleArray(v) => v.serialize(serializer),
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
      VOTableValue::CharASCII(v) => fmt.write_char(*v),
      VOTableValue::CharUnicode(v) => fmt.write_char(*v),
      VOTableValue::String(v) => fmt.write_fmt(format_args!("{}", v)),
      VOTableValue::BitArray(v) => fmt.write_fmt(format_args!("{:?}", &v.0)),
      VOTableValue::ByteArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::ShortArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::IntArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::LongArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::FloatArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::DoubleArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::ComplexFloatArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
      VOTableValue::ComplexDoubleArray(v) => fmt.write_fmt(format_args!("{:?}", &v)),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Schema {
  Bool,
  Bit,
  Byte {
    null: Option<u8>,
  },
  Short {
    null: Option<i16>,
  },
  Int {
    null: Option<i32>,
  },
  Long {
    null: Option<i64>,
  },
  Float,
  Double,
  ComplexFloat,
  ComplexDouble,
  CharASCII,
  CharUnicode,
  FixedLengthStringASCII {
    n_chars: usize,
  },
  FixedLengthStringUnicode {
    n_chars: usize,
  },
  VariableLengthStringASCII {
    /// Maximum number of characters the array may contain if declared as `INT*`), `None` if
    /// declared as `*`.
    n_chars_max: Option<usize>,
  },
  VariableLengthStringUnicode {
    /// Maximum number of characters the array may contain if declared as `INT*`), `None` if
    /// declared as `*`.
    n_chars_max: Option<usize>,
  },
  FixedLengthBitArray {
    n_bits: usize,
  },
  VariableLengthBitArray {
    /// Maximum number of characters the array may contain if declared as `INT*`), `None` if
    /// declared as `*`.
    n_bits_max: Option<usize>,
  },
  FixedLengthArray {
    n_elems: usize,
    elem_schema: Box<Schema>,
  },
  VariableLengthArray {
    /// Maximum number of elements the array may contain if declared as `INT*`), `None` if
    /// declared as `*`.
    n_elems_max: Option<usize>,
    /// The underlying schema can contains a fixed length array (of fixed length array, ...)
    /// of primitive type
    elem_schema: Box<Schema>,
  }, // { n_chars_min: usize }
}

impl Schema {
  /// Returns the size, in bytes, of the binary representation of an entry associated to the Schema.
  /// # Output
  /// * `Ok` for fixed length objects
  /// * `Err` containing first the size of an element and then the upper bound of the len (if known), for variable size objects
  pub fn byte_len(&self) -> Result<usize, (usize, Option<usize>)> {
    match self {
      Schema::Bool => Ok(size_of::<u8>()),
      Schema::Bit => Ok(size_of::<u8>()),
      Schema::Byte { .. } => Ok(size_of::<u8>()),
      Schema::Short { .. } => Ok(size_of::<i16>()),
      Schema::Int { .. } => Ok(size_of::<i32>()),
      Schema::Long { .. } => Ok(size_of::<i64>()),
      Schema::Float => Ok(size_of::<f32>()),
      Schema::Double => Ok(size_of::<f64>()),
      Schema::ComplexFloat => Ok(size_of::<f32>() << 1),
      Schema::ComplexDouble => Ok(size_of::<f64>() << 2),
      Schema::CharASCII => Ok(size_of::<u8>()),
      Schema::CharUnicode => Ok(size_of::<u16>()),
      Schema::FixedLengthStringASCII { n_chars } => Ok(n_chars * size_of::<u8>()),
      Schema::FixedLengthStringUnicode { n_chars } => Ok(n_chars * size_of::<u16>()),
      Schema::VariableLengthStringASCII { n_chars_max } => {
        Err((size_of::<u8>(), n_chars_max.map(|n| n * size_of::<u8>())))
      }
      Schema::VariableLengthStringUnicode { n_chars_max } => {
        Err((size_of::<u16>(), n_chars_max.map(|n| n * size_of::<u16>())))
      }
      Schema::FixedLengthArray {
        n_elems,
        elem_schema,
      } => match elem_schema.as_ref() {
        Schema::Bit => Ok((*n_elems + 7) / 8),
        _ => elem_schema.byte_len().map(|l| l * n_elems),
      },
      Schema::VariableLengthArray {
        n_elems_max,
        elem_schema,
      } => {
        let elem_byte_len = elem_schema
          .byte_len()
          .expect("Variable size array of variable size elements not supported in VOTable!");
        Err((elem_byte_len, n_elems_max.map(move |n| n * elem_byte_len)))
      }
      Schema::FixedLengthBitArray { n_bits } => Ok((*n_bits + 7) / 8),
      Schema::VariableLengthBitArray { n_bits_max } => {
        Err((0, n_bits_max.map(move |n_bits| (n_bits + 7) / 8)))
      }
    }
  }

  // For VOTable DATATABLE field deserialization
  pub fn value_from_str(&self, s: &str) -> Result<VOTableValue, VOTableError> {
    Ok(if s.is_empty() {
      VOTableValue::Null
    } else {
      match self {
        Schema::Bool => {
          if s == "?" {
            VOTableValue::Null
          } else {
            match s {
                        "0" | "f" | "F" => VOTableValue::Bool(false),
                        "1" | "t" | "T" => VOTableValue::Bool(true),
                        _ => {
                          s.parse::<bool>().map(VOTableValue::Bool).map_err(|_| VOTableError::Custom(
                            format!("Unable too parse boolean value. Expected: '0', '1', 't', 'f', 'T', 'F' or 'true', 'false'. Actual: '{}'", s))
                          )?
                        }
                      }
          }
        }
        Schema::Bit => match s {
          "0" => VOTableValue::Bool(false),
          "1" => VOTableValue::Bool(true),
          _ => return Err(VOTableError::Custom(format!("Unknown bit value: '{}'", s))),
        },
        Schema::Byte { null: None } => {
          VOTableValue::Byte(s.parse().map_err(VOTableError::ParseInt)?)
        }
        Schema::Byte { null: Some(null) } => {
          let val = s.parse().map_err(VOTableError::ParseInt)?;
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Byte(val)
          }
        }
        Schema::Short { null: None } => {
          VOTableValue::Short(s.parse().map_err(VOTableError::ParseInt)?)
        }
        Schema::Short { null: Some(null) } => {
          let val = s.parse().map_err(VOTableError::ParseInt)?;
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Short(val)
          }
        }
        Schema::Int { null: None } => VOTableValue::Int(s.parse().map_err(VOTableError::ParseInt)?),
        Schema::Int { null: Some(null) } => {
          let val = s.parse().map_err(VOTableError::ParseInt)?;
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Int(val)
          }
        }
        Schema::Long { null: None } => {
          VOTableValue::Long(s.parse().map_err(VOTableError::ParseInt)?)
        }
        Schema::Long { null: Some(null) } => {
          let val = s.parse().map_err(VOTableError::ParseInt)?;
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Long(val)
          }
        }
        Schema::Float => VOTableValue::Float(s.parse().map_err(VOTableError::ParseFloat)?),
        Schema::Double => VOTableValue::Double(s.parse().map_err(VOTableError::ParseFloat)?),
        Schema::ComplexFloat => match s.split_once(' ') {
          Some((l, r)) => VOTableValue::ComplexFloat((
            l.parse().map_err(VOTableError::ParseFloat)?,
            r.parse().map_err(VOTableError::ParseFloat)?,
          )),
          None => {
            return Err(VOTableError::Custom(format!(
              "Unable to parse complex float value: {}",
              s
            )))
          }
        },
        Schema::ComplexDouble => match s.split_once(' ') {
          Some((l, r)) => VOTableValue::ComplexDouble((
            l.parse().map_err(VOTableError::ParseFloat)?,
            r.parse().map_err(VOTableError::ParseFloat)?,
          )),
          None => {
            return Err(VOTableError::Custom(format!(
              "Unable to parse complex double value: {}",
              s
            )))
          }
        },
        Schema::CharASCII => VOTableValue::CharASCII(s.chars().next().unwrap()), // unwrap ok since we already checked for empty string
        Schema::CharUnicode => VOTableValue::CharUnicode(s.chars().next().unwrap()), // unwrap ok since we already checked for empty string
        Schema::FixedLengthStringASCII { n_chars: _ } => VOTableValue::String(s.to_owned()),
        Schema::VariableLengthStringASCII { n_chars_max: _ } => VOTableValue::String(s.to_owned()),
        Schema::FixedLengthStringUnicode { n_chars: _ } => VOTableValue::String(s.to_owned()),
        Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          VOTableValue::String(s.to_owned())
        }
        // Schema::Bytes => unreachable!() // only in binary mode?
        Schema::FixedLengthArray {
          n_elems,
          elem_schema,
        } => {
          let (n_actual_elems, value) = elem_schema.parse_array(s)?;
          if n_actual_elems != *n_elems {
            return Err(VOTableError::Custom(format!(
              "Wrong number of fixed length array elements. Expected: {}. Actual: {}.",
              n_elems, n_actual_elems
            )));
          }
          value
        }
        Schema::VariableLengthArray {
          n_elems_max: _,
          elem_schema,
        } => elem_schema.parse_array(s)?.1,
        Schema::FixedLengthBitArray { n_bits } => {
          let (n_actual_elems, value) = self.parse_bit_array(s)?;
          if n_actual_elems != *n_bits {
            return Err(VOTableError::Custom(format!(
              "Wrong number of fixed length bits elements. Expected: {}. Actual: {}.",
              n_bits, n_actual_elems
            )));
          }
          value
        }
        Schema::VariableLengthBitArray { n_bits_max: _ } => self.parse_bit_array(s)?.1,
      }
    })
  }

  fn parse_bit_array(&self, array_str: &str) -> Result<(usize, VOTableValue), VOTableError> {
    let elems: Vec<&str> = array_str.trim().split(' ').collect();
    let mut bitvec = BV::new();
    for s in elems {
      match s {
        "0" => bitvec.push(false),
        "1" => bitvec.push(true),
        _ => return Err(VOTableError::Custom(format!("Unknown bit value: '{}'", s))),
      }
    }
    Ok((bitvec.len(), VOTableValue::BitArray(BitVec(bitvec))))
  }

  fn parse_array(&self, array_str: &str) -> Result<(usize, VOTableValue), VOTableError> {
    // let data: Vec<> = s.trim().split(' ').map(|s| elem_schema.value_from_str(s)).collect()?;
    let elems: Vec<&str> = array_str.trim().split(' ').collect();
    let n_elems = elems.len();
    Ok((
      n_elems,
      match self {
        // Schema::Bool => VOTableValue::Bool() elem_it.map(s.parse().map_err(VOTableError::ParseBool)).collect()?,
        Schema::Byte { .. } => VOTableValue::ByteArray(
          elems
            .iter()
            .map(|s| s.parse().map_err(VOTableError::ParseInt))
            .collect::<Result<Vec<u8>, VOTableError>>()?,
        ),
        Schema::Short { .. } => VOTableValue::ShortArray(
          elems
            .iter()
            .map(|s| s.parse().map_err(VOTableError::ParseInt))
            .collect::<Result<Vec<i16>, VOTableError>>()?,
        ),
        Schema::Int { .. } => VOTableValue::IntArray(
          elems
            .iter()
            .map(|s| s.parse().map_err(VOTableError::ParseInt))
            .collect::<Result<Vec<i32>, VOTableError>>()?,
        ),
        Schema::Long { .. } => VOTableValue::LongArray(
          elems
            .iter()
            .map(|s| s.parse().map_err(VOTableError::ParseInt))
            .collect::<Result<Vec<i64>, VOTableError>>()?,
        ),
        Schema::Float => VOTableValue::FloatArray(
          elems
            .iter()
            .map(|s| s.parse().map_err(VOTableError::ParseFloat))
            .collect::<Result<Vec<f32>, VOTableError>>()?,
        ),
        Schema::Double => VOTableValue::DoubleArray(
          elems
            .iter()
            .map(|s| s.parse().map_err(VOTableError::ParseFloat))
            .collect::<Result<Vec<f64>, VOTableError>>()?,
        ),
        Schema::ComplexFloat => VOTableValue::ComplexFloatArray(
          elems
            .iter()
            .step_by(2)
            .zip(elems.iter().skip(1).step_by(2))
            .map(|(l_str, r_str)| {
              l_str
                .parse()
                .map_err(VOTableError::ParseFloat)
                .and_then(|l| {
                  r_str
                    .parse()
                    .map_err(VOTableError::ParseFloat)
                    .map(|r| (l, r))
                })
            })
            .collect::<Result<Vec<(f32, f32)>, VOTableError>>()?,
        ),
        Schema::ComplexDouble => VOTableValue::ComplexDoubleArray(
          elems
            .iter()
            .step_by(2)
            .zip(elems.iter().skip(1).step_by(2))
            .map(|(l_str, r_str)| {
              l_str
                .parse()
                .map_err(VOTableError::ParseFloat)
                .and_then(|l| {
                  r_str
                    .parse()
                    .map_err(VOTableError::ParseFloat)
                    .map(|r| (l, r))
                })
            })
            .collect::<Result<Vec<(f64, f64)>, VOTableError>>()?,
        ),
        _ => {
          return Err(VOTableError::Custom(format!(
            "Unexpected Array type: {:?}",
            self
          )))
        }
      },
    ))
  }

  pub fn serialize_seed<S>(&self, value: &VOTableValue, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match value {
      VOTableValue::Null =>
        match self {
          Schema::Bool => serializer.serialize_u8(b'?'),
          Schema::Byte { null } => serializer.serialize_u8(null.unwrap_or(u8::MAX)),
          Schema::Short { null } => serializer.serialize_i16(null.unwrap_or(i16::MIN)),
          Schema::Int { null } => serializer.serialize_i32(null.unwrap_or(i32::MIN)),
          Schema::Long { null } => serializer.serialize_i64(null.unwrap_or(i64::MIN)),
          Schema::Float => serializer.serialize_f32(f32::NAN),
          Schema::Double => serializer.serialize_f64(f64::NAN),
          Schema::ComplexFloat => [f32::NAN, f32::NAN].serialize(serializer),
          Schema::ComplexDouble => [f64::NAN, f64::NAN].serialize(serializer),
          Schema::CharASCII => serializer.serialize_u8(b'\0'),
          Schema::CharUnicode => serializer.serialize_char('\0'),
          Schema::FixedLengthStringASCII { n_chars } => serialize_fixed_length_array(serializer, *n_chars, &vec![0; *n_chars]),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, (*n_chars) << 1, &vec![0; (*n_chars) << 1]),
          Schema::VariableLengthStringASCII { n_chars_max: _ } => serialize_variable_length_array(serializer, &[0; 0]),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &[0; 0]),
          // Schema::Bytes => serializer.serialize_bytes([0_u8; 0].as_slice())
          Schema::FixedLengthArray { n_elems, elem_schema } => serialize_fixed_length_array(serializer, *n_elems, &vec![0; (*n_elems) * elem_schema.byte_len().unwrap()]),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema: _ } => serialize_variable_length_array(serializer, &[0; 0]),
          Schema::Bit => serializer.serialize_u8(0_u8),
          Schema::FixedLengthBitArray { n_bits } => serialize_fixed_length_array(serializer, (7 + *n_bits)/ 8, &vec![0; (7 + *n_bits)/ 8]),
          Schema::VariableLengthBitArray { n_bits_max: _ } => serialize_variable_length_array(serializer, &[0_u8; 0]),
        }
      VOTableValue::Bool(v) => {
        assert!(matches!(self, Schema::Bool));
        serializer.serialize_u8(*v as u8)
      },
      VOTableValue::Byte(v) => {
        match self {
          Schema::Byte { .. } => serializer.serialize_u8(*v),
          Schema::Short { .. } => serializer.serialize_i16(*v as i16),
          Schema::Int { .. } => serializer.serialize_i32(*v as i32),
          Schema::Long { .. } => serializer.serialize_i64(*v as i64),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Byte with schema: {:?}", self)))
        }
      },
      VOTableValue::Short(v) => {
        match self {
          Schema::Short { .. } => serializer.serialize_i16(*v),
          Schema::Int { .. } => serializer.serialize_i32(*v as i32),
          Schema::Long { .. } => serializer.serialize_i64(*v as i64),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Short with schema: {:?}", self)))
        }
      },
      VOTableValue::Int(v) => {
        match self {
          Schema::Int { .. } => serializer.serialize_i32(*v),
          Schema::Long { .. } => serializer.serialize_i64(*v as i64),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Int with schema: {:?}", self)))
        }
      },
      VOTableValue::Long(v) => {
        match self {
          Schema::Long { .. } => serializer.serialize_i64(*v),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Long with schema: {:?}", self)))
        }
      },
      VOTableValue::Float(v) => {
        match self {
          Schema::Float => serializer.serialize_f32(*v),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Float with schema: {:?}", self)))
        }
      },
      VOTableValue::Double(v) => {
        match self {
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v),
          _ => Err(S::Error::custom(format!("Value of type Double with schema: {:?}", self)))
        }
      },
      VOTableValue::ComplexFloat((l, r)) =>
        match self {
          Schema::ComplexFloat => [*l, *r].serialize(serializer),
          Schema::ComplexDouble => [*l as f64, *r as f64].serialize(serializer),
          _ => Err(S::Error::custom(format!("Value of type ComplexDouble with schema: {:?}", self)))
        }
      VOTableValue::ComplexDouble((l, r)) =>
        match self {
          Schema::ComplexFloat => [*l as f32, *r as f32].serialize(serializer),
          Schema::ComplexDouble => [*l, *r].serialize(serializer),
          _ => Err(S::Error::custom(format!("Value of type ComplexDouble with schema: {:?}", self)))
        }
      VOTableValue::CharASCII(v) => {
        match self {
          Schema::CharASCII => serializer.serialize_u8(*v as u8),
          Schema::CharUnicode => serializer.serialize_char(*v),
          Schema::FixedLengthStringASCII { n_chars } => serialize_fixed_length_array(serializer, *n_chars, v.to_string().as_bytes()),
          Schema::VariableLengthStringASCII { n_chars_max: _ } => serialize_variable_length_array(serializer, v.to_string().as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, *n_chars, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          _ => Err(S::Error::custom(format!("Value of type CharASCII with schema: {:?}", self)))
        }
      },
      VOTableValue::CharUnicode(v) => {
        match self {
          Schema::CharASCII => serializer.serialize_u8(*v as u8),
          Schema::CharUnicode => serializer.serialize_char(*v),
          Schema::FixedLengthStringASCII { n_chars } => serialize_fixed_length_array(serializer, *n_chars, v.to_string().as_bytes()),
          Schema::VariableLengthStringASCII { n_chars_max: _ } => serialize_variable_length_array(serializer, v.to_string().as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, *n_chars, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          _ => Err(S::Error::custom(format!("Value of type CharUnicode with schema: {:?}", self)))
        }
      },
      VOTableValue::String(s) =>
        match &self {
          Schema::FixedLengthStringASCII { n_chars } => serialize_fixed_length_array(serializer, *n_chars, s.as_bytes()),
          Schema::VariableLengthStringASCII { n_chars_max: _ } => serialize_variable_length_array(serializer, s.as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, *n_chars, &encode_ucs2(s.as_str()).map_err(S::Error::custom)?),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &encode_ucs2(s.as_str()).map_err(S::Error::custom)?),
          _ => Err(S::Error::custom(format!("Wrong schema associated to String. Actual: {:?}. Expected: \
          FixedLengthStringASCII, VariableLengthStringASCII\
          FixedLengthStringUnicode or VariableLengthStringUnicode.", &self)))
        }
      VOTableValue::BitArray(bitvec) => {
        let v: Vec<u8> = BV::clone(&bitvec.0).into_vec();
        match &self {
          Schema::FixedLengthBitArray { n_bits} => {
            let n_bytes = (7 + *n_bits) / 8;
            if n_bytes != v.len() {
              return Err(S::Error::custom(format!("Wrong number of bytes in BitArray. Actual: {}. Expected: {}.", v.len(), n_bytes)));
            }
            serialize_fixed_length_array(serializer, v.len(), &v)
          },
          Schema::VariableLengthBitArray { n_bits_max: _ } => serialize_variable_length_array(serializer, &v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to BitArray. Actual: {:?}. Expected: FixedLengthBitArray or VariableLengthBitArray.", &self)))
        }
      }
      VOTableValue::ByteArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Byte { .. }) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _,  elem_schema } if matches!(elem_schema.as_ref(), &Schema::Byte { .. }) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ByteArray. Actual: {:?}. Expected: FixedLengthArray(Byte) or VariableLengthArray(Byte).", &self)))
        }
      VOTableValue::ShortArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Short { .. }) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Short { .. }) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ShortArray. Actual: {:?}. Expected: FixedLengthArray(Short) or VariableLengthArray(Short).", &self)))
        }
      VOTableValue::IntArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Int { .. }) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Int { .. }) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to IntArray. Actual: {:?}. Expected: FixedLengthArray(Int) or VariableLengthArray(Int).", &self)))
        }
      VOTableValue::LongArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Long { .. }) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Long { .. }) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to LongArray. Actual: {:?}. Expected: FixedLengthArray(Long) or VariableLengthArray(Long).", &self)))
        }
      VOTableValue::FloatArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Float) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Float) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to FloatArray. Actual: {:?}. Expected: FixedLengthArray(Float) or VariableLengthArray(Float).", &self)))
        }
      VOTableValue::DoubleArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Double) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Double) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to DoubleArray. Actual: {:?}. Expected: FixedLengthArray(Double) or VariableLengthArray(Double).", &self)))
        }
      VOTableValue::ComplexFloatArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::ComplexFloat) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::ComplexFloat) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ComplexFloatArray. Actual: {:?}. Expected: FixedLengthArray(ComplexFloat) or VariableLengthArray(ComplexFloat).", &self)))
        }
      VOTableValue::ComplexDoubleArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::ComplexDouble) =>
            serialize_fixed_length_array(serializer, *n_elems, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::ComplexDouble) =>
            serialize_variable_length_array(serializer, v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ComplexDoubleArray. Actual: {:?}. Expected: FixedLengthArray(ComplexDouble) or VariableLengthArray(ComplexDouble).", &self)))
        },
    }
  }

  pub fn replace_by_proper_value_if_necessary(
    &self,
    value: &mut VOTableValue,
  ) -> Result<(), String> {
    let new_val = match value {
      VOTableValue::Null => None,
      VOTableValue::Bool(_) => None,
      VOTableValue::Byte(v) => match self {
        Schema::Byte { .. } => None,
        Schema::Short { .. } => Some(VOTableValue::Short(*v as i16)),
        Schema::Int { .. } => Some(VOTableValue::Int(*v as i32)),
        Schema::Long { .. } => Some(VOTableValue::Long(*v as i64)),
        Schema::Float => Some(VOTableValue::Float(*v as f32)),
        Schema::Double => Some(VOTableValue::Double(*v as f64)),
        _ => return Err(format!("Value of type Byte with schema: {:?}", self)),
      },
      VOTableValue::Short(v) => match self {
        Schema::Short { .. } => None,
        Schema::Int { .. } => Some(VOTableValue::Int(*v as i32)),
        Schema::Long { .. } => Some(VOTableValue::Long(*v as i64)),
        Schema::Float => Some(VOTableValue::Float(*v as f32)),
        Schema::Double => Some(VOTableValue::Double(*v as f64)),
        _ => return Err(format!("Value of type Short with schema: {:?}", self)),
      },
      VOTableValue::Int(v) => match self {
        Schema::Int { .. } => None,
        Schema::Long { .. } => Some(VOTableValue::Long(*v as i64)),
        Schema::Float => Some(VOTableValue::Float(*v as f32)),
        Schema::Double => Some(VOTableValue::Double(*v as f64)),
        _ => return Err(format!("Value of type Int with schema: {:?}", self)),
      },
      VOTableValue::Long(v) => match self {
        Schema::Long { .. } => None,
        Schema::Float => Some(VOTableValue::Float(*v as f32)),
        Schema::Double => Some(VOTableValue::Double(*v as f64)),
        _ => return Err(format!("Value of type Long with schema: {:?}", self)),
      },
      VOTableValue::Float(v) => match self {
        Schema::Float => None,
        Schema::Double => Some(VOTableValue::Double(*v as f64)),
        _ => return Err(format!("Value of type Float with schema: {:?}", self)),
      },
      VOTableValue::Double(v) => match self {
        Schema::Float => Some(VOTableValue::Float(*v as f32)),
        Schema::Double => None,
        _ => return Err(format!("Value of type Double with schema: {:?}", self)),
      },
      VOTableValue::ComplexFloat((l, r)) => match self {
        Schema::ComplexFloat => None,
        Schema::ComplexDouble => Some(VOTableValue::ComplexDouble((*l as f64, *r as f64))),
        _ => {
          return Err(format!(
            "Value of type ComplexDouble with schema: {:?}",
            self
          ))
        }
      },
      VOTableValue::ComplexDouble((l, r)) => match self {
        Schema::ComplexFloat => Some(VOTableValue::ComplexFloat((*l as f32, *r as f32))),
        Schema::ComplexDouble => None,
        _ => {
          return Err(format!(
            "Value of type ComplexDouble with schema: {:?}",
            self
          ))
        }
      },
      VOTableValue::CharASCII(v) => match self {
        Schema::CharASCII => None,
        Schema::CharUnicode => Some(VOTableValue::CharUnicode(*v)),
        Schema::FixedLengthStringASCII { n_chars: 1 }
        | Schema::VariableLengthStringASCII { n_chars_max: _ }
        | Schema::FixedLengthStringUnicode { n_chars: 1 }
        | Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          Some(VOTableValue::String(String::from(*v)))
        }
        _ => return Err(format!("Value of type CharASCII with schema: {:?}", self)),
      },
      VOTableValue::CharUnicode(v) => match self {
        Schema::CharASCII => Some(VOTableValue::CharASCII(*v)),
        Schema::CharUnicode => None,
        Schema::FixedLengthStringASCII { n_chars: 1 }
        | Schema::VariableLengthStringASCII { n_chars_max: _ }
        | Schema::FixedLengthStringUnicode { n_chars: 1 }
        | Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          Some(VOTableValue::String(String::from(*v)))
        }
        _ => return Err(format!("Value of type CharUnicode with schema: {:?}", self)),
      },
      VOTableValue::String(s) => {
        if s.is_empty() {
          match &self {
            Schema::FixedLengthStringASCII { .. }
            | Schema::VariableLengthStringASCII { .. }
            | Schema::FixedLengthStringUnicode { .. }
            | Schema::VariableLengthStringUnicode { .. } => None,
            _ => Some(VOTableValue::Null),
          }
        } else {
          None
        }
      }
      VOTableValue::BitArray(_) => None,
      VOTableValue::ByteArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::ShortArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::IntArray(_) => None,  // TODO: convert array if not the right type...
      VOTableValue::LongArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::FloatArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::DoubleArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::ComplexFloatArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::ComplexDoubleArray(_) => None, // TODO: convert array if not the right type...
    };
    if let Some(new_val) = new_val {
      let _ = std::mem::replace(value, new_val);
    }
    Ok(())
  }
}

fn serialize_fixed_length_array<T, S>(serializer: S, len: usize, v: &[T]) -> Result<S::Ok, S::Error>
where
  T: Serialize,
  S: Serializer,
{
  let mut seq = serializer.serialize_tuple(len)?;
  for byte in v {
    seq.serialize_element(byte)?;
  }
  seq.end()
}

fn serialize_variable_length_array<T, S>(serializer: S, v: &[T]) -> Result<S::Ok, S::Error>
where
  T: Serialize,
  S: Serializer,
{
  let mut seq = serializer.serialize_seq(Some(v.len()))?;
  for byte in v {
    seq.serialize_element(byte)?;
  }
  seq.end()
}

impl From<&Field> for Schema {
  fn from(field: &Field) -> Self {
    // For all expect Bits, CharASCII, CharUNICODE
    fn from_regulartypeschema_and_arraysize(arraysize: &ArraySize, schema: Schema) -> Schema {
      match arraysize {
        ArraySize::Fixed1D { size } => Schema::FixedLengthArray {
          n_elems: *size as usize,
          elem_schema: Box::new(schema),
        },
        ArraySize::FixedND { sizes } => {
          let mut schema = schema;
          for size in sizes.iter().cloned() {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          schema
        }
        ArraySize::Variable1D => Schema::VariableLengthArray {
          n_elems_max: None,
          elem_schema: Box::new(schema),
        },
        ArraySize::VariableND { sizes } => {
          let mut schema = schema;
          for size in sizes.iter().cloned() {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: None,
            elem_schema: Box::new(schema),
          }
        }
        ArraySize::VariableWithUpperLimit1D { upper_limit } => Schema::VariableLengthArray {
          n_elems_max: Some(*upper_limit as usize),
          elem_schema: Box::new(schema),
        },
        ArraySize::VariableWithUpperLimitND { sizes, upper_limit } => {
          let mut schema = schema;
          for size in sizes.iter().cloned() {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: Some(*upper_limit as usize),
            elem_schema: Box::new(schema),
          }
        }
      }
    }

    match (field.datatype, &field.arraysize) {
      (Datatype::Logical, None) => Schema::Bool,
      (Datatype::Bit, None) => Schema::Bit,
      (Datatype::Byte, None) => Schema::Byte {
        null: field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default(), // i.e. null = None
      },
      (Datatype::ShortInt, None) => Schema::Short {
        null: field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default(), // i.e. null = None
      },
      (Datatype::Int, None) => Schema::Int {
        null: field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default(), // i.e. null = None
      },
      (Datatype::LongInt, None) => Schema::Long {
        null: field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default(), // i.e. null = None
      },
      (Datatype::CharASCII, None) => Schema::CharASCII,
      (Datatype::CharUnicode, None) => Schema::CharUnicode,
      (Datatype::Float, None) => Schema::Float,
      (Datatype::Double, None) => Schema::Double,
      (Datatype::ComplexFloat, None) => Schema::ComplexFloat,
      (Datatype::ComplexDouble, None) => Schema::ComplexDouble,
      // Char/String
      (Datatype::CharASCII, Some(size)) => match size {
        ArraySize::Fixed1D { size } => Schema::FixedLengthStringASCII {
          n_chars: *size as usize,
        },
        ArraySize::FixedND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringASCII {
            n_chars: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          schema
        }
        ArraySize::Variable1D => Schema::VariableLengthStringASCII { n_chars_max: None },
        ArraySize::VariableND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringASCII {
            n_chars: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: None,
            elem_schema: Box::new(schema),
          }
        }
        ArraySize::VariableWithUpperLimit1D { upper_limit } => Schema::VariableLengthStringASCII {
          n_chars_max: Some(*upper_limit as usize),
        },
        ArraySize::VariableWithUpperLimitND { sizes, upper_limit } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringASCII {
            n_chars: it.next().unwrap_or(0) as usize,
          };
          for size in sizes.iter().cloned() {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: Some(*upper_limit as usize),
            elem_schema: Box::new(schema),
          }
        }
      },
      (Datatype::CharUnicode, Some(size)) => match size {
        ArraySize::Fixed1D { size } => Schema::FixedLengthStringUnicode {
          n_chars: *size as usize,
        },
        ArraySize::FixedND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringUnicode {
            n_chars: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          schema
        }
        ArraySize::Variable1D => Schema::VariableLengthStringUnicode { n_chars_max: None },
        ArraySize::VariableND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringUnicode {
            n_chars: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: None,
            elem_schema: Box::new(schema),
          }
        }
        ArraySize::VariableWithUpperLimit1D { upper_limit } => {
          Schema::VariableLengthStringUnicode {
            n_chars_max: Some(*upper_limit as usize),
          }
        }
        ArraySize::VariableWithUpperLimitND { sizes, upper_limit } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringUnicode {
            n_chars: it.next().unwrap_or(0) as usize,
          };
          for size in sizes.iter().cloned() {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: Some(*upper_limit as usize),
            elem_schema: Box::new(schema),
          }
        }
      },
      // Arrays
      (Datatype::Logical, Some(size)) => from_regulartypeschema_and_arraysize(size, Schema::Bool),
      (Datatype::Bit, Some(size)) => match size {
        ArraySize::Fixed1D { size } => Schema::FixedLengthBitArray {
          n_bits: *size as usize,
        },
        ArraySize::FixedND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthBitArray {
            n_bits: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          schema
        }
        ArraySize::Variable1D => Schema::VariableLengthBitArray { n_bits_max: None },
        ArraySize::VariableND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthBitArray {
            n_bits: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: None,
            elem_schema: Box::new(schema),
          }
        }
        ArraySize::VariableWithUpperLimit1D { upper_limit } => Schema::VariableLengthBitArray {
          n_bits_max: Some(*upper_limit as usize),
        },
        ArraySize::VariableWithUpperLimitND { sizes, upper_limit } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthBitArray {
            n_bits: it.next().unwrap_or(0) as usize,
          };
          for size in sizes.iter().cloned() {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          Schema::VariableLengthArray {
            n_elems_max: Some(*upper_limit as usize),
            elem_schema: Box::new(schema),
          }
        }
      },
      (Datatype::Byte, Some(size)) => {
        let null = field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default();
        let elem_schema = Schema::Byte { null };
        from_regulartypeschema_and_arraysize(size, elem_schema)
      }
      (Datatype::ShortInt, Some(size)) => {
        let null = field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default();
        let elem_schema = Schema::Short { null };
        from_regulartypeschema_and_arraysize(size, elem_schema)
      }
      (Datatype::Int, Some(size)) => {
        let null = field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default();
        let elem_schema = Schema::Int { null };
        from_regulartypeschema_and_arraysize(size, elem_schema)
      }
      (Datatype::LongInt, Some(size)) => {
        let null = field
          .null_value()
          .map(|null_str| null_str.parse())
          .transpose()
          .unwrap_or_default();
        let elem_schema = Schema::Long { null };
        from_regulartypeschema_and_arraysize(size, elem_schema)
      }
      (Datatype::Float, Some(size)) => from_regulartypeschema_and_arraysize(size, Schema::Float),
      (Datatype::Double, Some(size)) => from_regulartypeschema_and_arraysize(size, Schema::Double),
      (Datatype::ComplexFloat, Some(size)) => {
        from_regulartypeschema_and_arraysize(size, Schema::ComplexFloat)
      }
      (Datatype::ComplexDouble, Some(size)) => {
        from_regulartypeschema_and_arraysize(size, Schema::ComplexDouble)
      }
    }
  }
}

impl<'de> DeserializeSeed<'de> for &Schema {
  type Value = VOTableValue;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    match self {
      Schema::Bool => Ok(match <u8>::deserialize(deserializer)? {
        b'0' | b'f' | b'F' => VOTableValue::Bool(false),
        b'1' | b't' | b'T' => VOTableValue::Bool(true),
        _ => VOTableValue::Null,
      }),
      Schema::Bit => <u8>::deserialize(deserializer).map(|b| VOTableValue::Bool(b != 0)),
      Schema::Byte { null: None } => <u8>::deserialize(deserializer).map(VOTableValue::Byte),
      Schema::Short { null: None } => <i16>::deserialize(deserializer).map(VOTableValue::Short),
      Schema::Int { null: None } => <i32>::deserialize(deserializer).map(VOTableValue::Int),
      Schema::Long { null: None } => <i64>::deserialize(deserializer).map(VOTableValue::Long),
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
      Schema::CharASCII => deserializer
        .deserialize_u8(CharVisitor)
        .map(VOTableValue::CharASCII),
      Schema::CharUnicode => deserializer
        .deserialize_u16(CharVisitor)
        .map(VOTableValue::CharUnicode),
      Schema::FixedLengthStringASCII { n_chars } => {
        let n_bytes = *n_chars;
        let visitor = FixedLengthArrayVisitor::new(n_bytes);
        let bytes: Vec<u8> = deserializer.deserialize_tuple(n_bytes, visitor)?;
        String::from_utf8(bytes)
          .map_err(D::Error::custom)
          .map(VOTableValue::String)
      }
      Schema::FixedLengthStringUnicode { n_chars } => {
        let visitor = FixedLengthArrayVisitor::new(*n_chars);
        let bytes: Vec<u16> = deserializer.deserialize_tuple(*n_chars, visitor)?;
        decode_ucs2(bytes)
          .map_err(D::Error::custom)
          .map(VOTableValue::String)
      }
      Schema::VariableLengthStringASCII { n_chars_max } => {
        let visitor = VariableLengthArrayVisitor::new(*n_chars_max);
        let bytes: Vec<u8> = deserializer.deserialize_seq(visitor)?;
        String::from_utf8(bytes)
          .map_err(D::Error::custom)
          .map(VOTableValue::String)
      }
      Schema::VariableLengthStringUnicode { n_chars_max } => {
        let visitor = VariableLengthArrayVisitor::new(*n_chars_max);
        let bytes: Vec<u16> = deserializer.deserialize_seq(visitor)?;
        decode_ucs2(bytes)
          .map_err(D::Error::custom)
          .map(VOTableValue::String)
      }
      // Schema::Bytes => deserializer.deserialize_bytes(BytesVisitor).map(VOTableValue::Bytes),
      Schema::FixedLengthArray {
        n_elems,
        elem_schema,
      } => match elem_schema.as_ref() {
        Schema::Byte { .. } => {
          let visitor = FixedLengthArrayVisitor::<u8>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::ByteArray)
        }
        Schema::Short { .. } => {
          let visitor = FixedLengthArrayVisitor::<i16>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::ShortArray)
        }
        Schema::Int { .. } => {
          let visitor = FixedLengthArrayVisitor::<i32>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::IntArray)
        }
        Schema::Long { .. } => {
          let visitor = FixedLengthArrayVisitor::<i64>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::LongArray)
        }
        Schema::Float => {
          let visitor = FixedLengthArrayVisitor::<f32>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::FloatArray)
        }
        Schema::Double => {
          let visitor = FixedLengthArrayVisitor::<f64>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::DoubleArray)
        }
        Schema::ComplexFloat => {
          let visitor = FixedLengthArrayVisitor::<(f32, f32)>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::ComplexFloatArray)
        }
        Schema::ComplexDouble => {
          let visitor = FixedLengthArrayVisitor::<(f64, f64)>::new(*n_elems);
          deserializer
            .deserialize_tuple(*n_elems, visitor)
            .map(VOTableValue::ComplexDoubleArray)
        }
        _ => Err(D::Error::custom(format!(
          "Unexpected datatype in FixedLengthArray: {:?}",
          elem_schema
        ))),
      },
      Schema::VariableLengthArray {
        n_elems_max,
        elem_schema,
      } => match elem_schema.as_ref() {
        Schema::Byte { .. } => {
          let visitor = VariableLengthArrayVisitor::<u8>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::ByteArray)
        }
        Schema::Short { .. } => {
          let visitor = VariableLengthArrayVisitor::<i16>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::ShortArray)
        }
        Schema::Int { .. } => {
          let visitor = VariableLengthArrayVisitor::<i32>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::IntArray)
        }
        Schema::Long { .. } => {
          let visitor = VariableLengthArrayVisitor::<i64>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::LongArray)
        }
        Schema::Float => {
          let visitor = VariableLengthArrayVisitor::<f32>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::FloatArray)
        }
        Schema::Double => {
          let visitor = VariableLengthArrayVisitor::<f64>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::DoubleArray)
        }
        Schema::ComplexFloat => {
          let visitor = VariableLengthArrayVisitor::<(f32, f32)>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::ComplexFloatArray)
        }
        Schema::ComplexDouble => {
          let visitor = VariableLengthArrayVisitor::<(f64, f64)>::new(*n_elems_max);
          deserializer
            .deserialize_seq(visitor)
            .map(VOTableValue::ComplexDoubleArray)
        }
        _ => Err(D::Error::custom(format!(
          "Unexpected datatype in VariableLengthArray: {:?}",
          elem_schema
        ))),
      },
      Schema::FixedLengthBitArray { n_bits } => {
        let n_bytes = (7 + *n_bits) / 8;
        let visitor = FixedLengthArrayVisitor::<u8>::new(n_bytes);
        let bytes: Vec<u8> = deserializer.deserialize_tuple(n_bytes, visitor)?;
        Ok(VOTableValue::BitArray(BitVec(BV::from_vec(bytes))))
      }
      Schema::VariableLengthBitArray { n_bits_max } => {
        let visitor =
          VariableLengthArrayVisitor::<u8>::new((*n_bits_max).map(|n_bits| (n_bits + 7) / 8));
        let bytes: Vec<u8> = deserializer.deserialize_seq(visitor)?;
        Ok(VOTableValue::BitArray(BitVec(BV::from_vec(bytes))))
      }
    }
  }
}

pub fn decode_ucs2(bytes: Vec<u16>) -> Result<String, VOTableError> {
  let mut bytes_utf8 = vec![0_u8; bytes.len() << 1];
  let n = ucs2::decode(&bytes, &mut bytes_utf8).map_err(VOTableError::FromUCS2)?;
  bytes_utf8.truncate(n);
  String::from_utf8(bytes_utf8).map_err(VOTableError::FromUtf8)
}

pub fn encode_ucs2(s: &str) -> Result<Vec<u16>, VOTableError> {
  let mut ucs2_buff = vec![0_u16; s.len()];
  let n = ucs2::encode(s, &mut ucs2_buff).map_err(VOTableError::ToUCS2)?;
  ucs2_buff.truncate(n);
  Ok(ucs2_buff)
}
