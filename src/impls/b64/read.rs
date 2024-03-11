use std::{
  io::{self, BufRead, BufReader, Error, ErrorKind, Read},
  mem::size_of,
};

use base64::{
  engine::{general_purpose, GeneralPurpose},
  read::DecoderReader,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{de::Visitor, Deserializer};

use crate::{error::VOTableError, impls::Schema};

/// Take a Byte iterator from a BufRead and remove the '\n', 'r' and ' ' characters.
/// We recall that the allowed characters in base64 are: '0-9a-zA-Z+-' and '=' (for padding).
/// but for display purposes other characters may be added in the VOTable base 64 stream.
///
/// This object can then be decorated by a [DecoderReader](https://docs.rs/base64/latest/base64/read/struct.DecoderReader.html).
///
/// # Remark
///   This Bytes based implementation (iterating char by char) is probably not the most
///   efficient, but is quite simple to implement. To be changed if performances are really poor.
///   Also, have a look at [memchr](https://docs.rs/memchr/latest/memchr/)
pub struct B64Cleaner<'a, R: BufRead> {
  reader: &'a mut R,
  is_over: bool,
}

impl<'a, R: BufRead> B64Cleaner<'a, R> {
  pub fn new(reader: &'a mut R) -> Self {
    Self {
      reader,
      is_over: false,
    }
  }

  pub fn is_over(&self) -> bool {
    self.is_over
  }

  pub fn into_inner(self) -> &'a mut R {
    self.reader
  }
}

impl<'a, R: BufRead> Read for B64Cleaner<'a, R> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    if self.is_over {
      return Ok(0);
    }
    for (i, byte) in buf.iter_mut().enumerate() {
      let mut bytes = (&mut *self.reader).bytes();
      *byte = loop {
        match bytes.next() {
          Some(read_byte) => {
            match read_byte? {
              // Simply ignore blank and carriage return (possibly added for formatting purpose)
              b'\n' | b'\t' | b' ' => continue,
              // Return when we detect the beginning of the </STREAM> tag
              b'<' => {
                assert_eq!(bytes.next().unwrap().unwrap(), b'/');
                assert_eq!(bytes.next().unwrap().unwrap(), b'S');
                assert_eq!(bytes.next().unwrap().unwrap(), b'T');
                assert_eq!(bytes.next().unwrap().unwrap(), b'R');
                assert_eq!(bytes.next().unwrap().unwrap(), b'E');
                assert_eq!(bytes.next().unwrap().unwrap(), b'A');
                assert_eq!(bytes.next().unwrap().unwrap(), b'M');
                assert_eq!(bytes.next().unwrap().unwrap(), b'>');
                self.is_over = true;
                return Ok(i);
              }
              // Valid b64 chars are 0-9a-zA-Z+/= (= for padding only), we let the base64 decoder
              // throw an error, no need to check them here
              b => break b,
            }
          }
          None => {
            return Err(Error::new(
              ErrorKind::UnexpectedEof,
              "Premature end of b64 encoded binary data",
            ))
          }
        }
      }
    }
    Ok(buf.len())
  }
}

pub struct BulkBinaryRowDeserializer<'a, R: BufRead> {
  reader: BufReader<DecoderReader<'static, GeneralPurpose, B64Cleaner<'a, R>>>,
  bulk_reader: Vec<BulkReaderElem>,
}

impl<'a, R: BufRead> BulkBinaryRowDeserializer<'a, R> {
  pub fn new_binary(
    reader: DecoderReader<'static, GeneralPurpose, B64Cleaner<'a, R>>,
    schemas: &[Schema],
  ) -> Self {
    Self {
      reader: BufReader::new(reader),
      bulk_reader: BulkReaderElem::from_schemas(schemas, false),
    }
  }

  pub fn new_binary2(
    reader: DecoderReader<'static, GeneralPurpose, B64Cleaner<'a, R>>,
    schemas: &[Schema],
  ) -> Self {
    Self {
      reader: BufReader::new(reader),
      bulk_reader: BulkReaderElem::from_schemas(schemas, true),
    }
  }

  pub fn has_data_left(&mut self) -> Result<bool, io::Error> {
    self.reader.fill_buf().map(|b| !b.is_empty())
  }

  pub fn read_raw_row(&mut self, buf: &mut Vec<u8>) -> Result<usize, VOTableError> {
    BulkReaderElem::read_all(self.bulk_reader.as_slice(), &mut self.reader, buf)
  }
}

enum BulkReaderElem {
  Fixed { n_bytes: usize },
  VariableBits,
  VariableBytes { n_bytes_by_elem: usize },
}

