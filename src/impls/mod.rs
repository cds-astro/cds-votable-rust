use std::{
  cmp::Ordering,
  fmt::{self, Display, Formatter, Write},
  mem::size_of,
  slice::Iter,
};

use bitvec::{order::Msb0, vec::BitVec as BV};
use log::{trace, warn};
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
pub mod seeds;
pub mod visitors;

use crate::impls::{
  seeds::{
    new_fixed_length_array_of_boolean_seed, new_var_length_array_of_boolean_seed,
    new_var_length_of_fixed_len_array_of_boolean_seed, BooleanSead,
    FixedLengthArrayOfUTF8StringSeed, FixedLengthArrayOfUnidecodeStringSeed,
    FixedLengthArrayPhantomSeed, FixedLengthUTF8StringSeed, FixedLengthUnicodeStringSeed,
    VarLengthArrayOfUTF8StringSeed, VarLengthArrayOfUnicodeStringSeed, VarLengthArrayPhantomSeed,
    VarLengthUTF8StringSeed, VarLengthUnicodeStringSeed, VarLengthVectorOfVectorSeed,
    VarLengthVectorOfVectorSeedWithSeed,
  },
  visitors::CharVisitor,
};

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

pub struct OptBool<'a>(&'a Option<bool>);
impl<'a> Serialize for OptBool<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self.0 {
      Some(true) => serializer.serialize_u8(b'1'),
      Some(false) => serializer.serialize_u8(b'0'),
      None => serializer.serialize_u8(b'?'),
    }
  }
}

