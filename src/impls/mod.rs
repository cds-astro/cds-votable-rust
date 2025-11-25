use std::{
  fmt::{self, Display, Formatter, Write},
  marker::PhantomData,
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
  impls::visitors::{FixedLengthArraySeed, FixedLengthArrayVisitor, VariableLengthArrayVisitor},
  table::TableElem,
};

pub mod b64;
pub mod mem;
pub mod visitors;
use crate::impls::visitors::{
  FixedLengthArrayOfUTF8StringSeed, FixedLengthArrayPhantomSeed, FixedLengthUTF8StringSeed,
  FixedLengthUnicodeStringSeed,
};
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

pub struct FixedLengthStringUTF8(String);
impl Serialize for FixedLengthStringUTF8 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_fixed_length_array(serializer, self.0.len(), self.0.as_bytes())
  }
}
pub struct VariableLengthStringUTF8(String);
impl Serialize for VariableLengthStringUTF8 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_variable_length_array(serializer, self.0.as_bytes())
  }
}
pub struct FixedLengthStringUnicode(String);
impl Serialize for FixedLengthStringUnicode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_fixed_length_array(
      serializer,
      self.0.len(),
      &encode_ucs2(self.0.as_str()).map_err(S::Error::custom)?,
    )
  }
}
pub struct VariableLengthStringUnicode(String);
impl Serialize for VariableLengthStringUnicode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_variable_length_array(
      serializer,
      &encode_ucs2(self.0.as_str()).map_err(S::Error::custom)?,
    )
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
  StringArray(Vec<String>),
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
      VOTableValue::StringArray(v) => v.serialize(serializer), // depends on unicode vs regular utf8-strings ?
    }
  }
}