impl BulkReaderElem {
  fn from_schemas(schemas: &[Schema], binary2: bool) -> Vec<Self> {
    let mut elems: Vec<BulkReaderElem> = Vec::new();
    let mut prev_n_bytes = if binary2 {
      // Fixed Array of bytes
      (schemas.len() + 7) / 8 // bytes for the null flags
    } else {
      0_usize
    };
    for schema in schemas {
      match schema.byte_len() {
        Ok(n_bytes) => prev_n_bytes += n_bytes,
        Err((0, _)) => {
          if prev_n_bytes != 0 {
            elems.push(BulkReaderElem::Fixed {
              n_bytes: prev_n_bytes,
            });
            prev_n_bytes = 0;
          }
          elems.push(BulkReaderElem::VariableBits);
        }
        Err((n_bytes, _)) => {
          if prev_n_bytes != 0 {
            elems.push(BulkReaderElem::Fixed {
              n_bytes: prev_n_bytes,
            });
            prev_n_bytes = 0;
          }
          elems.push(BulkReaderElem::VariableBytes {
            n_bytes_by_elem: n_bytes,
          });
        }
      }
    }
    if prev_n_bytes != 0 {
      elems.push(BulkReaderElem::Fixed {
        n_bytes: prev_n_bytes,
      });
    }
    elems
  }

  /// Put in `buf` the bytes of `elems` read from the given `reader`.
  fn read_all<R: BufRead>(
    elems: &[Self],
    mut reader: R,
    buf: &mut Vec<u8>,
  ) -> Result<usize, VOTableError> {
    let read_write_len = |reader: &mut R, buf: &mut Vec<u8>| {
      reader
        .read_i32::<BigEndian>()
        .and_then(|len| buf.write_i32::<BigEndian>(len).map(|()| len))
        .map_err(VOTableError::Io)
    };
    let mut cur = 0;
    for elem in elems {
      let n_bytes = match elem {
        BulkReaderElem::Fixed { n_bytes } => *n_bytes,
        BulkReaderElem::VariableBits => {
          let len = read_write_len(&mut reader, buf)?;
          cur += size_of::<i32>();
          ((len + 7) / 8) as usize
        }
        BulkReaderElem::VariableBytes { n_bytes_by_elem } => {
          let len = read_write_len(&mut reader, buf)?;
          cur += size_of::<i32>();
          *n_bytes_by_elem * (len as usize)
        }
      };
      let mut copy = vec![0_u8; n_bytes];
      reader
        .read_exact(copy.as_mut_slice())
        .map_err(VOTableError::Io)?;
      buf.append(&mut copy);
      cur += n_bytes;
    }
    Ok(cur)
  }
}

// Owned version of B64Cleaner...
pub struct OwnedB64Cleaner<R: BufRead> {
  reader: R,
  is_over: bool,
}

impl<R: BufRead> OwnedB64Cleaner<R> {
  pub fn new(reader: R) -> Self {
    Self {
      reader,
      is_over: false,
    }
  }

  pub fn is_over(&self) -> bool {
    self.is_over
  }

  pub fn into_inner(self) -> R {
    self.reader
  }
}

impl<R: BufRead> Read for OwnedB64Cleaner<R> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    if self.is_over {
      return Ok(0);
    }
    let mut bytes = (&mut self.reader).bytes();
    for (i, byte) in buf.iter_mut().enumerate() {
      *byte = loop {
        match bytes.next() {
          Some(read_byte) => {
            match read_byte? {
              // Simply ignore blank and carriage return (possibly added for formatting purpose)
              b'\n' | b'\t' | b' ' => continue,
              // Return when we detect the beginning of the </STREAM> tag
              b'<' => {
                assert_eq!(bytes.next().unwrap().unwrap(), b'/');
                assert_eq!(bytes.next().unwrap().unwrap(), b'S');
                assert_eq!(bytes.next().unwrap().unwrap(), b'T');
                assert_eq!(bytes.next().unwrap().unwrap(), b'R');
                assert_eq!(bytes.next().unwrap().unwrap(), b'E');
                assert_eq!(bytes.next().unwrap().unwrap(), b'A');
                assert_eq!(bytes.next().unwrap().unwrap(), b'M');
                assert_eq!(bytes.next().unwrap().unwrap(), b'>');
                self.is_over = true;
                return Ok(i);
              }
              // Valid b64 chars are 0-9a-zA-Z+/= (= for padding only), we let the base64 decoder
              // throw an error, no need to check them here
              b => break b,
            }
          }
          None => {
            return Err(Error::new(
              ErrorKind::UnexpectedEof,
              "Premature end of b64 encoded binary data",
            ))
          }
        }
      }
    }

    /* Don't know if wa can do better than iterating char by char...
    The best option may be to implement our own base64::read::DecoderReader ...
    (to avoid a copy)
    loop {
      let mut available = match self.reader.fill_buf() {
        Ok(n) => n,
        Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
        Err(e) => return Err(e),
      };
      if let Some(to) = memchr::memchr(b'<', available) {
        available = available[..i];
      };
      // TO BE DONE!!
      match memchr::memchr(b'<', available) {
        Some(i) => {
          buf.extend_from_slice(&available[..=i]);
          (true, i + 1)
        }
        None => {
          buf.extend_from_slice(available);
          (false, available.len())
        }
      }
    }*/

    Ok(buf.len())
  }
}

