//! Minimal buffer of a self-describing deserialized value, used to try deserializing the same
//! input into several types (like serde does for untagged enums).

use std::{fmt, marker::PhantomData};

use serde::{
  Deserialize, Deserializer,
  de::{self, IntoDeserializer, MapAccess, SeqAccess, Visitor},
  forward_to_deserialize_any,
};

/// Buffered value, as provided by a self-describing deserializer.
#[derive(Debug)]
pub enum Content {
  Bool(bool),
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  I8(i8),
  I16(i16),
  I32(i32),
  I64(i64),
  F32(f32),
  F64(f64),
  Char(char),
  String(String),
  Bytes(Vec<u8>),
  None,
  Some(Box<Content>),
  Unit,
  Newtype(Box<Content>),
  Seq(Vec<Content>),
  Map(Vec<(Content, Content)>),
}

impl Content {
  /// Returns `true` if the content is a unit or a none value.
  pub fn is_unit_or_none(&self) -> bool {
    matches!(self, Content::Unit | Content::None)
  }
}

impl<'de> Deserialize<'de> for Content {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_any(ContentVisitor)
  }
}

struct ContentVisitor;

impl<'de> Visitor<'de> for ContentVisitor {
  type Value = Content;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("any value")
  }

  fn visit_bool<E: de::Error>(self, v: bool) -> Result<Content, E> {
    Ok(Content::Bool(v))
  }
  fn visit_i8<E: de::Error>(self, v: i8) -> Result<Content, E> {
    Ok(Content::I8(v))
  }
  fn visit_i16<E: de::Error>(self, v: i16) -> Result<Content, E> {
    Ok(Content::I16(v))
  }
  fn visit_i32<E: de::Error>(self, v: i32) -> Result<Content, E> {
    Ok(Content::I32(v))
  }
  fn visit_i64<E: de::Error>(self, v: i64) -> Result<Content, E> {
    Ok(Content::I64(v))
  }
  fn visit_u8<E: de::Error>(self, v: u8) -> Result<Content, E> {
    Ok(Content::U8(v))
  }
  fn visit_u16<E: de::Error>(self, v: u16) -> Result<Content, E> {
    Ok(Content::U16(v))
  }
  fn visit_u32<E: de::Error>(self, v: u32) -> Result<Content, E> {
    Ok(Content::U32(v))
  }
  fn visit_u64<E: de::Error>(self, v: u64) -> Result<Content, E> {
    Ok(Content::U64(v))
  }
  fn visit_f32<E: de::Error>(self, v: f32) -> Result<Content, E> {
    Ok(Content::F32(v))
  }
  fn visit_f64<E: de::Error>(self, v: f64) -> Result<Content, E> {
    Ok(Content::F64(v))
  }
  fn visit_char<E: de::Error>(self, v: char) -> Result<Content, E> {
    Ok(Content::Char(v))
  }
  fn visit_str<E: de::Error>(self, v: &str) -> Result<Content, E> {
    Ok(Content::String(v.to_owned()))
  }
  fn visit_string<E: de::Error>(self, v: String) -> Result<Content, E> {
    Ok(Content::String(v))
  }
  fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Content, E> {
    Ok(Content::Bytes(v.to_vec()))
  }
  fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Content, E> {
    Ok(Content::Bytes(v))
  }
  fn visit_unit<E: de::Error>(self) -> Result<Content, E> {
    Ok(Content::Unit)
  }
  fn visit_none<E: de::Error>(self) -> Result<Content, E> {
    Ok(Content::None)
  }
  fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Content, D::Error> {
    Content::deserialize(deserializer).map(|v| Content::Some(Box::new(v)))
  }
  fn visit_newtype_struct<D: Deserializer<'de>>(
    self,
    deserializer: D,
  ) -> Result<Content, D::Error> {
    Content::deserialize(deserializer).map(|v| Content::Newtype(Box::new(v)))
  }
  fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<Content, V::Error> {
    let mut v = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(4096));
    while let Some(e) = seq.next_element()? {
      v.push(e);
    }
    Ok(Content::Seq(v))
  }
  fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Content, V::Error> {
    let mut v = Vec::with_capacity(map.size_hint().unwrap_or(0).min(4096));
    while let Some(kv) = map.next_entry()? {
      v.push(kv);
    }
    Ok(Content::Map(v))
  }
}

/// Deserializer reading from a borrowed, buffered, [`Content`].
pub struct ContentRefDeserializer<'a, E> {
  content: &'a Content,
  err: PhantomData<E>,
}

