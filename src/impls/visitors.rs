use std::{
  fmt::{self, Formatter},
  marker::PhantomData,
  str::from_utf8,
  string::String,
};

use serde::{
  de::{DeserializeOwned, DeserializeSeed, Error, SeqAccess, Unexpected, Visitor},
  Deserialize,
};

use super::seeds::{FixedLengthVectorAppenderSeed, FixedLengthVectorAppenderSeedWithSeed};
use crate::error::VOTableError;

/// Structure made to visit a primitive or an optional primitive.
/// Attempts to visit a primitive different from the one it as been made for will fail.
pub struct VisitorPrim<E> {
  _marker: PhantomData<E>,
}

/// Visit the possible binary (u8) values of a boolean
pub struct OptBoolVisitor;

impl<'de> Visitor<'de> for OptBoolVisitor {
  type Value = Option<bool>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a Option<bool>")
  }

  fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
  where
    E: Error,
  {
    match v {
      b'0' | b'f' | b'F' => Ok(Some(false)),
      b'1' | b't' | b'T' => Ok(Some(true)),
      b'?' | b' ' | b'\0' => Ok(None),
      _ => Err(Error::custom(format!(
        "Wrong boolean code. Expected: '0', '1', 'f', 't', 'F', 'T', '?', ' ', '\0' . Actual: char={}, u8={}",
        v as char, v
      ))),
    }
  }
}

pub struct StringVisitor;

// Copy/paste from serde::de::impls.rs
impl<'de> Visitor<'de> for StringVisitor {
  type Value = String;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a string")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(v.to_owned())
  }

  fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(v)
  }

  fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
  where
    E: Error,
  {
    match from_utf8(v) {
      Ok(s) => Ok(s.to_owned()),
      Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
    }
  }

  fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
  where
    E: Error,
  {
    match String::from_utf8(v) {
      Ok(s) => Ok(s),
      Err(e) => Err(Error::invalid_value(
        Unexpected::Bytes(&e.into_bytes()),
        &self,
      )),
    }
  }
}

pub struct CharVisitor;

impl<'de> Visitor<'de> for CharVisitor {
  type Value = char;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a character")
  }

  #[inline]
  fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(v as char)
  }

  #[inline]
  fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
  where
    E: Error,
  {
    // In VOTable, unicode chars are encoded in UCS-2,
    // see: https://stackoverflow.com/questions/36236364/why-java-char-uses-utf-16
    let mut buf = vec![0_u8; 3];
    let n_bytes = ucs2::decode(&[v], &mut buf)
      .map_err(VOTableError::FromUCS2)
      .map_err(Error::custom)?;
    let s = String::from_utf8_lossy(&buf[..n_bytes]);
    let mut iter = s.chars();
    match (iter.next(), iter.next()) {
      (Some(c), None) => Ok(c),
      _ => Err(Error::custom(
        "Error transforming Unicode UCS-2 to UTF-16 char",
      )),
    }
  }

  #[inline]
  fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(v)
  }

  #[inline]
  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    let mut iter = v.chars();
    match (iter.next(), iter.next()) {
      (Some(c), None) => Ok(c),
      _ => Err(Error::invalid_value(Unexpected::Str(v), &self)),
    }
  }
}

pub struct BytesVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
  type Value = Vec<u8>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a borrowed byte array")
  }

  fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(v.as_bytes().to_vec())
  }

  fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
  where
    E: Error,
  {
    Ok(v.to_vec())
  }
}

pub struct FixedLengthArrayVisitor<'de, T: Deserialize<'de>> {
  len: usize,
  _marker: &'de PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> FixedLengthArrayVisitor<'de, T> {
  pub fn new(len: usize) -> Self {
    Self {
      len,
      _marker: &PhantomData,
    }
  }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for FixedLengthArrayVisitor<'de, T> {
  type Value = Vec<T>;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    let mut v: Vec<T> = Vec::with_capacity(self.len);
    for _ in 0..self.len {
      v.push(
        seq
          .next_element()?
          .ok_or_else(|| Error::custom(String::from("Premature end of stream")))?,
      );
    }
    Ok(v)
  }
}

pub struct FixedLengthArrayVisitorWithSeed<T: Clone> {
  len: usize,
  seed: T,
}

impl<U: DeserializeOwned> FixedLengthArrayVisitorWithSeed<PhantomData<U>> {
  pub fn new_default(len: usize) -> Self {
    Self::new(len, PhantomData)
  }
}

impl<T: Clone> FixedLengthArrayVisitorWithSeed<T> {
  pub fn new(len: usize, seed: T) -> Self {
    Self { len, seed }
  }
}

impl<'de, T: DeserializeSeed<'de> + Clone> Visitor<'de> for FixedLengthArrayVisitorWithSeed<T> {
  type Value = Vec<T::Value>;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    (0..self.len)
      .map(|_| {
        seq
          .next_element_seed(self.seed.clone())
          .and_then(|opt| opt.ok_or_else(|| Error::custom(String::from("Premature end of stream"))))
      })
      .collect()
  }
}

pub struct VariableLengthArrayVisitor<'de, T: Deserialize<'de>> {
  /// Maximum size of the array, if knowned.
  max_len: Option<usize>,
  _marker: &'de PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Default for VariableLengthArrayVisitor<'de, T> {
  fn default() -> Self {
    Self::new(None)
  }
}

impl<'de, T: Deserialize<'de>> VariableLengthArrayVisitor<'de, T> {
  pub fn new(max_len: Option<usize>) -> Self {
    Self {
      max_len,
      _marker: &PhantomData,
    }
  }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for VariableLengthArrayVisitor<'de, T> {
  type Value = Vec<T>;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    let mut v: Vec<T> = Vec::with_capacity(
      self
        .max_len
        .unwrap_or_else(|| seq.size_hint().unwrap_or(16)),
    );
    while let Some(value) = seq.next_element()? {
      v.push(value);
    }
    Ok(v)
  }
}

