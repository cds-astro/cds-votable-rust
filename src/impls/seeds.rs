//! This module contains deserialization seeds.

use std::{marker::PhantomData, string::String};

use crate::impls::visitors::VarVectorOfFixedVectorAppenderVisitor;
use serde::{
  de::{Deserialize, DeserializeOwned, DeserializeSeed, Error},
  Deserializer,
};

use super::{
  decode_ucs2,
  visitors::{
    FixedLengthArrayVisitorWithSeed, FixedLengthVectorAppenderVisitor,
    FixedLengthVectorAppenderVisitorWithSeed, VarVectorOfFixedVectorAppenderVisitorWithSeed,
    VariableLengthArrayVisitor, VariableLengthArrayVisitorWithSeed,
  },
};

// SEEDS FOR FIXED LENGTH ELEMENTS

/// Structure implementing 'deserialize' for fixed length arrays of primitive types.
#[derive(Clone)]
pub struct FixedLengthArrayPhantomSeed<T: DeserializeOwned> {
  len: usize,
  _phantom: PhantomData<T>,
}
impl<T: DeserializeOwned> FixedLengthArrayPhantomSeed<T> {
  pub fn new(len: usize) -> Self {
    Self {
      len,
      _phantom: PhantomData,
    }
  }
}
impl<'de, T: DeserializeOwned + 'de> DeserializeSeed<'de> for FixedLengthArrayPhantomSeed<T> {
  type Value = Vec<T>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    // deserializer.deserialize_tuple(self.len, FixedLengthArrayVisitor::<T>::new(self.len))
    deserializer.deserialize_tuple(
      self.len,
      FixedLengthArrayVisitorWithSeed::<PhantomData<T>>::new_default(self.len),
    )
  }
}

/// Structure implementing 'deserialize' for fixed length arrays of any type of elements, provided
/// the seed to deserialization each element.
/// This can be used for fixed length arrays of fixed length arrays.
#[derive(Clone)]
pub struct FixedLengthArraySeed<T: Clone> {
  len: usize,
  seed: T,
}
impl<T: Clone> FixedLengthArraySeed<T> {
  pub fn with_seed(len: usize, seed: T) -> Self {
    Self { len, seed }
  }
}
impl<T: DeserializeOwned + Clone> FixedLengthArraySeed<PhantomData<T>> {
  pub fn new(len: usize) -> Self {
    Self::with_seed(len, PhantomData)
  }
}
impl<'de, T: 'de + DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for FixedLengthArraySeed<T> {
  type Value = Vec<T::Value>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    // deserializer.deserialize_tuple(self.len, FixedLengthArrayVisitor::<T>::new(self.len))
    deserializer.deserialize_tuple(
      self.len,
      FixedLengthArrayVisitorWithSeed::new(self.len, self.seed),
    )
  }
}

/// Structure implementing 'deserialize' for fixed length UTF-8 Strings.
#[derive(Clone)]
pub struct FixedLengthUTF8StringSeed {
  n_bytes: usize,
}
impl FixedLengthUTF8StringSeed {
  pub fn new(n_bytes: usize) -> Self {
    Self { n_bytes }
  }
}
impl<'de> DeserializeSeed<'de> for FixedLengthUTF8StringSeed {
  type Value = String;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    FixedLengthArrayPhantomSeed::<u8>::new(self.n_bytes)
      .deserialize(deserializer)
      .and_then(|bytes| {
        // If starts with ascii char NULL, the string is empty
        if bytes.starts_with(&[0]) {
          Ok(String::new())
        } else {
          String::from_utf8(bytes).map_err(Error::custom)
        }
      })
  }
}

/// Structure implementing 'deserialize' for fixed length arrays of UTF-8 Strings.
#[derive(Clone)]
pub struct FixedLengthArrayOfUTF8StringSeed {
  array_size: usize,
  seed: FixedLengthUTF8StringSeed,
}
impl FixedLengthArrayOfUTF8StringSeed {
  pub fn new(array_size: usize, n_bytes_per_string: usize) -> Self {
    Self {
      array_size,
      seed: FixedLengthUTF8StringSeed::new(n_bytes_per_string),
    }
  }
}
impl<'de> DeserializeSeed<'de> for FixedLengthArrayOfUTF8StringSeed {
  type Value = Vec<String>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    FixedLengthArraySeed::with_seed(self.array_size, self.seed).deserialize(deserializer)
  }
}