impl<'a, E> ContentRefDeserializer<'a, E> {
  pub fn new(content: &'a Content) -> Self {
    Self {
      content,
      err: PhantomData,
    }
  }
}

impl<E> Clone for ContentRefDeserializer<'_, E> {
  fn clone(&self) -> Self {
    *self
  }
}
impl<E> Copy for ContentRefDeserializer<'_, E> {}

impl<'de, 'a, E: de::Error> IntoDeserializer<'de, E> for &'a Content {
  type Deserializer = ContentRefDeserializer<'a, E>;

  fn into_deserializer(self) -> Self::Deserializer {
    ContentRefDeserializer::new(self)
  }
}

impl<'de, E: de::Error> Deserializer<'de> for ContentRefDeserializer<'_, E> {
  type Error = E;

  fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
    match self.content {
      Content::Bool(v) => visitor.visit_bool(*v),
      Content::U8(v) => visitor.visit_u8(*v),
      Content::U16(v) => visitor.visit_u16(*v),
      Content::U32(v) => visitor.visit_u32(*v),
      Content::U64(v) => visitor.visit_u64(*v),
      Content::I8(v) => visitor.visit_i8(*v),
      Content::I16(v) => visitor.visit_i16(*v),
      Content::I32(v) => visitor.visit_i32(*v),
      Content::I64(v) => visitor.visit_i64(*v),
      Content::F32(v) => visitor.visit_f32(*v),
      Content::F64(v) => visitor.visit_f64(*v),
      Content::Char(v) => visitor.visit_char(*v),
      Content::String(v) => visitor.visit_str(v),
      Content::Bytes(v) => visitor.visit_bytes(v),
      Content::None => visitor.visit_none(),
      Content::Some(v) => visitor.visit_some(ContentRefDeserializer::new(v)),
      Content::Unit => visitor.visit_unit(),
      Content::Newtype(v) => visitor.visit_newtype_struct(ContentRefDeserializer::new(v)),
      Content::Seq(v) => {
        let mut seq = de::value::SeqDeserializer::new(v.iter());
        let value = visitor.visit_seq(&mut seq)?;
        // Fails if not all elements have been consumed (e.g. tuple shorter than the sequence)
        seq.end()?;
        Ok(value)
      }
      Content::Map(v) => {
        let mut map = de::value::MapDeserializer::new(v.iter().map(|(k, v)| (k, v)));
        let value = visitor.visit_map(&mut map)?;
        map.end()?;
        Ok(value)
      }
    }
  }

  fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
    match self.content {
      Content::None | Content::Unit => visitor.visit_none(),
      Content::Some(v) => visitor.visit_some(ContentRefDeserializer::new(v)),
      _ => visitor.visit_some(self),
    }
  }

  fn deserialize_newtype_struct<V: Visitor<'de>>(
    self,
    _name: &'static str,
    visitor: V,
  ) -> Result<V::Value, E> {
    match self.content {
      Content::Newtype(v) => visitor.visit_newtype_struct(ContentRefDeserializer::new(v)),
      _ => visitor.visit_newtype_struct(self),
    }
  }

  forward_to_deserialize_any! {
    bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
    bytes byte_buf unit unit_struct seq tuple tuple_struct map struct enum
    identifier ignored_any
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::de::value::Error;

  fn deser<'a, T: Deserialize<'a>>(content: &'a Content) -> Result<T, Error> {
    T::deserialize(ContentRefDeserializer::<Error>::new(content))
  }

  #[test]
  fn test_content_ref_deserializer() {
    let c = Content::Seq(vec![Content::F64(1.5), Content::F64(2.5)]);
    assert_eq!(deser::<(f64, f64)>(&c).unwrap(), (1.5, 2.5));
    // Tuples must consume the whole sequence
    let c = Content::Seq(vec![Content::F64(1.5), Content::F64(2.5), Content::U8(3)]);
    assert!(deser::<(f64, f64)>(&c).is_err());
    assert_eq!(deser::<Vec<f64>>(&c).unwrap(), vec![1.5, 2.5, 3.0]);
    // Integer conversions
    assert_eq!(deser::<u8>(&Content::I64(255)).unwrap(), 255);
    assert!(deser::<u8>(&Content::I64(256)).is_err());
    // Options
    let c = Content::Seq(vec![Content::Bool(true), Content::Unit, Content::None]);
    assert_eq!(
      deser::<Vec<Option<bool>>>(&c).unwrap(),
      vec![Some(true), None, None]
    );
    // Single char string
    assert_eq!(deser::<char>(&Content::String("a".into())).unwrap(), 'a');
    assert!(deser::<char>(&Content::String("ab".into())).is_err());
  }
}
