
use std::io::{self, Bytes, Read, BufRead, Error, ErrorKind, BufReader};

use serde::{Deserializer, de::Visitor};
use base64::{
  engine::GeneralPurpose,
  read::DecoderReader,
};
use byteorder::{BigEndian, ReadBytesExt};

use crate::error::VOTableError;

/// Take a Byte iterator from a BufRead and remove the '\n', 'r' and ' ' characters.
/// We recall that the allowed characters in base64 are: '0-9a-zA-Z+-' and '=' (for padding).
/// but for display purposes other characters may be added in the VOTable base 64 stream.
///
/// This object can then be decorated by a [DecoderReader](https://docs.rs/base64/latest/base64/read/struct.DecoderReader.html).
///
/// # Remark
///   This Bytes based implementation (iterating char by char) is probably not the most
///   efficient, but is quite simple to implement. To be changed if performances are really poor.
pub struct B64Cleaner<'a, R: BufRead>{
  bytes: Bytes<&'a mut R>,
  is_over: bool,
}

impl<'a, R: BufRead> B64Cleaner<'a, R> {

  pub fn new(reader: &'a mut R) -> Self {
    Self {
      bytes: reader.bytes(),
      is_over: false,
    }
  }

  pub fn is_over(&self) -> bool {
    self.is_over
  }

}

impl<'a, R: BufRead> Read for B64Cleaner<'a, R> {

  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    if self.is_over {
      return Ok(0);
    }
    for (i, byte) in buf.iter_mut().enumerate() {
      *byte = loop {
        match self.bytes.next() {
          Some(read_byte) => {
            match read_byte? {
              // Simply ignore blank and carriage return (possibly added for formatting purpose)
              b'\n' | b'\t' | b' ' => continue,
              // Return when we detect the beginning of the </STREAM> tag
              b'<' => {
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'/');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'S');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'T');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'R');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'E');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'A');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'M');
                assert_eq!(self.bytes.next().unwrap().unwrap(), b'>');
                self.is_over = true;
                return Ok(i)
              },
              // Valid b64 chars are 0-9a-zA-Z+/= (= for padding only), we let the base64 decoder 
              // throw an error, no need to check them here
              b => break b,
            }
          }
          None => return Err(Error::new(ErrorKind::UnexpectedEof, "Premature end of b64 encoded binary data")),
        }
      }
    }
    Ok(buf.len())
  }
}

pub struct BinaryDeserializer<'a, R: BufRead> {
  reader: BufReader<DecoderReader<'static, GeneralPurpose, B64Cleaner<'a, R>>>
}

impl<'de, 'a, R: BufRead> BinaryDeserializer<'a, R> {

  pub fn new(reader: DecoderReader<'static, GeneralPurpose, B64Cleaner<'a, R>>) -> Self {
    Self { reader: BufReader::new(reader) }
  }

  pub fn has_data_left(&mut self) -> Result<bool, io::Error> {
    self.reader.fill_buf().map(|b| !b.is_empty())
  }


  /*
  pub fn deserialize_row(&'a mut self, row_schema: &[Schema]) -> Result<Vec<VOTableValue>, VOTableError> {
    let mut row: Vec<VOTableValue> = Vec::with_capacity(row_schema.len());
    for field_schema in row_schema {
      let field = field_schema.deserialize(&mut *self)?;
      row.push(field);
    }
    Ok(row)
  }
  */
}

// <'de, 'a: 'de, R: BufRead>
//   'a lasts at least as long as 'de
impl<'de, 'b, 'a: 'b, R: BufRead> Deserializer<'de> for &'b mut BinaryDeserializer<'a, R> {

  type Error = VOTableError;

  fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No any in VOTable since we use a schema")
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    let byte = self.reader.read_u8().map_err(VOTableError::Io)?;
    match byte {
      b'F' | b'f' | b'0' => visitor.visit_bool(false),
      b'T' | b't' | b'1' => visitor.visit_bool(true),
      _ => visitor.visit_none() // TODO: not implemented!! Should implement a opt_bool visitor instead :o/
    }
  }

  fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No i8 in VOTable")
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    visitor.visit_i16(self.reader.read_i16::<BigEndian>().map_err(VOTableError::Io)?)
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    visitor.visit_i32(self.reader.read_i32::<BigEndian>().map_err(VOTableError::Io)?)
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    visitor.visit_i64(self.reader.read_i64::<BigEndian>().map_err(VOTableError::Io)?)
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    visitor.visit_u8(self.reader.read_u8().map_err(VOTableError::Io)?)
  }

  fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No u16 in VOTable")
  }

  fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No u32 in VOTable")
  }

  fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No u64 in VOTable")
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    visitor.visit_f32(self.reader.read_f32::<BigEndian>().map_err(VOTableError::Io)?)
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    visitor.visit_f64(self.reader.read_f64::<BigEndian>().map_err(VOTableError::Io)?)
  }

  fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    // We deserialize either a u8 or u16 using a CharVisitor.
    unreachable!("Not used because there is a difference between ASCII and Unicode chars")
  }

  fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("Not used because there is a difference between ASCII and Unicode Strings")
  }

  fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("Not used because there is a difference between ASCII and Unicode Strings")
  }

  fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("Use deserialize_seq or deserialize_tuple to deserialize bytes")
  }

  fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("Use deserialize_seq or deserialize_tuple to deserialize bytes")
  }

  fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No option in VOTable binary data")
  }

  fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No unit in VOTable binary data")
  }

  fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No unit struct in VOTable binary data")
  }

  fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No newtype struct in VOTable binary data")
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    // used to deserialize variable length array
    struct Access<'b, 'a: 'b, R: BufRead> {
      deserializer: &'b mut BinaryDeserializer<'a, R>,
      len: usize,
    }

    impl<'de, 'b, 'a: 'b, R: BufRead> serde::de::SeqAccess<'de> for Access<'b, 'a, R>
    {
      type Error = VOTableError;

      fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
          T: serde::de::DeserializeSeed<'de>,
      {
        if self.len > 0 {
          self.len -= 1;
          let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
          Ok(Some(value))
        } else {
          Ok(None)
        }
      }

      fn size_hint(&self) -> Option<usize> {
        Some(self.len)
      }
    }

    let len = self.reader.read_i32::<BigEndian>().map_err(VOTableError::Io)? as usize;
    visitor.visit_seq(Access {
      deserializer: self,
      len,
    })
  }

  fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    // used to deserialize fixed length array and rows
    struct Access<'b, 'a: 'b, R: BufRead> {
      deserializer: &'b mut BinaryDeserializer<'a, R>,
      len: usize,
    }

    impl<'de, 'b, 'a: 'b, R: BufRead> serde::de::SeqAccess<'de> for Access<'b, 'a, R>
    {
      type Error = VOTableError;

      fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
          T: serde::de::DeserializeSeed<'de>,
      {
        if self.len > 0 {
          self.len -= 1;
          let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
          Ok(Some(value))
        } else {
          Ok(None)
        }
      }

      fn size_hint(&self) -> Option<usize> {
        Some(self.len)
      }
    }

    visitor.visit_seq(Access {
      deserializer: self,
      len,
    })
  }

  fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No tuple struct in VOTable binary data")
  }

  fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No map in VOTable binary data")
  }

  fn deserialize_struct<V>(self, _name: &'static str, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No struct in VOTable binary data")
  }

  fn deserialize_enum<V>(self, _name: &'static str, _variants: &'static [&'static str], _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No map in VOTable binary data")
  }

  fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("No identifier in VOTable binary data")
  }

  fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
    unreachable!("We have to read everything in VOTable binary data")
  }
}