// Owned version of BulkBinaryRowDeserializer
pub struct OwnedBulkBinaryRowDeserializer<R: BufRead> {
  reader: BufReader<DecoderReader<'static, GeneralPurpose, OwnedB64Cleaner<R>>>,
  bulk_reader: Vec<BulkReaderElem>,
}

impl<R: BufRead> OwnedBulkBinaryRowDeserializer<R> {
  pub fn new_binary(
    reader: DecoderReader<'static, GeneralPurpose, OwnedB64Cleaner<R>>,
    schemas: &[Schema],
  ) -> Self {
    Self {
      reader: BufReader::new(reader),
      bulk_reader: BulkReaderElem::from_schemas(schemas, false),
    }
  }

  pub fn new_binary2(
    reader: DecoderReader<'static, GeneralPurpose, OwnedB64Cleaner<R>>,
    schemas: &[Schema],
  ) -> Self {
    Self {
      reader: BufReader::new(reader),
      bulk_reader: BulkReaderElem::from_schemas(schemas, true),
    }
  }

  pub fn has_data_left(&mut self) -> Result<bool, VOTableError> {
    self
      .reader
      .fill_buf()
      .map(|b| !b.is_empty())
      .map_err(VOTableError::Io)
  }

  pub fn read_raw_row(&mut self, buf: &mut Vec<u8>) -> Result<usize, VOTableError> {
    BulkReaderElem::read_all(self.bulk_reader.as_slice(), &mut self.reader, buf)
  }

  pub fn skip_remaining_data(self) -> Result<Self, VOTableError> {
    // Retrieve the inner most reader
    let Self {
      reader,
      bulk_reader,
    } = self;
    let mut reader = reader.into_inner().into_inner().into_inner();
    // Read (discarding content till '</STREAM>' is reached
    // - Adapted from https://doc.rust-lang.org/src/std/io/mod.rs.html but discarding the read elements
    fn read_until<R: BufRead>(r: &mut R, delim: u8) -> Result<usize, VOTableError> {
      let mut read = 0;
      loop {
        let (done, used) = {
          let available = match r.fill_buf() {
            Ok(n) => n,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(VOTableError::Io(e)),
          };
          match memchr::memchr(delim, available) {
            Some(i) => (true, i + 1),
            None => (false, available.len()),
          }
        };
        r.consume(used);
        read += used;
        if done || used == 0 {
          return Ok(read);
        }
      }
    }
    // - Here we assumed we only have base64 characters or '\n', '\t', ... (no comments, CDSATA, ...)
    //   before the </STREAM>
    read_until(&mut reader, b'<').and_then(|_| {
      let mut buf = [0_u8; 8];
      reader
        .read_exact(&mut buf)
        .map_err(VOTableError::Io)
        .and_then(|()| {
          if &buf == b"/STREAM>" {
            Ok(())
          } else {
            Err(VOTableError::Custom(format!(
              "Unexpected input. Expected: '/STREAM>'. Actual: {:?}'",
              std::str::from_utf8(&buf)
            )))
          }
        })
    })?;
    // Re-create the interenal reader, but with over=true
    let reader = BufReader::new(DecoderReader::new(
      OwnedB64Cleaner {
        reader,
        is_over: true,
      },
      &general_purpose::STANDARD,
    ));
    Ok(Self {
      reader,
      bulk_reader,
    })
  }

  pub fn into_inner(self) -> R {
    self.reader.into_inner().into_inner().into_inner()
  }
}

/// Read from binary data.
/*pub struct BinaryDeserializer<'a, R: BufRead> {
  reader: BufReader<DecoderReader<'static, GeneralPurpose, B64Cleaner<'a, R>>>,
}*/
pub struct BinaryDeserializer<R: BufRead> {
  reader: R,
}