/// Structure implementing 'deserialize' for fixed length Unicode Strings.
#[derive(Clone)]
pub struct FixedLengthUnicodeStringSeed {
  n_chars: usize,
}
impl FixedLengthUnicodeStringSeed {
  pub fn new(n_chars: usize) -> Self {
    Self { n_chars }
  }
}
impl<'de> DeserializeSeed<'de> for FixedLengthUnicodeStringSeed {
  type Value = String;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    FixedLengthArrayPhantomSeed::<u16>::new(self.n_chars)
      .deserialize(deserializer)
      .and_then(|chars| {
        // If starts with ascii char NULL, the string is empty
        if chars.starts_with(&[0]) {
          Ok(String::new())
        } else {
          decode_ucs2(chars).map_err(Error::custom)
        }
      })
  }
}

/// Structure implementing 'deserialize' for fixed length arrays of Unicode Strings.
#[derive(Clone)]
pub struct FixedLengthArrayOfUnidecodeStringSeed {
  array_size: usize,
  seed: FixedLengthUnicodeStringSeed,
}
impl FixedLengthArrayOfUnidecodeStringSeed {
  pub fn new(array_size: usize, n_chars_per_string: usize) -> Self {
    Self {
      array_size,
      seed: FixedLengthUnicodeStringSeed::new(n_chars_per_string),
    }
  }
}
impl<'de> DeserializeSeed<'de> for FixedLengthArrayOfUnidecodeStringSeed {
  type Value = Vec<String>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    FixedLengthArraySeed::with_seed(self.array_size, self.seed).deserialize(deserializer)
  }
}

// SEEDS FOR VARIABLE LENGTH ELEMENTS

/// Structure implementing 'deserialize' for variable length arrays of primitive types.
/// # Info
/// * `max_len` is only used to pre-allocate vector size.
#[derive(Clone)]
pub struct VarLengthArrayPhantomSeed<T: DeserializeOwned> {
  max_len: Option<usize>,
  _phantom: PhantomData<T>,
}
impl<T: DeserializeOwned> VarLengthArrayPhantomSeed<T> {
  pub fn new(max_len: Option<usize>) -> Self {
    Self {
      max_len,
      _phantom: PhantomData,
    }
  }
}
impl<'de, T: DeserializeOwned + 'de> DeserializeSeed<'de> for VarLengthArrayPhantomSeed<T> {
  type Value = Vec<T>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(VariableLengthArrayVisitor::<T>::new(self.max_len))
  }
}

/// Structure implementing 'deserialize' for var length arrays of any type of elements, provided
/// the seed to deserialization each element.
/// This can be used for var length arrays of fixed length arrays.
#[derive(Clone)]
pub struct VarLengthArraySeed<T: Clone> {
  max_len: Option<usize>,
  seed: T,
}
impl<T: Clone> VarLengthArraySeed<T> {
  pub fn with_seed(max_len: Option<usize>, seed: T) -> Self {
    Self { max_len, seed }
  }
}
impl<T: DeserializeOwned + Clone> VarLengthArraySeed<PhantomData<T>> {
  pub fn new(max_len: Option<usize>) -> Self {
    Self::with_seed(max_len, PhantomData)
  }
}
impl<'de, T: 'de + DeserializeSeed<'de> + Clone> DeserializeSeed<'de> for VarLengthArraySeed<T> {
  type Value = Vec<T::Value>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(VariableLengthArrayVisitorWithSeed::new(
      self.max_len,
      self.seed,
    ))
  }
}

/// Structure implementing 'deserialize' for var length UTF-8 Strings.
#[derive(Clone)]
pub struct VarLengthUTF8StringSeed {
  n_bytes_max: Option<usize>,
}
impl VarLengthUTF8StringSeed {
  pub fn new(n_bytes_max: Option<usize>) -> Self {
    Self { n_bytes_max }
  }
}
impl<'de> DeserializeSeed<'de> for VarLengthUTF8StringSeed {
  type Value = String;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    VarLengthArrayPhantomSeed::<u8>::new(self.n_bytes_max)
      .deserialize(deserializer)
      .and_then(|bytes| String::from_utf8(bytes).map_err(Error::custom))
  }
}

