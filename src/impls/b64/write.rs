

use std::io::Write;
use base64::{
  write::EncoderWriter,
  engine::GeneralPurpose,
};
use byteorder::{BigEndian, WriteBytesExt};
use serde::{
  Serialize, Serializer, 
  ser::{SerializeSeq, SerializeTuple}
};
use serde::ser::{SerializeMap, SerializeStruct, SerializeStructVariant, SerializeTupleStruct, SerializeTupleVariant};

use crate::error::VOTableError;

const N_CHAR_PER_LINE: usize = 72;

pub struct B64Formatter<W: Write> {
  n_curr: usize,
  writer: W,
}

impl<W: Write> B64Formatter<W> {
  
  pub fn new(writer: W) -> Self {
    Self {
      n_curr: 0,
      writer
    }
  }
  
}

impl<W: Write> Write for B64Formatter<W> {
  
  fn write(&mut self, mut buf: &[u8]) -> std::io::Result<usize> {
    let buf_size = buf.len();
    while !buf.is_empty() {
      let n = N_CHAR_PER_LINE - self.n_curr;
      if n <= buf.len() {
        self.n_curr = 0;
        let (bl, br) = buf.split_at(n);
        self.writer.write_all(bl)?;
        self.writer.write_all(b"\n")?;
        buf = br;
      } else {
        self.n_curr += buf.len();
        let (bl, br) = buf.split_at(buf.len());
        self.writer.write_all(bl)?;
        buf = br;
      }
    }
    Ok(buf_size)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.writer.flush()
  }
}

pub struct BinarySerializer<W: Write> {
  writer: EncoderWriter<'static, GeneralPurpose, B64Formatter<W>>
}

impl<W: Write> BinarySerializer<W> {
  pub fn new(writer: EncoderWriter<'static, GeneralPurpose, B64Formatter<W>>) -> Self {
    Self { writer }
  }
}

impl<'a, W: Write> Serializer for &'a mut BinarySerializer<W> {
  
  type Ok = ();
  type Error = VOTableError;
  type SerializeSeq = SerializeSeqOrTuple<'a, W>;
  type SerializeTuple = SerializeSeqOrTuple<'a, W>;
  type SerializeTupleStruct = DummySerialize;
  type SerializeTupleVariant = DummySerialize;
  type SerializeMap = DummySerialize;
  type SerializeStruct = DummySerialize;
  type SerializeStructVariant = DummySerialize;

  fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
    unreachable!("bool serialize in u8 (because of the null value case '?' and of '[tTfF]'.")
  }

  fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
    unreachable!("No i8 in VOTable")
  }

  fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
    self.writer.write_i16::<BigEndian>(v).map_err(VOTableError::Io)
  }

  fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
    self.writer.write_i32::<BigEndian>(v).map_err(VOTableError::Io)
  }

  fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
    self.writer.write_i64::<BigEndian>(v).map_err(VOTableError::Io)
  }

  fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
    self.writer.write_u8(v).map_err(VOTableError::Io)
  }

  fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
    self.writer.write_u16::<BigEndian>(v).map_err(VOTableError::Io)
  }

  fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
    unreachable!("No u32 in VOTable")
  }

  fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
    unreachable!("No u64 in VOTable")
  }

  fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
    self.writer.write_f32::<BigEndian>(v).map_err(VOTableError::Io)
  }

  fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
    self.writer.write_f64::<BigEndian>(v).map_err(VOTableError::Io)
  }

  fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
    // Dedicated to unicode char (ASCII char serialized using u8)

    // In VOTable, unicode chars are encoded in UCS-2, 
    // see: https://stackoverflow.com/questions/36236364/why-java-char-uses-utf-16
    let mut buf = vec![0_u16; 3];
    let n_bytes = ucs2::encode(v.to_string().as_str(), &mut buf)
      .map_err(VOTableError::ToUCS2)?;
    debug_assert_eq!(n_bytes, 2);
    self.serialize_u16(buf[0])
  }

  fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
    unreachable!("Use serialize_seq or serialize_tuple to serialize str.")
  }

  fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
    unreachable!("Use serialize_seq or serialize_tuple to serialize bytes.")
  }

  fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
    unreachable!("No none in VOTable")
  }

  fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
    unreachable!("No some in VOTable")
  }

  fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
    unreachable!("No unit in VOTable")
  }

  fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
    unreachable!("No unit struct in VOTable")
  }

  fn serialize_unit_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str) -> Result<Self::Ok, Self::Error> {
    unreachable!("No unit variant in VOTable")
  }

  fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
    unreachable!("No newtpe struct in VOTable")
  }

  fn serialize_newtype_variant<T: ?Sized>(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _value: &T) -> Result<Self::Ok, Self::Error> where T: Serialize {
    unreachable!("No newtype variant in VOTable")
  }

  fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
    if let Some(len) = len {
      self.serialize_i32(len as i32)?;
      Ok(SerializeSeqOrTuple { ser: self })
    } else {
      unreachable!("We are supposed to know the sequence size in advance");
    }
  }

  fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
    // no need to write the len, it is known by the deserializer thanks to the schema
    Ok(SerializeSeqOrTuple { ser: self })
  }

  fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
    unreachable!("No tuple struct in VOTable")
  }

  fn serialize_tuple_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
    unreachable!("No tuple variant in VOTable")
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
    unreachable!("No map in VOTable")
  }

  fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
    unreachable!("No struct in VOTable")
  }

  fn serialize_struct_variant(self, _name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
    unreachable!("No struct variant in VOTable")
  }
}

pub struct SerializeSeqOrTuple<'a, W: Write> {
  ser: &'a mut BinarySerializer<W>
}

impl<'a, W: Write> SerializeSeq for SerializeSeqOrTuple<'a, W> {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
    value.serialize(&mut *self.ser)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

impl<'a, W: Write> SerializeTuple for SerializeSeqOrTuple<'a, W> {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
    value.serialize(&mut *self.ser)
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    Ok(())
  }
}

pub struct DummySerialize;

impl SerializeTupleStruct for DummySerialize {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error> where T: Serialize {
    unreachable!()
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    unreachable!()
  }
}

impl SerializeTupleVariant for DummySerialize {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error> where T: Serialize {
    unreachable!()
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    unreachable!()
  }
}

impl SerializeMap for DummySerialize {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error> where T: Serialize {
    unreachable!()
  }

  fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error> where T: Serialize {
    unreachable!()
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    unreachable!()
  }
}

impl SerializeStruct for DummySerialize {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error> where T: Serialize {
    unreachable!()
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    unreachable!()
  }
}

impl SerializeStructVariant for DummySerialize {
  type Ok = ();
  type Error = VOTableError;

  fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error> where T: Serialize {
    unreachable!()
  }

  fn end(self) -> Result<Self::Ok, Self::Error> {
    unreachable!()
  }
}