pub struct FixedLengthStringUTF8<'a>(&'a str);
impl<'a> Serialize for FixedLengthStringUTF8<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_fixed_length_array(serializer, self.0.as_bytes())
  }
}
pub struct VariableLengthStringUTF8<'a>(&'a str);
impl<'a> Serialize for VariableLengthStringUTF8<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_variable_length_array(serializer, self.0.as_bytes())
  }
}
pub struct FixedLengthStringUnicode<'a>(&'a str);
impl<'a> Serialize for FixedLengthStringUnicode<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_fixed_length_array(serializer, &encode_ucs2(self.0).map_err(S::Error::custom)?)
  }
}
pub struct VariableLengthStringUnicode<'a>(&'a str);
impl<'a> Serialize for VariableLengthStringUnicode<'a> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serialize_variable_length_array(serializer, &encode_ucs2(self.0).map_err(S::Error::custom)?)
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
  BooleanArray(Vec<Option<bool>>),
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
/// Here it is the implementation for Serde compatible data types.
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
      VOTableValue::BooleanArray(v) => v.serialize(serializer),
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
    fn write_bool_array(fmt: &mut Formatter, elems: &[Option<bool>]) -> fmt::Result {
      let mut it = elems.iter();
      if let Some(e) = it.next() {
        match e {
          Some(b) => fmt.write_fmt(format_args!("{}", b)),
          None => fmt.write_str("?"),
        }?;
        for e in elems.iter() {
          match e {
            Some(b) => fmt.write_fmt(format_args!(" {}", b)),
            None => fmt.write_str(" ?"),
          }?;
        }
      }
      Ok(())
    }
    fn write_array_of_complex<T: Display>(fmt: &mut Formatter, elems: &[(T, T)]) -> fmt::Result {
      let mut it = elems.iter();
      if let Some(e) = it.next() {
        fmt.write_fmt(format_args!("{} {}", e.0, e.1))?;
        for e in elems.iter() {
          fmt.write_fmt(format_args!(" {} {}", e.0, e.1))?;
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
      VOTableValue::BooleanArray(v) => write_bool_array(fmt, v.as_slice()),
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
  },
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

  /// Returns the fixed len of this schema, i.e. the fixed number of items it contains.
  /// * for primitive type, the value is `1`.
  /// * for String types, the value is `1`.
  /// * for fixed length arrays, the value is the number of elements in the array
  /// * for fixed length arrays of fixed length arrays, returns the products of the lenghts
  /// * raise an error for variable length types.
  pub fn fixed_len(&self) -> Result<usize, VOTableError> {
    match self {
      Schema::Bool
      | Schema::Bit
      | Schema::Byte { .. }
      | Schema::Short { .. }
      | Schema::Int { .. }
      | Schema::Long { .. }
      | Schema::Float
      | Schema::Double
      | Schema::ComplexFloat
      | Schema::ComplexDouble
      | Schema::CharASCII
      | Schema::CharUnicode
      | Schema::FixedLengthStringUTF8 { .. }
      | Schema::FixedLengthStringUnicode { .. } => Ok(1),
      Schema::VariableLengthStringUTF8 { .. } | Schema::VariableLengthStringUnicode { .. } => {
        Err(VOTableError::Custom(String::from(
          "`fixed_len` method not compatible with variable length Strings",
        )))
      }
      Schema::FixedLengthArray {
        n_elems,
        elem_schema,
      } => elem_schema.fixed_len().map(|len| len * *n_elems),
      Schema::VariableLengthArray { .. }
      | Schema::FixedLengthBitArray { .. }
      | Schema::VariableLengthBitArray { .. } => Err(VOTableError::Custom(String::from(
        "`fixed_len` method not compatible with variable length arrays",
      ))),
    }
  }

  /// For arrays (of arrays), returns the Schema of the primitive elements.
  pub fn primitive_schema(&self) -> Result<&Schema, VOTableError> {
    match self {
      Schema::Bool
      | Schema::Bit
      | Schema::Byte { .. }
      | Schema::Short { .. }
      | Schema::Int { .. }
      | Schema::Long { .. }
      | Schema::Float
      | Schema::Double
      | Schema::ComplexFloat
      | Schema::ComplexDouble
      | Schema::CharASCII
      | Schema::CharUnicode
      | Schema::FixedLengthStringUTF8 { .. }
      | Schema::FixedLengthStringUnicode { .. } => Ok(self),
      Schema::VariableLengthStringUTF8 { .. } | Schema::VariableLengthStringUnicode { .. } => Err(
        VOTableError::Custom(String::from("No primitive type for variable String")),
      ),
      Schema::FixedLengthArray {
        n_elems: _,
        elem_schema,
      } => elem_schema.primitive_schema(),
      Schema::VariableLengthArray { .. }
      | Schema::FixedLengthBitArray { .. }
      | Schema::VariableLengthBitArray { .. } => Err(VOTableError::Custom(String::from(
        "No primitive type for variable arrays",
      ))),
    }
  }

  /*
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
  }*/

  /// For arrays (of arrays), returns the Schema of the primitive elements and the total number
  /// of elements in fixed length array (of fixed length arrays), flattening possibly nested fixed length arrays.
  /// * String are considered as primitives here, and the size of the string is not included into
  ///  the returned array len.
  pub fn primitive_type_and_array_len(&self) -> Result<(&Schema, usize), VOTableError> {
    match self {
      Schema::Bool
      | Schema::Bit
      | Schema::Byte { .. }
      | Schema::Short { .. }
      | Schema::Int { .. }
      | Schema::Long { .. }
      | Schema::Float
      | Schema::Double
      | Schema::ComplexFloat
      | Schema::ComplexDouble
      | Schema::CharASCII
      | Schema::CharUnicode
      | Schema::FixedLengthStringUTF8 { .. }
      | Schema::FixedLengthStringUnicode { .. } => Ok((self, 1)),
      Schema::VariableLengthStringUTF8 { .. } | Schema::VariableLengthStringUnicode { .. } => {
        Err(VOTableError::Custom(String::from(
          "Type variable length String not expected here!",
        )))
      }
      Schema::FixedLengthArray {
        n_elems,
        elem_schema,
      } => elem_schema
        .primitive_type_and_array_len()
        .map(|(s, n)| (s, n * *n_elems)),
      Schema::VariableLengthArray { .. }
      | Schema::FixedLengthBitArray { .. }
      | Schema::VariableLengthBitArray { .. } => Err(VOTableError::Custom(String::from(
        "Type variable array not expected here!",
      ))),
    }
  }

  // For VOTable DATATABLE field deserialization
  pub fn value_from_str(&self, s: &str) -> Result<VOTableValue, VOTableError> {
    if s.is_empty() {
      Ok(VOTableValue::Null)
    } else {
      match self {
        Schema::Bool => parse_opt_bool(s).map(|opt_bool| {
          opt_bool
            .map(VOTableValue::Bool)
            .unwrap_or(VOTableValue::Null)
        }),
        Schema::Bit => match s {
          "0" => Ok(VOTableValue::Bool(false)),
          "1" => Ok(VOTableValue::Bool(true)),
          _ => Err(VOTableError::Custom(format!("Unknown bit value: '{}'", s))),
        },
        Schema::Byte { null: None } => s
          .parse()
          .map_err(VOTableError::ParseInt)
          .map(VOTableValue::Byte),
        Schema::Byte { null: Some(null) } => s.parse().map_err(VOTableError::ParseInt).map(|val| {
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Byte(val)
          }
        }),
        Schema::Short { null: None } => s
          .parse()
          .map_err(VOTableError::ParseInt)
          .map(VOTableValue::Short),
        Schema::Short { null: Some(null) } => {
          s.parse().map_err(VOTableError::ParseInt).map(|val| {
            if val == *null {
              VOTableValue::Null
            } else {
              VOTableValue::Short(val)
            }
          })
        }
        Schema::Int { null: None } => s
          .parse()
          .map_err(VOTableError::ParseInt)
          .map(VOTableValue::Int),
        Schema::Int { null: Some(null) } => s.parse().map_err(VOTableError::ParseInt).map(|val| {
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Int(val)
          }
        }),
        Schema::Long { null: None } => s
          .parse()
          .map_err(VOTableError::ParseInt)
          .map(VOTableValue::Long),
        Schema::Long { null: Some(null) } => s.parse().map_err(VOTableError::ParseInt).map(|val| {
          if val == *null {
            VOTableValue::Null
          } else {
            VOTableValue::Long(val)
          }
        }),
        Schema::Float => s
          .parse()
          .map_err(VOTableError::ParseFloat)
          .map(VOTableValue::Float),
        Schema::Double => s
          .parse()
          .map_err(VOTableError::ParseFloat)
          .map(VOTableValue::Double),
        Schema::ComplexFloat => match s.split_once(' ') {
          Some((l, r)) => l
            .parse()
            .and_then(|l| r.parse().map(|r| VOTableValue::ComplexFloat((l, r))))
            .map_err(VOTableError::ParseFloat),
          None => Err(VOTableError::Custom(format!(
            "Unable to parse complex float value: {}",
            s
          ))),
        },
        Schema::ComplexDouble => match s.split_once(' ') {
          Some((l, r)) => l
            .parse()
            .and_then(|l| r.parse().map(|r| VOTableValue::ComplexDouble((l, r))))
            .map_err(VOTableError::ParseFloat),
          None => Err(VOTableError::Custom(format!(
            "Unable to parse complex double value: {}",
            s
          ))),
        },
        Schema::CharASCII => Ok(VOTableValue::CharASCII(s.chars().next().unwrap())), // unwrap ok since we already checked for empty string
        Schema::CharUnicode => Ok(VOTableValue::CharUnicode(s.chars().next().unwrap())), // unwrap ok since we already checked for empty string
        Schema::FixedLengthStringUTF8 { n_bytes } => {
          let mut s = s.to_owned();
          let len = s.len();
          match len.cmp(n_bytes) {
            Ordering::Equal => {}
            Ordering::Less => {
              warn!("Too small string length '{}', pad with '\0'", &s);
              // Here we are safe because we add 0s, which are legal ASCII chars.
              unsafe { s.as_mut_vec() }.append(&mut vec![0_u8; *n_bytes - len]);
            }
            Ordering::Greater => {
              const fn is_utf8_char_boundary(c: u8) -> bool {
                // This is bit magic equivalent to: b < 128 || b >= 192
                (c as i8) >= -0x40
              }
              warn!("Too large string length '{}', remove elements!", &s);
              let vec_u8_view = unsafe { s.as_mut_vec() };
              vec_u8_view.truncate(*n_bytes);
              // Replace last non-UTF8-boundary bytes by '\0'.
              for byte in vec_u8_view.iter_mut().rev() {
                if is_utf8_char_boundary(*byte) {
                  break;
                } else {
                  *byte = 0_u8;
                }
              }
            }
          }
          Ok(VOTableValue::String(s))
        }
        Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => {
          Ok(VOTableValue::String(s.to_owned()))
        }
        Schema::FixedLengthStringUnicode { n_chars: _ } => {
          // TODO: convert into unicode and count the number of chars, possibly truncate or pad pad with '\0'
          Ok(VOTableValue::String(s.to_owned()))
        }
        Schema::VariableLengthStringUnicode { n_chars_max: _ } => {
          Ok(VOTableValue::String(s.to_owned()))
        }
        // Schema::Bytes => unreachable!() // only in binary mode?
        Schema::FixedLengthArray {
          n_elems,
          elem_schema,
        } => elem_schema.parse_fixed_length_array(*n_elems, s),
        Schema::VariableLengthArray {
          n_elems_max: _,
          elem_schema,
        } => elem_schema.parse_variable_length_array(s),
        Schema::FixedLengthBitArray { n_bits } => {
          let (n_actual_elems, value) = self.parse_bit_array(s)?;
          if n_actual_elems != *n_bits {
            Err(VOTableError::Custom(format!(
              "Wrong number of fixed length bits elements. Expected: {}. Actual: {}.",
              n_bits, n_actual_elems
            )))
          } else {
            Ok(value)
          }
        }
        Schema::VariableLengthBitArray { n_bits_max: _ } => self.parse_bit_array(s).map(|(_, v)| v),
      }
    }
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
      Schema::Bool
      | Schema::Byte { .. }
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
       => {}
      Schema::Bit => {}
      SVOTableValchema::CharASCII => {}
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
      Schema::Bool
      | Schema::Byte { .. }
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
      Schema::Bool => elems
        .into_iter()
        .map(parse_opt_bool)
        .collect::<Result<Vec<Option<bool>>, VOTableError>>()
        .map(VOTableValue::BooleanArray),
      Schema::Byte { .. } => elems
        .into_iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<u8>, VOTableError>>()
        .map(VOTableValue::ByteArray),
      Schema::Short { .. } => elems
        .into_iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<i16>, VOTableError>>()
        .map(VOTableValue::ShortArray),
      Schema::Int { .. } => elems
        .into_iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<i32>, VOTableError>>()
        .map(VOTableValue::IntArray),
      Schema::Long { .. } => elems
        .into_iter()
        .map(|s| s.parse().map_err(VOTableError::ParseInt))
        .collect::<Result<Vec<i64>, VOTableError>>()
        .map(VOTableValue::LongArray),
      Schema::Float => elems
        .into_iter()
        .map(|s| s.parse().map_err(VOTableError::ParseFloat))
        .collect::<Result<Vec<f32>, VOTableError>>()
        .map(VOTableValue::FloatArray),
      Schema::Double => elems
        .into_iter()
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
          Schema::FixedLengthStringUTF8 { n_bytes } => serialize_fixed_length_array(serializer, &vec![0_u8; *n_bytes]),
          Schema::FixedLengthStringUnicode { n_chars } => serialize_fixed_length_array(serializer, &vec![0_u16; *n_chars]),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, &[0_u8; 0]),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => serialize_variable_length_array(serializer, &[0_u16; 0]),
          // Schema::Bytes => serializer.serialize_bytes([0_u8; 0].as_slice())
          Schema::FixedLengthArray { n_elems, elem_schema } => {
            elem_schema.byte_len()
              .map_err(|_| S::Error::custom(format!("Sub schema contains variable length elements: {:?}", elem_schema.as_ref())))
              .and_then(|byte_len| serialize_fixed_length_array(serializer, &vec![0_u8; (*n_elems) * byte_len]))
          }
          Schema::VariableLengthArray { n_elems_max: _, elem_schema: _ } => {
            serialize_variable_length_array(serializer, &[0_u8; 0])
          }
          Schema::Bit => serializer.serialize_u8(0_u8),
          Schema::FixedLengthBitArray { n_bits } => serialize_fixed_length_array(serializer, &vec![0_u8; (7 + *n_bits) / 8]),
          Schema::VariableLengthBitArray { n_bits_max: _ } => serialize_variable_length_array(serializer, &[0_u8; 0]),
        }
      VOTableValue::Bool(v) => {
        assert!(matches!(self, Schema::Bool));
        serializer.serialize_bool(*v)
      }
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
      }
      VOTableValue::Short(v) => {
        match self {
          Schema::Short { .. } => serializer.serialize_i16(*v),
          Schema::Int { .. } => serializer.serialize_i32(*v as i32),
          Schema::Long { .. } => serializer.serialize_i64(*v as i64),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Short with schema: {:?}", self)))
        }
      }
      VOTableValue::Int(v) => {
        match self {
          Schema::Int { .. } => serializer.serialize_i32(*v),
          Schema::Long { .. } => serializer.serialize_i64(*v as i64),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Int with schema: {:?}", self)))
        }
      }
      VOTableValue::Long(v) => {
        match self {
          Schema::Long { .. } => serializer.serialize_i64(*v),
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Long with schema: {:?}", self)))
        }
      }
      VOTableValue::Float(v) => {
        match self {
          Schema::Float => serializer.serialize_f32(*v),
          Schema::Double => serializer.serialize_f64(*v as f64),
          _ => Err(S::Error::custom(format!("Value of type Float with schema: {:?}", self)))
        }
      }
      VOTableValue::Double(v) => {
        match self {
          Schema::Float => serializer.serialize_f32(*v as f32),
          Schema::Double => serializer.serialize_f64(*v),
          _ => Err(S::Error::custom(format!("Value of type Double with schema: {:?}", self)))
        }
      }
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
          Schema::FixedLengthStringUTF8 { n_bytes: _ } => serialize_fixed_length_array(serializer, v.to_string().as_bytes()),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, v.to_string().as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars: _ } => encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom).and_then(|v| serialize_fixed_length_array(serializer, &v)),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom).and_then(|v| serialize_variable_length_array(serializer, &v)),
          _ => Err(S::Error::custom(format!("Value of type CharASCII with schema: {:?}", self)))
        }
      }
      VOTableValue::CharUnicode(v) => {
        match self {
          Schema::CharASCII => serializer.serialize_u8(*v as u8),
          Schema::CharUnicode => serializer.serialize_char(*v),
          Schema::FixedLengthStringUTF8 { n_bytes: _ } => serialize_fixed_length_array(serializer, v.to_string().as_bytes()),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => serialize_variable_length_array(serializer, v.to_string().as_bytes()),
          Schema::FixedLengthStringUnicode { n_chars: _ } => encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom).and_then(|v| serialize_fixed_length_array(serializer, &v)),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => encode_ucs2(v.to_string().as_str()).map_err(S::Error::custom).and_then(|v| serialize_variable_length_array(serializer, &v)),
          _ => Err(S::Error::custom(format!("Value of type CharUnicode with schema: {:?}", self)))
        }
      }
      VOTableValue::String(s) =>
        match &self {
          Schema::FixedLengthStringUTF8 { n_bytes: _ } => FixedLengthStringUTF8(s.as_str()).serialize(serializer),
          Schema::VariableLengthStringUTF8 { n_bytes_max: _ } => VariableLengthStringUTF8(s.as_str()).serialize(serializer),
          Schema::FixedLengthStringUnicode { n_chars: _ } => FixedLengthStringUnicode(s.as_str()).serialize(serializer),
          Schema::VariableLengthStringUnicode { n_chars_max: _ } => VariableLengthStringUnicode(s.as_str()).serialize(serializer),
          _ => Err(S::Error::custom(format!("Wrong schema associated to S serialize_variable_length_array(serializer, &encode_ucs2(s.as_str()).map_err(S::Error::custom)?),tring. Actual: {:?}. Expected: \
          FixedLengthStringASCII, VariableLengthStringASCII\
          FixedLengthStringUnicode or VariableLengthStringUnicode.", &self)))
        }
      VOTableValue::BitArray(bitvec) => {
        let v: Vec<u8> = BV::clone(&bitvec.0).into_vec();
        match &self {
          Schema::FixedLengthBitArray { n_bits } => {
            let n_bytes = (7 + *n_bits) / 8;
            if n_bytes != v.len() {
              return Err(S::Error::custom(format!("Wrong number of bytes in BitArray. Actual: {}. Expected: {}.", v.len(), n_bytes)));
            }
            serialize_fixed_length_array(serializer, &v)
          }
          Schema::VariableLengthBitArray { n_bits_max: _ } => serialize_variable_length_array(serializer, &v),
          _ => Err(S::Error::custom(format!("Wrong schema associated to BitArray. Actual: {:?}. Expected: FixedLengthBitArray or VariableLengthBitArray.", &self)))
        }
      }
      VOTableValue::BooleanArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Bool { .. }) =>
            serialize_fixed_length_iter(serializer, v.iter().map(OptBool)),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Bool { .. }) =>
            serialize_variable_length_iter(serializer, v.len(), v.iter().map(OptBool)),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom)
              .and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter().map(OptBool))),
          _ => Err(S::Error::custom(format!("Wrong schema associated to BooleanArray. Actual: {:?}. Expected: FixedLengthArray(Byte) or VariableLengthArray(Byte).", &self)))
        }
      VOTableValue::ByteArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Byte { .. }) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Byte { .. }) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom)
              .and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ByteArray. Actual: {:?}. Expected: FixedLengthArray(Byte) or VariableLengthArray(Byte).", &self)))
        }
      VOTableValue::ShortArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::Short { .. })) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Short { .. }) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom)
              .and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ShortArray. Actual: {:?}. Expected: FixedLengthArray(Short) or VariableLengthArray(Short).", &self)))
        }
      VOTableValue::IntArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::Int { .. })) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Int { .. }) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom)
              .and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to IntArray. Actual: {:?}. Expected: FixedLengthArray(Int) or VariableLengthArray(Int).", &self)))
        }
      VOTableValue::LongArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::Long { .. })) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Long { .. }) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom)
              .and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to LongArray. Actual: {:?}. Expected: FixedLengthArray(Long) or VariableLengthArray(Long).", &self)))
        }
      VOTableValue::FloatArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::Float)) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Float) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom)
              .and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to FloatArray. Actual: {:?}. Expected: FixedLengthArray(Float) or VariableLengthArray(Float).", &self)))
        }
      VOTableValue::DoubleArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::Double)) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::Double) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom).and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to DoubleArray. Actual: {:?}. Expected: FixedLengthArray(Double) or VariableLengthArray(Double).", &self)))
        }
      VOTableValue::ComplexFloatArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::ComplexFloat)) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::ComplexFloat) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom).and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ComplexFloatArray. Actual: {:?}. Expected: FixedLengthArray(ComplexFloat) or VariableLengthArray(ComplexFloat).", &self)))
        }
      VOTableValue::ComplexDoubleArray(v) =>
        match &self {
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::ComplexDouble)) =>
            serialize_fixed_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::ComplexDouble) =>
            serialize_variable_length_array(serializer, v),
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthArray { .. }) =>
            elem_schema.fixed_len().map_err(S::Error::custom).and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter())),
          _ => Err(S::Error::custom(format!("Wrong schema associated to ComplexDoubleArray. Actual: {:?}. Expected: FixedLengthArray(ComplexDouble) or VariableLengthArray(ComplexDouble).", &self)))
        },
      VOTableValue::StringArray(v) => {
        match &self {
          // UTF-8
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::FixedLengthStringUTF8 { .. })) => {
            serialize_fixed_length_iter(serializer, v.iter().map(|s| FixedLengthStringUTF8(s.as_str())))
          }
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthStringUTF8 { .. }) => {
            serialize_variable_length_iter(serializer, v.len(), v.iter().map(|s| FixedLengthStringUTF8(s.as_str())))
          }
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::FixedLengthStringUTF8 { .. })) => {
            elem_schema.fixed_len().map_err(S::Error::custom).and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter().map(|s| FixedLengthStringUTF8(s.as_str()))))
          }
          // Unicode
          Schema::FixedLengthArray { n_elems: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::FixedLengthStringUnicode { .. })) => {
            serialize_fixed_length_iter(serializer, v.iter().map(|s| FixedLengthStringUnicode(s.as_str())))
          }
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.as_ref(), &Schema::FixedLengthStringUnicode { .. }) => {
            serialize_variable_length_iter(serializer, v.len(), v.iter().map(|s| FixedLengthStringUnicode(s.as_str())))
          }
          Schema::VariableLengthArray { n_elems_max: _, elem_schema } if matches!(elem_schema.primitive_schema(), Ok(&Schema::FixedLengthStringUnicode { .. })) => {
            elem_schema.fixed_len().map_err(S::Error::custom).and_then(|len| serialize_variable_length_iter(serializer, v.len() / len, v.iter().map(|s| FixedLengthStringUnicode(s.as_str()))))
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
      VOTableValue::BooleanArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::ByteArray(_) => None,    // TODO: convert array if not the right type...
      VOTableValue::ShortArray(_) => None,   // TODO: convert array if not the right type...
      VOTableValue::IntArray(_) => None,     // TODO: convert array if not the right type...
      VOTableValue::LongArray(_) => None,    // TODO: convert array if not the right type...
      VOTableValue::FloatArray(_) => None,   // TODO: convert array if not the right type...
      VOTableValue::DoubleArray(_) => None,  // TODO: convert array if not the right type...
      VOTableValue::ComplexFloatArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::ComplexDoubleArray(_) => None, // TODO: convert array if not the right type...
      VOTableValue::StringArray(_) => None,  // TODO: convert array if not the right type...
    };
    if let Some(new_val) = new_val {
      let _ = std::mem::replace(value, new_val);
    }
    Ok(())
  }
}