/// Structure implementing 'deserialize' for var length arrays of UTF-8 Strings.
#[derive(Clone)]
pub struct VarLengthArrayOfUTF8StringSeed {
  array_max_len: Option<usize>,
  seed: FixedLengthUTF8StringSeed,
}
impl VarLengthArrayOfUTF8StringSeed {
  pub fn new(array_max_len: Option<usize>, n_bytes_per_string: usize) -> Self {
    Self {
      array_max_len,
      seed: FixedLengthUTF8StringSeed::new(n_bytes_per_string),
    }
  }
}
impl<'de> DeserializeSeed<'de> for VarLengthArrayOfUTF8StringSeed {
  type Value = Vec<String>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    VarLengthArraySeed::with_seed(self.array_max_len, self.seed).deserialize(deserializer)
  }
}

/// Structure implementing 'deserialize' for variable length Unicode Strings.
#[derive(Clone)]
pub struct VarLengthUnicodeStringSeed {
  n_chars_max: Option<usize>,
}
impl VarLengthUnicodeStringSeed {
  pub fn new(n_chars_max: Option<usize>) -> Self {
    Self { n_chars_max }
  }
}
impl<'de> DeserializeSeed<'de> for VarLengthUnicodeStringSeed {
  type Value = String;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    VarLengthArrayPhantomSeed::<u16>::new(self.n_chars_max)
      .deserialize(deserializer)
      .and_then(|chars| decode_ucs2(chars).map_err(Error::custom))
  }
}

/// Structure implementing 'deserialize' for variable length arrays of Unicode Strings.
#[derive(Clone)]
pub struct VarLengthArrayOfUnicodeStringSeed {
  array_max_len: Option<usize>,
  seed: FixedLengthUnicodeStringSeed,
}
impl VarLengthArrayOfUnicodeStringSeed {
  pub fn new(array_max_len: Option<usize>, n_chars_per_string: usize) -> Self {
    Self {
      array_max_len,
      seed: FixedLengthUnicodeStringSeed::new(n_chars_per_string),
    }
  }
}
impl<'de> DeserializeSeed<'de> for VarLengthArrayOfUnicodeStringSeed {
  type Value = Vec<String>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    VarLengthArraySeed::with_seed(self.array_max_len, self.seed).deserialize(deserializer)
  }
}

/// Appender for variable array of fixed length arrays
pub struct FixedLengthVectorAppenderSeed<'a, T: 'a> {
  len: usize,
  v: &'a mut Vec<T>,
}
impl<'a, T: 'a> FixedLengthVectorAppenderSeed<'a, T> {
  pub fn new(len: usize, v: &'a mut Vec<T>) -> Self {
    Self { len, v }
  }
}
impl<'de, 'a, T> DeserializeSeed<'de> for FixedLengthVectorAppenderSeed<'a, T>
where
  T: Deserialize<'de>,
{
  type Value = ();

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_tuple(
      self.len,
      FixedLengthVectorAppenderVisitor::new(self.len, self.v),
    )
  }
}

pub struct VarLengthVectorAppenderSeed<'a, T: 'a> {
  /// Size of the fixed length array the variable array contains.
  len: usize,
  /// Vector in which all results are concatenated
  v: &'a mut Vec<T>,
}
impl<'a, T: 'a> VarLengthVectorAppenderSeed<'a, T> {
  pub fn new(len: usize, v: &'a mut Vec<T>) -> Self {
    Self { len, v }
  }
}
impl<'de, 'a, T> DeserializeSeed<'de> for VarLengthVectorAppenderSeed<'a, T>
where
  T: Deserialize<'de>,
{
  type Value = ();

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(VarVectorOfFixedVectorAppenderVisitor::new(self.len, self.v))
  }
}

