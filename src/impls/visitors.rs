
use std::{
  fmt::{self, Formatter},
  str::from_utf8,
  marker::PhantomData,
};

use serde::{
  Deserialize,
  de::{Error, SeqAccess, Unexpected, Visitor},
  __private::{from_utf8_lossy, size_hint}
};

use crate::error::VOTableError;

/// Structure made to visit a primitive or an optional primitive.
/// Attempts to visit a primitive different from the one it as been made for will fail.
pub struct VisitorPrim<E> {
  _marker: PhantomData<E>
}

/*
pub fn get_visitor<E>() -> VisitorPrim<E> {
  VisitorPrim {
    _marker: PhantomData
  }
}

macro_rules! primitive_visitor {
  ($ty:ty, $doc:tt, $method:ident) => {
    impl<'de> Visitor<'de> for VisitorPrim<$ty> {
      type Value = $ty;
      
      fn expecting(&self, formatter: & mut fmt::Formatter) -> fmt::Result {
        write!(formatter, $doc)
      }
      
      fn $method<E> (self, v: $ty) -> Result<Self::Value, E>
        where E: Error {
        Ok(v)
      }
    }
  };
}

primitive_visitor!(bool, "a boolean", visit_bool);
primitive_visitor!(char, "a char", visit_char);
primitive_visitor!(u8, "an u8", visit_u8);
primitive_visitor!(i16, "an i16", visit_i16);
primitive_visitor!(i32, "an i32", visit_i32);
primitive_visitor!(i64, "an i64", visit_i64);
primitive_visitor!(f32, "an f32", visit_f32);
primitive_visitor!(f64, "an f64", visit_f64);
*/

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
    let s = from_utf8_lossy(&buf[..n_bytes]);
    let mut iter = s.chars();
    match (iter.next(), iter.next()) {
      (Some(c), None) => Ok(c),
      _ => Err(Error::custom("Error transforming Unicode UCS-2 to UTF-16 char")),
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


/*
pub struct ComplexFloatVisitor;

impl<'a> Visitor<'a> for BytesVisitor {
  type Value = (f32, f32);

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a borrowed byte array")
  }

  fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>,
  {
    let seq = seq;
    
    seed: DeserializeSeed<'de>;
    
    seq.next_element_seed()
    seq.next_element_seed()  
    Err(Error::invalid_type(Unexpected::Seq, &self))
  }
}
*/



pub struct FixedLengthArrayVisitor<'de, T: Deserialize<'de>> {
  len: usize,
  _marker: &'de PhantomData<T>
}

impl<'de, T: Deserialize<'de>> FixedLengthArrayVisitor<'de, T> {
  pub fn new(len: usize) -> Self {
    Self { len, _marker: &PhantomData }
  }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for FixedLengthArrayVisitor<'de, T> {
  
  type Value = Vec<T>;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where 
      A: SeqAccess<'de>
  {
    let mut v: Vec<T> = Vec::with_capacity(self.len);
    for _ in 0..self.len {
      v.push(seq.next_element()?.ok_or_else(|| Error::custom(String::from("Premature end of stream")))?); 
    }
    Ok(v)
  }
}


pub struct VariableLengthArrayVisitor<'de, T: Deserialize<'de>> {
  _marker: &'de PhantomData<T>
}

impl<'de, T: Deserialize<'de>> Default for VariableLengthArrayVisitor<'de, T> {
    fn default() -> Self {
         Self::new()
    }
}

impl<'de, T: Deserialize<'de>> VariableLengthArrayVisitor<'de, T> {
  pub fn new() -> Self {
    Self { _marker: &PhantomData }
  }
}

impl<'de, T: Deserialize<'de>> Visitor<'de> for VariableLengthArrayVisitor<'de, T> {

  type Value = Vec<T>;

  fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
    formatter.write_str("an array")
  }

  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>
  {
    let mut v: Vec<T> = Vec::with_capacity(size_hint::cautious(seq.size_hint()));
    while let Some(value) = seq.next_element()? {
      v.push(value);
    }
    Ok(v)
  }
}

 