/// Same code for fixed length arrays of arrays
fn serialize_fixed_length_array<T, S>(serializer: S, v: &[T]) -> Result<S::Ok, S::Error>
where
  T: Serialize,
  S: Serializer,
{
  // we do nothing with the len!
  let mut seq = serializer.serialize_tuple(v.len())?;
  for elem in v {
    seq.serialize_element(elem)?;
  }
  seq.end()
}

fn serialize_fixed_length_iter<T, S, I>(serializer: S, it: I) -> Result<S::Ok, S::Error>
where
  T: Serialize,
  S: Serializer,
  I: Iterator<Item = T>,
{
  // we do nothing with the len!
  let mut seq = serializer.serialize_tuple(it.size_hint().0)?;
  for elem in it {
    seq.serialize_element(&elem)?;
  }
  seq.end()
}

/// Only for variable length array of primitives (arrays of arrays are flattened!)
fn serialize_variable_length_array<T, S>(serializer: S, v: &[T]) -> Result<S::Ok, S::Error>
where
  T: Serialize,
  S: Serializer,
{
  let mut seq = serializer.serialize_seq(Some(v.len()))?;
  for elem in v {
    seq.serialize_element(elem)?;
  }
  seq.end()
}

/// For variable length array, only the size of the variable part must be written!
/// # Param
/// * `var_len`: in case of arrays of arrays, size of the outer, variable, array;
///               else, it is the actual len of the given slice `v`
fn serialize_variable_length_iter<T, S, I>(
  serializer: S,
  var_len: usize,
  it: I,
) -> Result<S::Ok, S::Error>
where
  T: Serialize,
  S: Serializer,
  I: Iterator<Item = T>,
{
  // `var_len` is the value written in the BINARY/BINARY2 serialization
  let mut seq = serializer.serialize_seq(Some(var_len))?;
  for elem in it {
    seq.serialize_element(&elem)?;
  }
  seq.end()
}