pub struct VariableLengthArrayVisitorWithSeed<T: Clone> {
  /// Maximum size of the array, if knowned.
  max_len: Option<usize>,
  seed: T,
}

impl<U: DeserializeOwned> VariableLengthArrayVisitorWithSeed<PhantomData<U>> {
  pub fn new_default(max_len: Option<usize>) -> Self {
    Self::new(max_len, PhantomData)
  }
}

impl<T: Clone> VariableLengthArrayVisitorWithSeed<T> {
  pub fn new(max_len: Option<usize>, seed: T) -> Self {
    Self { max_len, seed }
  }
}

impl<'de, T: DeserializeSeed<'de> + Clone> Visitor<'de> for VariableLengthArrayVisitorWithSeed<T> {
  type Value = Vec<T::Value>;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    let mut v: Vec<T::Value> = Vec::with_capacity(
      self
        .max_len
        .unwrap_or_else(|| seq.size_hint().unwrap_or(16)),
    );
    while let Some(value) = seq.next_element_seed(self.seed.clone())? {
      v.push(value);
    }
    Ok(v)
  }
}

// VAR ARRAYS OF FIXED ARRAYS FOR PRIMITIVES

/// Appender for fixed length arrays
pub struct FixedLengthVectorAppenderVisitor<'a, T: 'a> {
  len: usize,
  v: &'a mut Vec<T>,
}
impl<'a, T: 'a> FixedLengthVectorAppenderVisitor<'a, T> {
  pub fn new(len: usize, v: &'a mut Vec<T>) -> Self {
    Self { len, v }
  }
}
impl<'de, 'a, T> Visitor<'de> for FixedLengthVectorAppenderVisitor<'a, T>
where
  T: Deserialize<'de>,
{
  type Value = ();
  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }
  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    for _ in 0..self.len {
      let e = seq.next_element().and_then(|opt| {
        opt.ok_or_else(|| Error::custom(String::from("Premature end of stream")))
      })?;
      self.v.push(e);
    }
    Ok(())
  }
}

/// Appender for variable array of fixed length arrays
pub struct VarVectorOfFixedVectorAppenderVisitor<'a, T: 'a> {
  len: usize,
  v: &'a mut Vec<T>,
}
impl<'a, T: 'a> VarVectorOfFixedVectorAppenderVisitor<'a, T> {
  pub fn new(len: usize, v: &'a mut Vec<T>) -> Self {
    Self { len, v }
  }
}
impl<'de, 'a, T> Visitor<'de> for VarVectorOfFixedVectorAppenderVisitor<'a, T>
where
  T: Deserialize<'de>,
{
  type Value = ();

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    while let Some(()) =
      seq.next_element_seed(FixedLengthVectorAppenderSeed::new(self.len, self.v))?
    {}
    Ok(())
  }
}

// VAR ARRAYS OF FIXED ARRAYS FOR TYPES WITH SEEDS

/// Appender for fixed length arrays
pub struct FixedLengthVectorAppenderVisitorWithSeed<'a, T, S>
where
  T: 'a,
  S: Clone,
{
  len: usize,
  v: &'a mut Vec<T>,
  seed: S,
}

impl<'a, T, S> FixedLengthVectorAppenderVisitorWithSeed<'a, T, S>
where
  T: 'a,
  S: Clone,
{
  pub fn new(len: usize, v: &'a mut Vec<T>, seed: S) -> Self {
    Self { len, v, seed }
  }
}
impl<'de, 'a, T, S> Visitor<'de> for FixedLengthVectorAppenderVisitorWithSeed<'a, T, S>
where
  T: Deserialize<'de>,
  S: DeserializeSeed<'de, Value = T> + Clone,
{
  type Value = ();
  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }
  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    for _ in 0..self.len {
      let e = seq.next_element_seed(self.seed.clone()).and_then(|opt| {
        opt.ok_or_else(|| Error::custom(String::from("Premature end of stream")))
      })?;
      self.v.push(e);
    }
    Ok(())
  }
}

/// Appender for variable array of fixed length arrays
pub struct VarVectorOfFixedVectorAppenderVisitorWithSeed<'a, T, S>
where
  T: 'a,
  S: Clone,
{
  len: usize,
  v: &'a mut Vec<T>,
  seed: S,
}
impl<'a, T, S> VarVectorOfFixedVectorAppenderVisitorWithSeed<'a, T, S>
where
  T: 'a,
  S: Clone,
{
  pub fn new(len: usize, v: &'a mut Vec<T>, seed: S) -> Self {
    Self { len, v, seed }
  }
}
impl<'de, 'a, T, S> Visitor<'de> for VarVectorOfFixedVectorAppenderVisitorWithSeed<'a, T, S>
where
  T: Deserialize<'de>,
  S: DeserializeSeed<'de, Value = T> + Clone,
{
  type Value = ();

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: SeqAccess<'de>,
  {
    while let Some(()) = seq.next_element_seed(FixedLengthVectorAppenderSeedWithSeed::new(
      self.len,
      self.v,
      self.seed.clone(),
    ))? {}
    Ok(())
  }
}
