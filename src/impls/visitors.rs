use std::{
  fmt::{self, Formatter},
  marker::PhantomData,
  str::from_utf8,
  string::String,
};

use serde::{
  de::{DeserializeOwned, DeserializeSeed, Error, SeqAccess, Unexpected, Visitor},
  Deserialize, Deserializer, Serialize,
};

use crate::{error::VOTableError, impls::decode_ucs2};

/// Structure made to visit a primitive or an optional primitive.
/// Attempts to visit a primitive different from the one it as been made for will fail.
pub struct VisitorPrim<E> {
  _marker: PhantomData<E>,
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
      .into_iter()
      .map(|_| {
        seq
          .next_element_seed(self.seed.clone())
          .and_then(|opt| opt.ok_or_else(|| Error::custom(String::from("Premature end of stream"))))
      })
      .collect()
  }
}

pub struct VariableLengthArrayVisitor<'de, T: Deserialize<'de>> {
  upper_n_elems: Option<usize>,
  _marker: &'de PhantomData<T>,
}

impl<'de, T: Deserialize<'de>> Default for VariableLengthArrayVisitor<'de, T> {
  fn default() -> Self {
    Self::new(None)
  }
}

impl<'de, T: Deserialize<'de>> VariableLengthArrayVisitor<'de, T> {
  pub fn new(upper_n_elems: Option<usize>) -> Self {
    Self {
      upper_n_elems,
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
        .upper_n_elems
        .unwrap_or_else(|| seq.size_hint().unwrap_or(16)),
    );
    while let Some(value) = seq.next_element()? {
      v.push(value);
    }
    Ok(v)
  }
}

#[derive(Clone)]
/// An object implementing 'deserialize' for fixed lengh array of primitive types.
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

#[derive(Clone)]
pub struct FixedLengthArraySeed<T: Clone> {
  len: usize,
  seed: T,
}
impl<T: Clone> FixedLengthArraySeed<T> {
  pub fn new(len: usize, seed: T) -> Self {
    Self { len, seed }
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
      .and_then(|bytes| String::from_utf8(bytes).map_err(Error::custom))
  }
}

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
    FixedLengthArraySeed::new(self.array_size, self.seed).deserialize(deserializer)
  }
}

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
      .and_then(|chars| decode_ucs2(chars).map_err(Error::custom))
  }
}

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
    FixedLengthArraySeed::new(self.array_size, self.seed).deserialize(deserializer)
  }
}