impl From<&Field> for Schema {
  fn from(field: &Field) -> Self {
    // For all except Bits, CharASCII, CharUNICODE
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

/// Used only for BINARY and BINARY2
impl<'de> DeserializeSeed<'de> for &Schema {
  type Value = VOTableValue;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    trace!("Deserialize field of schema: {:?}", self);
    match self {
      Schema::Bool => BooleanSead.deserialize(deserializer).map(|opt_bool| {
        opt_bool
          .map(VOTableValue::Bool)
          .unwrap_or(VOTableValue::Null)
      }),
      Schema::Bit => <u8>::deserialize(deserializer).map(|b| VOTableValue::Bool(b != 0)),
      Schema::Byte { null: None } => <u8>::deserialize(deserializer).map(VOTableValue::Byte),
      Schema::Short { null: None } => <i16>::deserialize(deserializer).map(VOTableValue::Short),
      Schema::Int { null: None } => <i32>::deserialize(deserializer).map(VOTableValue::Int),
      Schema::Long { null: None } => <i64>::deserialize(deserializer).map(VOTableValue::Long),
      Schema::Byte { null: Some(null) } => <u8>::deserialize(deserializer).map(|v| {
        if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Byte(v)
        }
      }),
      Schema::Short { null: Some(null) } => <i16>::deserialize(deserializer).map(|v| {
        if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Short(v)
        }
      }),
      Schema::Int { null: Some(null) } => <i32>::deserialize(deserializer).map(|v| {
        if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Int(v)
        }
      }),
      Schema::Long { null: Some(null) } => <i64>::deserialize(deserializer).map(|v| {
        if v == *null {
          VOTableValue::Null
        } else {
          VOTableValue::Long(v)
        }
      }),
      Schema::Float => <f32>::deserialize(deserializer).map(|v| {
        if v.is_finite() {
          VOTableValue::Float(v)
        } else {
          VOTableValue::Null
        }
      }),
      Schema::Double => <f64>::deserialize(deserializer).map(|v| {
        if v.is_finite() {
          VOTableValue::Double(v)
        } else {
          VOTableValue::Null
        }
      }),
      Schema::ComplexFloat => <(f32, f32)>::deserialize(deserializer).map(|(real, img)| {
        if real.is_finite() && img.is_finite() {
          VOTableValue::ComplexFloat((real, img))
        } else {
          VOTableValue::Null
        }
      }),
      Schema::ComplexDouble => <(f64, f64)>::deserialize(deserializer).map(|(real, img)| {
        if real.is_finite() && img.is_finite() {
          VOTableValue::ComplexDouble((real, img))
        } else {
          VOTableValue::Null
        }
      }),
      Schema::CharASCII => deserializer
        .deserialize_u8(CharVisitor)
        .map(VOTableValue::CharASCII),
      Schema::CharUnicode => deserializer
        .deserialize_u16(CharVisitor)
        .map(VOTableValue::CharUnicode),
      Schema::FixedLengthStringUTF8 { n_bytes } => FixedLengthUTF8StringSeed::new(*n_bytes)
        .deserialize(deserializer)
        .map(VOTableValue::String),
      Schema::FixedLengthStringUnicode { n_chars } => FixedLengthUnicodeStringSeed::new(*n_chars)
        .deserialize(deserializer)
        .map(VOTableValue::String),
      Schema::VariableLengthStringUTF8 {
        n_bytes_max: n_chars_max,
      } => VarLengthUTF8StringSeed::new(*n_chars_max)
        .deserialize(deserializer)
        .map(VOTableValue::String),
      Schema::VariableLengthStringUnicode { n_chars_max } => {
        VarLengthUnicodeStringSeed::new(*n_chars_max)
          .deserialize(deserializer)
          .map(VOTableValue::String)
      }
      // Schema::Bytes => deserializer.deserialize_bytes(BytesVisitor).map(VOTableValue::Bytes),
      Schema::FixedLengthArray {
        n_elems,
        elem_schema,
      } => match elem_schema.as_ref() {
        Schema::Bool => new_fixed_length_array_of_boolean_seed(*n_elems)
          .deserialize(deserializer)
          .map(VOTableValue::BooleanArray),
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
          FixedLengthArrayOfUnidecodeStringSeed::new(*n_elems, *n_chars)
            .deserialize(deserializer)
            .map(VOTableValue::StringArray)
        }
        Schema::FixedLengthArray {
          n_elems: n_sub_elems,
          elem_schema: sub_elems_schema,
        } => {
          // We convert arrays of arrays in flat arrays.
          // Form a performance perspective, we should opt for a solution avoiding the 'clone'.
          let virtual_schema = Schema::FixedLengthArray {
            n_elems: *n_elems * *n_sub_elems,
            elem_schema: sub_elems_schema.clone(),
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
        Schema::Bool => new_var_length_array_of_boolean_seed(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::BooleanArray),
        Schema::Byte { .. } => VarLengthArrayPhantomSeed::<u8>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::ByteArray),
        Schema::Short { .. } => VarLengthArrayPhantomSeed::<i16>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::ShortArray),
        Schema::Int { .. } => VarLengthArrayPhantomSeed::<i32>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::IntArray),
        Schema::Long { .. } => VarLengthArrayPhantomSeed::<i64>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::LongArray),
        Schema::Float => VarLengthArrayPhantomSeed::<f32>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::FloatArray),
        Schema::Double => VarLengthArrayPhantomSeed::<f64>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::DoubleArray),
        Schema::ComplexFloat => VarLengthArrayPhantomSeed::<(f32, f32)>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::ComplexFloatArray),
        Schema::ComplexDouble => VarLengthArrayPhantomSeed::<(f64, f64)>::new(*n_elems_max)
          .deserialize(deserializer)
          .map(VOTableValue::ComplexDoubleArray),
        Schema::FixedLengthStringUTF8 { n_bytes } => {
          VarLengthArrayOfUTF8StringSeed::new(*n_elems_max, *n_bytes)
            .deserialize(deserializer)
            .map(VOTableValue::StringArray)
        }
        Schema::FixedLengthStringUnicode { n_chars } => {
          VarLengthArrayOfUnicodeStringSeed::new(*n_elems_max, *n_chars)
            .deserialize(deserializer)
            .map(VOTableValue::StringArray)
        }
        Schema::FixedLengthArray {
          n_elems: sub_n_elems,
          elem_schema: sub_elem_schema,
        } => {
          let (elem_schema, mut sub_array_len) = sub_elem_schema
            .as_ref()
            .primitive_type_and_array_len()
            .map_err(D::Error::custom)?;
          sub_array_len *= *sub_n_elems;
          match elem_schema {
            Schema::Bool => {
              new_var_length_of_fixed_len_array_of_boolean_seed(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::BooleanArray)
            }
            Schema::Bit => todo!(),
            Schema::Byte { .. } => {
              VarLengthVectorOfVectorSeed::<u8>::new(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::ByteArray)
            }
            Schema::Short { .. } => {
              VarLengthVectorOfVectorSeed::<i16>::new(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::ShortArray)
            }
            Schema::Int { .. } => {
              VarLengthVectorOfVectorSeed::<i32>::new(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::IntArray)
            }
            Schema::Long { .. } => {
              VarLengthVectorOfVectorSeed::<i64>::new(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::LongArray)
            }
            Schema::Float => VarLengthVectorOfVectorSeed::<f32>::new(*n_elems_max, sub_array_len)
              .deserialize(deserializer)
              .map(VOTableValue::FloatArray),
            Schema::Double => VarLengthVectorOfVectorSeed::<f64>::new(*n_elems_max, sub_array_len)
              .deserialize(deserializer)
              .map(VOTableValue::DoubleArray),
            Schema::ComplexFloat => {
              VarLengthVectorOfVectorSeed::<(f32, f32)>::new(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::ComplexFloatArray)
            }
            Schema::ComplexDouble => {
              VarLengthVectorOfVectorSeed::<(f64, f64)>::new(*n_elems_max, sub_array_len)
                .deserialize(deserializer)
                .map(VOTableValue::ComplexDoubleArray)
            }
            Schema::CharASCII | Schema::CharUnicode => {
              unreachable!("More than one char is supposed to be considered as a String")
            }
            Schema::FixedLengthStringUTF8 { n_bytes } => {
              VarLengthVectorOfVectorSeedWithSeed::new_for_utf8_string(
                *n_elems_max,
                sub_array_len,
                *n_bytes,
              )
              .deserialize(deserializer)
              .map(VOTableValue::StringArray)
            }
            Schema::FixedLengthStringUnicode { n_chars } => {
              VarLengthVectorOfVectorSeedWithSeed::new_for_unicode_string(
                *n_elems_max,
                sub_array_len,
                *n_chars,
              )
              .deserialize(deserializer)
              .map(VOTableValue::StringArray)
            }
            Schema::FixedLengthArray { .. } => {
              unreachable!("FixedLengthArray not supposed to be a schema leaf.")
            }
            Schema::VariableLengthStringUTF8 { .. }
            | Schema::VariableLengthStringUnicode { .. }
            | Schema::VariableLengthArray { .. }
            | Schema::FixedLengthBitArray { .. }
            | Schema::VariableLengthBitArray { .. } => {
              unreachable!("Type returning an error  in `primitive_type_and_array_len`")
            }
          }
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

pub fn parse_opt_bool(s: &str) -> Result<Option<bool>, VOTableError> {
  match s {
    "0" | "f" | "F" => Ok(Some(false)),
    "1" | "t" | "T" => Ok(Some(true)),
    "?" | " " | "" => Ok(None),
    _ =>
      s.to_lowercase().parse::<bool>()
        .map(Some)
        .map_err(|_| VOTableError::Custom(
          format!("Unable to parse boolean value. Expected: '0', '1', 't', 'f', 'T', 'F', '?', ' ', '' or 'true', 'false'. Actual: '{}'", s))
        ),
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