pub struct VarLengthVectorOfVectorSeed<T> {
  /// upper limit on the number of fixed length array the variable array contains.
  var_max_len: Option<usize>,
  /// Size of the fixed length array the variable array contains.
  len: usize,
  /// Vector in which all results are concatenated
  _phantom: PhantomData<T>,
}
impl<T> VarLengthVectorOfVectorSeed<T> {
  pub fn new(var_max_len: Option<usize>, len: usize) -> Self {
    Self {
      var_max_len,
      len,
      _phantom: PhantomData,
    }
  }
}
impl<'de, T> DeserializeSeed<'de> for VarLengthVectorOfVectorSeed<T>
where
  T: Deserialize<'de>,
{
  type Value = Vec<T>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    let mut v = Vec::<T>::with_capacity(self.var_max_len.unwrap_or(16) * self.len);
    deserializer
      .deserialize_seq(VarVectorOfFixedVectorAppenderVisitor::new(self.len, &mut v))
      .map(|()| v)
  }
}

// With Seed

/// Appender for variable array of fixed length arrays of elements, provided a seed
pub struct FixedLengthVectorAppenderSeedWithSeed<'a, T: 'a, S> {
  len: usize,
  v: &'a mut Vec<T>,
  seed: S,
}
impl<'a, T: 'a, S> FixedLengthVectorAppenderSeedWithSeed<'a, T, S> {
  pub fn new(len: usize, v: &'a mut Vec<T>, seed: S) -> Self {
    Self { len, v, seed }
  }
}
impl<'de, 'a, T, S> DeserializeSeed<'de> for FixedLengthVectorAppenderSeedWithSeed<'a, T, S>
where
  T: Deserialize<'de>,
  S: DeserializeSeed<'de, Value = T> + Clone,
{
  type Value = ();

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_tuple(
      self.len,
      FixedLengthVectorAppenderVisitorWithSeed::new(self.len, self.v, self.seed),
    )
  }
}

pub struct VarLengthVectorAppenderSeedWithSeed<'a, T: 'a, S> {
  len: usize,
  v: &'a mut Vec<T>,
  seed: S,
}
impl<'a, T: 'a, S> VarLengthVectorAppenderSeedWithSeed<'a, T, S> {
  pub fn new(len: usize, v: &'a mut Vec<T>, seed: S) -> Self {
    Self { len, v, seed }
  }
}
impl<'de, 'a, T, S> DeserializeSeed<'de> for VarLengthVectorAppenderSeedWithSeed<'a, T, S>
where
  T: Deserialize<'de>,
  S: DeserializeSeed<'de, Value = T> + Clone,
{
  type Value = ();

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_seq(VarVectorOfFixedVectorAppenderVisitorWithSeed::new(
      self.len, self.v, self.seed,
    ))
  }
}

pub struct VarLengthVectorOfVectorOfStringSeed<S> {
  /// upper limit on the number of fixed length array the variable array contains.
  var_max_len: Option<usize>,
  /// Size of the fixed length array the variable array contains.
  len: usize,
  /// Vector in which all results are concatenated
  seed: S,
}
impl<S> VarLengthVectorOfVectorOfStringSeed<S> {
  pub fn new(var_max_len: Option<usize>, len: usize, seed: S) -> Self {
    Self {
      var_max_len,
      len,
      seed,
    }
  }
}
impl VarLengthVectorOfVectorOfStringSeed<FixedLengthUTF8StringSeed> {
  pub fn new_for_utf8_string(var_max_len: Option<usize>, len: usize, str_len: usize) -> Self {
    Self::new(var_max_len, len, FixedLengthUTF8StringSeed::new(str_len))
  }
}
impl VarLengthVectorOfVectorOfStringSeed<FixedLengthUnicodeStringSeed> {
  pub fn new_for_unicode_string(var_max_len: Option<usize>, len: usize, str_len: usize) -> Self {
    Self::new(var_max_len, len, FixedLengthUnicodeStringSeed::new(str_len))
  }
}
impl<'de, S> DeserializeSeed<'de> for VarLengthVectorOfVectorOfStringSeed<S>
where
  S: DeserializeSeed<'de> + Clone,
  S::Value: Deserialize<'de>,
{
  type Value = Vec<S::Value>;

  fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
  where
    D: Deserializer<'de>,
  {
    let mut v = Vec::<S::Value>::with_capacity(self.var_max_len.unwrap_or(16) * self.len);
    deserializer
      .deserialize_seq(VarVectorOfFixedVectorAppenderVisitorWithSeed::new(
        self.len, &mut v, self.seed,
      ))
      .map(|()| v)
  }
}