impl Display for VOTableValue {
  /// Use to write the field in TABLEDATA!!
  fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
    fn write_array<T: Display>(fmt: &mut Formatter, elems: &[T]) -> fmt::Result {
      let mut it = elems.iter();
      if let Some(e) = it.next() {
        fmt.write_fmt(format_args!("{}", e))?;
        for e in elems.iter() {
          fmt.write_fmt(format_args!(" {}", e))?;
        }
      }
      Ok(())
    }
    fn write_array_of_complex<T: Display>(fmt: &mut Formatter, elems: &[(T, T)]) -> fmt::Result {
      let mut it = elems.iter();
      if let Some(e) = it.next() {
        fmt.write_fmt(format_args!("{} {}", e.0, e.1))?;
        for e in elems.iter() {
          fmt.write_fmt(format_args!("{} {}", e.0, e.1))?;
        }
      }
      Ok(())
    }

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
      VOTableValue::ByteArray(v) => write_array(fmt, v.as_slice()),
      VOTableValue::ShortArray(v) => write_array(fmt, v.as_slice()),
      VOTableValue::IntArray(v) => write_array(fmt, v.as_slice()),
      VOTableValue::LongArray(v) => write_array(fmt, v.as_slice()),
      VOTableValue::FloatArray(v) => write_array(fmt, v.as_slice()),
      VOTableValue::DoubleArray(v) => write_array(fmt, v.as_slice()),
      VOTableValue::ComplexFloatArray(v) => write_array_of_complex(fmt, v.as_slice()),
      VOTableValue::ComplexDoubleArray(v) => write_array_of_complex(fmt, v.as_slice()),
      VOTableValue::StringArray(v) => {
        for s in v.iter() {
          fmt.write_str(&s)?;
        }
        Ok(())
      }
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
  FixedLengthStringUTF8 {
    n_bytes: usize,
  },
  FixedLengthStringUnicode {
    n_chars: usize,
  },
  VariableLengthStringUTF8 {
    /// Maximum number of characters the array may contain if declared as `INT*`), `None` if
    /// declared as `*`.
    n_bytes_max: Option<usize>,
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
      Schema::FixedLengthStringUTF8 { n_bytes } => Ok(n_bytes * size_of::<u8>()),
      Schema::FixedLengthStringUnicode { n_chars } => Ok(n_chars * size_of::<u16>()),
      Schema::VariableLengthStringUTF8 { n_bytes_max } => {
        Err((size_of::<u8>(), n_bytes_max.map(|n| n * size_of::<u8>())))
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

  /// In case of arrays (or arrays of arrays), returns the size, in bytes, of the array primitive elements.
  /// In case of arrays (or arrays or arrays of) of String, returns the size of the String (not the Char or UnicodeChar).
  /// Else (no array), this **must** provide the same result as `byte_len`.
  pub fn elem_byte_len(&self) -> Result<usize, (usize, Option<usize>)> {
    match self {
      Schema::FixedLengthArray {
        n_elems: _,
        elem_schema,
      }
      | Schema::VariableLengthArray {
        n_elems_max: _,
        elem_schema,
      } => elem_schema.elem_byte_len(),
      _ => self.byte_len(),
    }
  }

  /// Returns true is the element type (possibly inside an array or an array of array) is of type String.
  /// Arrays of those type of elements are concatenated instead of being black separated.
  pub fn contains_string_elem(&self) -> bool {
    match self {
      Schema::FixedLengthStringUTF8 { .. }
      | Schema::FixedLengthStringUnicode { .. }
      | Schema::VariableLengthStringUTF8 { .. }
      | Schema::VariableLengthStringUnicode { .. } => true,
      Schema::FixedLengthArray {
        n_elems: _,
        elem_schema,
      }
      | Schema::VariableLengthArray {
        n_elems_max: _,
        elem_schema,
      } => elem_schema.contains_string_elem(),
      _ => false,
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
                        _ =>
                          VOTableValue::Bool(s.to_lowercase().parse::<bool>()
                            .map_err(|_| VOTableError::Custom(
                            format!("Unable to parse boolean value. Expected: '0', '1', 't', 'f', 'T', 'F' or 'true', 'false'. Actual: '{}'", s))
                          )?),
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
        Schema::FixedLengthStringUTF8 { n_bytes: _ } => VOTableValue::String(s.to_owned()), // Pad witb \0 ??
        Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => VOTableValue::String(s.to_owned()),
        Schema::FixedLengthStringUnicode { n_chars: _ } => VOTableValue::String(s.to_owned()), // Pad witb \0 ??
        Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          VOTableValue::String(s.to_owned())
        }
        // Schema::Bytes => unreachable!() // only in binary mode?
        Schema::FixedLengthArray {
          n_elems,
          elem_schema,
        } => elem_schema.parse_fixed_length_array(*n_elems, s)?,
        Schema::VariableLengthArray {
          n_elems_max: _,
          elem_schema,
        } => elem_schema.parse_variable_length_array(s)?,
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

  /// The parent datatype is a fixed array of this type.
  /// Always returns a 1D array containing all elements (the multi-dimension array is just a view
  /// on top of a 1D-array).
  fn parse_fixed_length_array(
    &self,
    n_elems: usize,
    array_str: &str,
  ) -> Result<VOTableValue, VOTableError> {
    match self {
      Schema::Byte { .. }
      | Schema::Short { .. }
      | Schema::Int { .. }
      | Schema::Long { .. }
      | Schema::Float { .. }
      | Schema::Double { .. }
      | Schema::ComplexFloat { .. }
      | Schema::ComplexDouble { .. } => {
        self
          .parse_array_of_number(array_str)
          .and_then(|(n_actual_elems, value)| {
            if n_actual_elems == n_elems {
              Ok(value)
            } else {
              Err(VOTableError::Custom(format!(
                "Wrong number of fixed length array elements. Expected: {}. Actual: {}.",
                n_elems, n_actual_elems
              )))
            }
          })
      }
      Schema::FixedLengthStringUTF8 { .. } | Schema::FixedLengthStringUnicode { .. } => {
        self.parse_fixed_array_of_char(n_elems, array_str)
      }
      Schema::FixedLengthArray {
        n_elems: n_sub_elems,
        elem_schema,
      } => elem_schema.parse_fixed_length_array(n_elems * n_sub_elems, array_str),
      Schema::VariableLengthStringUnicode { .. } | Schema::VariableLengthStringUTF8 { .. } => {
        Err(VOTableError::Custom(String::from(
          "Fixed arrays of variable length strings not supported in VOTable.",
        )))
      }
      Schema::VariableLengthArray { .. } => Err(VOTableError::Custom(String::from(
        "Fixed arrays of variable arrays not supported in VOTable.",
      ))),
      /*
      Schema::Bool => {}
      Schema::Bit => {}
      Schema::CharASCII => {}
      Schema::CharUnicode => {}
      Schema::FixedLengthBitArray { .. } => {}
      Schema::VariableLengthBitArray { .. } => {}
      */
      _ => Err(VOTableError::Custom(format!(
        "Fixed arrays of {:?} not supported (yet?).",
        self
      ))),
    }
  }

  /// The parent datatype is a variable array of this type.
  fn parse_variable_length_array(&self, array_str: &str) -> Result<VOTableValue, VOTableError> {
    match self {
      Schema::Byte { .. }
      | Schema::Short { .. }
      | Schema::Int { .. }
      | Schema::Long { .. }
      | Schema::Float { .. }
      | Schema::Double { .. }
      | Schema::ComplexFloat { .. }
      | Schema::ComplexDouble { .. } => self.parse_array_of_number(array_str).map(|(_n, v)| v),
      Schema::FixedLengthStringUTF8 { .. } | Schema::FixedLengthStringUnicode { .. } => {
        self.parse_variable_array_of_char(array_str)
      }
      Schema::FixedLengthArray {
        n_elems: _,
        elem_schema,
      } => elem_schema.parse_variable_length_array(array_str),
      Schema::VariableLengthStringUnicode { .. } | Schema::VariableLengthStringUTF8 { .. } => {
        Err(VOTableError::Custom(String::from(
          "Variable arrays of variable length strings not supported in VOTable.",
        )))
      }
      Schema::VariableLengthArray { .. } => Err(VOTableError::Custom(String::from(
        "Variable arrays of variable arrays not supported in VOTable.",
      ))),
      /*
      Schema::Bool => {}
      Schema::Bit => {}
      Schema::CharASCII => {}
      Schema::CharUnicode => {}
      Schema::FixedLengthBitArray { .. } => {}
      Schema::VariableLengthBitArray { .. } => {}
      */
      _ => Err(VOTableError::Custom(format!(
        "Variable arrays of {:?} not supported (yet?).",
        self
      ))),
    }
  }

  /// Blank separated array
  fn parse_array_of_number(&self, array_str: &str) -> Result<(usize, VOTableValue), VOTableError> {
    // let data: Vec<> = s.trim().split(' ').map(|s| elem_schema.value_from_str(s)).collect()?;
    let elems: Vec<&str> = array_str.trim().split(' ').collect();
    let n_elems = elems.len();
    match self {
      // Schema::Bool => VOTableValue::Bool() elem_it.map(s.parse().map_err(VOTableError::ParseBool)).collect()?,
      Schema::Byte { .. } => elems
        .iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<u8>, VOTableError>>()
        .map(VOTableValue::ByteArray),
      Schema::Short { .. } => elems
        .iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<i16>, VOTableError>>()
        .map(VOTableValue::ShortArray),
      Schema::Int { .. } => elems
        .iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<i32>, VOTableError>>()
        .map(VOTableValue::IntArray),
      Schema::Long { .. } => elems
        .iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<i64>, VOTableError>>()
        .map(VOTableValue::LongArray),
      Schema::Float => elems
        .iter()
        .map(|s| s.parse().map_err(VOTableError::ParseFloat))
        .collect::<Result<Vec<f32>, VOTableError>>()
        .map(VOTableValue::FloatArray),
      Schema::Double => elems
        .iter()
        .map(|s| s.parse().map_err(VOTableError::ParseFloat))
        .collect::<Result<Vec<f64>, VOTableError>>()
        .map(VOTableValue::DoubleArray),
      Schema::ComplexFloat => elems
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
        .collect::<Result<Vec<(f32, f32)>, VOTableError>>()
        .map(VOTableValue::ComplexFloatArray),
      Schema::ComplexDouble => elems
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
        .collect::<Result<Vec<(f64, f64)>, VOTableError>>()
        .map(VOTableValue::ComplexDoubleArray),
      _ => Err(VOTableError::Custom(format!(
        "Unexpected Array type: {:?}",
        self
      ))),
    }
    .map(|value| (n_elems, value))
  }

  /// Returns a fixed length array of this Schema, which must be of fixed length String (no separator).
  pub fn parse_fixed_array_of_char(
    &self,
    n_elems: usize,
    array_str: &str,
  ) -> Result<VOTableValue, VOTableError> {
    match self {
      Schema::FixedLengthStringUTF8 { n_bytes } => {
        if array_str.len() == n_elems * *n_bytes {
          Ok(VOTableValue::StringArray(
            array_str
              .as_bytes()
              .chunks(*n_bytes)
              .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
              .collect(),
          ))
        } else {
          Err(VOTableError::Custom(format!(
            "Unexpected string len in array of {} strings of length {}. Expected: {}. Actual: {}. String: '{}'",
            n_elems, n_bytes, n_elems * *n_bytes, array_str.len(), &array_str
          )))
        }
      }
      Schema::FixedLengthStringUnicode { n_chars } => {
        let utf16: Vec<u16> = array_str.encode_utf16().collect::<Vec<u16>>();
        let s: Vec<String> = utf16
          .as_slice()
          .chunks(*n_chars)
          .map(String::from_utf16_lossy)
          .collect();
        if s.len() == n_elems {
          Ok(VOTableValue::StringArray(s))
        } else {
          Err(VOTableError::Custom(format!(
            "Unexpected number of elements in string array. Expected: {}. Actual: {}. String: '{}'",
            n_elems,
            s.len(),
            &array_str
          )))
        }
      }
      _ => unreachable!(),
    }
  }

  /// Returns a variable length array of this Schema, which must be of fixed length String (no separator).
  fn parse_variable_array_of_char(&self, array_str: &str) -> Result<VOTableValue, VOTableError> {
    match self {
      Schema::FixedLengthStringUTF8 { n_bytes } => Ok(VOTableValue::StringArray(
        array_str
          .as_bytes()
          .chunks(*n_bytes)
          .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
          .collect(),
      )),
      Schema::FixedLengthStringUnicode { n_chars } => {
        let utf16: Vec<u16> = array_str.encode_utf16().collect::<Vec<u16>>();
        let s: Vec<String> = utf16
          .as_slice()
          .chunks(*n_chars)
          .map(String::from_utf16_lossy)
          .collect();
        Ok(VOTableValue::StringArray(s))
      }
      _ => unreachable!(),
    }
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
          Schema::FixedLengthStringUTF8 { n_bytes: n_chars } => serialize_fixed_length_array(serializer, *n_chars, &vec![0; *n_chars]),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, (*n_chars) << 1, &vec![0; (*n_chars) << 1]),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, &[0; 0]),
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
          _ => Err(S::Error::custom(format!("Value of type Byte with schema: {:?}", self))) }
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
          Schema::FixedLengthStringUTF8 { n_bytes: n_chars } => serialize_fixed_length_array(serializer, *n_chars, v.to_string().as_bytes()),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, v.to_string().as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, *n_chars, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          _ => Err(S::Error::custom(format!("Value of type CharASCII with schema: {:?}", self)))
        }
      },
      VOTableValue::CharUnicode(v) => {
        match self {
          Schema::CharASCII => serializer.serialize_u8(*v as u8),
          Schema::CharUnicode => serializer.serialize_char(*v),
          Schema::FixedLengthStringUTF8 { n_bytes: n_chars } => serialize_fixed_length_array(serializer, *n_chars, v.to_string().as_bytes()),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, v.to_string().as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, *n_chars, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom)?),
          _ => Err(S::Error::custom(format!("Value of type CharUnicode with schema: {:?}", self)))
        }
      },
      VOTableValue::String(s) =>
        match &self {
          Schema::FixedLengthStringUTF8 { n_bytes: n_chars } => serialize_fixed_length_array(serializer, *n_chars, s.as_bytes()),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, s.as_bytes()),
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
      VOTableValue::StringArray(v) => {
        match &self {
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthStringUTF8 { .. }) => {
            serialize_fixed_length_array(serializer, *n_elems, v)
          },
          Schema::FixedLengthArray { n_elems, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthStringUnicode { .. }) => {
            serialize_fixed_length_array(serializer, *n_elems, v)
          },
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthStringUTF8 { .. }) => {
            serialize_variable_length_array(serializer, v)
          },
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthStringUnicode { .. }) => {
            serialize_variable_length_array(serializer, v)
          }
          _ => Err(S::Error::custom(format!("Wrong schema associated to StringArray. Actual: {:?}. Expected: FixedLengthArray(FixedLengthStringASCII) or VariableLengthArray(FixedLengthStringUnicode).", &self)))
        }
      }
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
        Schema::FixedLengthStringUTF8 { n_bytes: 1 }
        | Schema::VariableLengthStringUTF8 { n_bytes_max: _ }
        | Schema::FixedLengthStringUnicode { n_chars: 1 }
        | Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          Some(VOTableValue::String(String::from(*v)))
        }
        _ => return Err(format!("Value of type CharASCII with schema: {:?}", self)),
      },
      VOTableValue::CharUnicode(v) => match self {
        Schema::CharASCII => Some(VOTableValue::CharASCII(*v)),
        Schema::CharUnicode => None,
        Schema::FixedLengthStringUTF8 { n_bytes: 1 }
        | Schema::VariableLengthStringUTF8 { n_bytes_max: _ }
        | Schema::FixedLengthStringUnicode { n_chars: 1 }
        | Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          Some(VOTableValue::String(String::from(*v)))
        }
        _ => return Err(format!("Value of type CharUnicode with schema: {:?}", self)),
      },
      VOTableValue::String(s) => {
        if s.is_empty() {
          match &self {
            Schema::FixedLengthStringUTF8 { .. }
            | Schema::VariableLengthStringUTF8 { .. }
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
      VOTableValue::StringArray(_) => None, // TODO: convert array if not the right type...
                                           // ICI, EN FAIT, FAIRE LA DIFFERENCE ENTRE StringUTF8 et StringUnidocde en fonction du schema!!!
                                           // SERIALISATIOn DIFFERENTE!!
                                           // GERE lES TABLEAUX!!
                                           // Ou pas...
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
        ArraySize::Fixed1D { size } => Schema::FixedLengthStringUTF8 {
          n_bytes: *size as usize,
        },
        ArraySize::FixedND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringUTF8 {
            n_bytes: it.next().unwrap_or(0) as usize,
          };
          for size in it {
            schema = Schema::FixedLengthArray {
              n_elems: size as usize,
              elem_schema: Box::new(schema),
            }
          }
          schema
        }
        ArraySize::Variable1D => Schema::VariableLengthStringUTF8 { n_bytes_max: None },
        ArraySize::VariableND { sizes } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringUTF8 {
            n_bytes: it.next().unwrap_or(0) as usize,
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
        ArraySize::VariableWithUpperLimit1D { upper_limit } => Schema::VariableLengthStringUTF8 {
          n_bytes_max: Some(*upper_limit as usize),
        },
        ArraySize::VariableWithUpperLimitND { sizes, upper_limit } => {
          let mut it = sizes.iter().cloned();
          let mut schema = Schema::FixedLengthStringUTF8 {
            n_bytes: it.next().unwrap_or(0) as usize,
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
      Schema::FixedLengthStringUTF8 { n_bytes } => {
        FixedLengthUTF8StringSeed::new(*n_bytes)
          .deserialize(deserializer)
          .map(VOTableValue::String)
        /*let visitor = FixedLengthArrayVisitor::new(*n_bytes);
        deserializer
          .deserialize_tuple(*n_bytes, visitor)
          .and_then(|bytes| String::from_utf8(bytes).map_err(D::Error::custom))
          .map(VOTableValue::String)*/
        /*deserializer
        .deserialize_tuple(*n_bytes, FixedLengthUTF8StringVisitor::new(*n_bytes))
        .map(VOTableValue::String)*/
      }
      Schema::FixedLengthStringUnicode { n_chars } => {
        /*let visitor = FixedLengthArrayVisitor::new(*n_chars);
        let bytes: Vec<u16> = deserializer.deserialize_tuple(*n_chars, visitor)?;
        decode_ucs2(bytes)
          .map_err(D::Error::custom)
          .map(VOTableValue::String)*/
        /*deserializer
        .deserialize_tuple(*n_chars, FixedLengthUnicodeStringVisitor::new(*n_chars))
        .map(VOTableValue::String)*/
        FixedLengthUnicodeStringSeed::new(*n_chars)
          .deserialize(deserializer)
          .map(VOTableValue::String)
      }
      Schema::VariableLengthStringUTF8 {
        n_bytes_max: n_chars_max,
      } => {
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
        Schema::Byte { .. } => FixedLengthArrayPhantomSeed::<u8>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::ByteArray),
        Schema::Short { .. } => FixedLengthArrayPhantomSeed::<i16>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::ShortArray),
        Schema::Int { .. } => FixedLengthArrayPhantomSeed::<i32>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::IntArray),
        Schema::Long { .. } => FixedLengthArrayPhantomSeed::<i64>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::LongArray),
        Schema::Float => FixedLengthArrayPhantomSeed::<f32>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::FloatArray),
        Schema::Double => FixedLengthArrayPhantomSeed::<f64>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::DoubleArray),
        Schema::ComplexFloat => FixedLengthArrayPhantomSeed::<(f32, f32)>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::ComplexFloatArray),
        Schema::ComplexDouble => FixedLengthArrayPhantomSeed::<(f64, f64)>::new(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::ComplexDoubleArray),
        Schema::FixedLengthStringUTF8 { n_bytes } => {
          FixedLengthArrayOfUTF8StringSeed::new(*n_elems, *n_bytes)
            .deserialize(deserializer)
            .map(VOTableValue::StringArray)
        }
        Schema::FixedLengthStringUnicode { n_chars } => {
          FixedLengthArrayOfUTF8StringSeed::new(*n_elems, *n_chars)
            .deserialize(deserializer)
            .map(VOTableValue::StringArray)
        }
        Schema::FixedLengthArray {
          n_elems: sub_n_elems,
          elem_schema,
        } => {
          // We convert arrays of arrays in flat arrays
          let virtual_schema = Schema::FixedLengthArray {
            n_elems: *n_elems * *sub_n_elems,
            elem_schema: elem_schema.clone(),
          };
          virtual_schema.deserialize(deserializer)
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
        // FIXED UTF8 String!
        // FIXED Unicode STRING:
        Schema::FixedLengthArray {
          n_elems: sub_n_elems,
          elem_schema,
        } => {
          // NO!! here, we must multiply the number read in seq_access byt the fixed array length!!
          /*let virtual_schema = Schema::VariableLengthArray {
            n_elems_max: n_elems_max.map(|n_elems_max| n_elems_max * *sub_n_elems),
            elem_schema: elem_schema.clone(),
          };
          virtual_schema.deserialize(deserializer)
          */
          /*elem_schema.get_n_base_elems_racursive
          elem_schema.get_primitive_type_recursive
          Faire un array of array visitor??
          Faire un match sur primiteve type , seq_acces de FixedArray ...*/
          todo!()
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