impl<R: BufRead> BinaryDeserializer<R> {
  pub fn new(reader: R) -> Self {
    Self { reader }
  }
  pub fn has_data_left(&mut self) -> Result<bool, io::Error> {
    self.reader.fill_buf().map(|b| !b.is_empty())
  }
  pub fn into_inner(self) -> R {
    self.reader
  }
}

impl<'de, 'b, R: BufRead> Deserializer<'de> for &'b mut BinaryDeserializer<R> {
  type Error = VOTableError;

  fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No any in VOTable since we use a schema")
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    let byte = self.reader.read_u8().map_err(VOTableError::Io)?;
    match byte {
      b'F' | b'f' | b'0' => visitor.visit_bool(false),
      b'T' | b't' | b'1' => visitor.visit_bool(true),
      _ => visitor.visit_none(), // TODO: not implemented!! Should implement a opt_bool visitor instead :o/
    }
  }

  fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No i8 in VOTable")
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_i16(
      self
        .reader
        .read_i16::<BigEndian>()
        .map_err(VOTableError::Io)?,
    )
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_i32(
      self
        .reader
        .read_i32::<BigEndian>()
        .map_err(VOTableError::Io)?,
    )
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_i64(
      self
        .reader
        .read_i64::<BigEndian>()
        .map_err(VOTableError::Io)?,
    )
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_u8(self.reader.read_u8().map_err(VOTableError::Io)?)
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    // Only used for unicode chars with the CharVisitor
    self
      .reader
      .read_u16::<BigEndian>()
      .map_err(VOTableError::Io)
      .and_then(|v| visitor.visit_u16(v))
  }

  fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No u32 in VOTable")
  }

  fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No u64 in VOTable")
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_f32(
      self
        .reader
        .read_f32::<BigEndian>()
        .map_err(VOTableError::Io)?,
    )
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    visitor.visit_f64(
      self
        .reader
        .read_f64::<BigEndian>()
        .map_err(VOTableError::Io)?,
    )
  }

  fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    // We deserialize either a u8 or u16 using a CharVisitor.
    unreachable!("Not used because there is a difference between ASCII and Unicode chars")
  }

  fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("Not used because there is a difference between ASCII and Unicode Strings")
  }

  fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("Not used because there is a difference between ASCII and Unicode Strings")
  }

  fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("Use deserialize_seq or deserialize_tuple to deserialize bytes")
  }

  fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("Use deserialize_seq or deserialize_tuple to deserialize bytes")
  }

  fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No option in VOTable binary data")
  }

  fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No unit in VOTable binary data")
  }

  fn deserialize_unit_struct<V>(
    self,
    _name: &'static str,
    _visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No unit struct in VOTable binary data")
  }

  fn deserialize_newtype_struct<V>(
    self,
    _name: &'static str,
    _visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No newtype struct in VOTable binary data")
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    // used to deserialize variable length array
    /*struct Access<'b, 'a: 'b, R: BufRead> {
      deserializer: &'b mut BinaryDeserializer<'a, R>,
      len: usize,
    }*/
    struct Access<'b, R: BufRead> {
      deserializer: &'b mut BinaryDeserializer<R>,
      len: usize,
    }

    // impl<'de, 'b, 'a: 'b, R: BufRead> serde::de::SeqAccess<'de> for Access<'b, 'a, R> {
    impl<'de, 'b, R: BufRead> serde::de::SeqAccess<'de> for Access<'b, R> {
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

    let len = self
      .reader
      .read_i32::<BigEndian>()
      .map_err(VOTableError::Io)? as usize;
    visitor.visit_seq(Access {
      deserializer: self,
      len,
    })
  }

  fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    // used to deserialize fixed length array and rows
    struct Access<'b, R: BufRead> {
      deserializer: &'b mut BinaryDeserializer<R>,
      len: usize,
    }

    impl<'de, 'b, R: BufRead> serde::de::SeqAccess<'de> for Access<'b, R> {
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

  fn deserialize_tuple_struct<V>(
    self,
    _name: &'static str,
    _len: usize,
    _visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No tuple struct in VOTable binary data")
  }

  fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No map in VOTable binary data")
  }

  fn deserialize_struct<V>(
    self,
    _name: &'static str,
    _fields: &'static [&'static str],
    _visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No struct in VOTable binary data")
  }

  fn deserialize_enum<V>(
    self,
    _name: &'static str,
    _variants: &'static [&'static str],
    _visitor: V,
  ) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No map in VOTable binary data")
  }

  fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("No identifier in VOTable binary data")
  }

  fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    unreachable!("We have to read everything in VOTable binary data")
  }
}
