
use std::{
  str, 
  collections::HashMap,
  io::{BufRead, Write},
};

use quick_xml::{Reader, Writer, events::{Event, BytesText, attributes::Attributes}};

use paste::paste;

use serde_json::Value;

use super::{QuickXmlReadWrite, error::VOTableError};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Info {
  // attributes
  #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub value: String,
  // ??
  #[serde(skip_serializing_if = "Option::is_none")]
  pub xtype: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // content
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
}

impl Info {
  pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
    Info {
      id: None,
      name: name.into(),
      value: value.into(),
      xtype: None,
      ref_: None,
      unit: None,
      ucd: None,
      utype: None,
      extra: Default::default(),
      content: None,
    }
  }

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(xtype);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(unit);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);

  impl_builder_insert_extra!();

  impl_builder_opt_string_attr!(content);
}

impl QuickXmlReadWrite for Info {
  const TAG: &'static str = "INFO";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut info = Self::new(NULL, NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      info = match attr.key {
        b"ID" => info.set_id(value),
        b"name" => { info.name = value.to_string(); info },
        b"value" => { info.value = value.to_string(); info },
        b"xtype" => info.set_xtype(value),
        b"ref" => info.set_ref(value),
        b"unit" => info.set_unit(value),
        b"ucd" => info.set_ucd(value),
        b"utype" => info.set_utype(value),
        _ => info.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    if info.name.as_str() == NULL || info.value.as_str() == NULL {
      Err(VOTableError::Custom(format!("Attributes 'name' and 'value' are mandatory in tag '{}'", Self::TAG))) 
    } else {
      Ok(info)
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    read_content!(Self, self, reader, reader_buff)
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    read_content_by_ref!(Self, self, reader, reader_buff)
  }
  
  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>,
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    write_opt_string_attr!(self, elem_writer, ID);
    elem_writer = elem_writer.with_attribute(("name", self.name.as_str()));
    elem_writer = elem_writer.with_attribute(("value", self.value.as_str()));
    write_opt_string_attr!(self, elem_writer, xtype);
    write_opt_string_attr!(self, elem_writer, ref_, "ref");
    write_opt_string_attr!(self, elem_writer, unit);
    write_opt_string_attr!(self, elem_writer, ucd);
    write_opt_string_attr!(self, elem_writer, utype);
    write_extra!(self, elem_writer);
    write_content!(self, elem_writer);
    Ok(())
  }
}



#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{Reader, events::Event, Writer};
  use crate::{
    QuickXmlReadWrite,
    info::Info,
  };
  
  fn test_info_read(xml: &str) -> Info {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == Info::TAG_BYTES => {
          let mut info = Info::from_attributes(e.attributes()).unwrap();
          info.read_sub_elements_and_clean(reader, &mut buff, &()).unwrap();
          return info;
        }
        Event::Empty(ref mut e) if e.local_name() == Info::TAG_BYTES => {
          let info = Info::from_attributes(e.attributes()).unwrap();
          return info;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        _ => unreachable!(),
      }
    }
  }
  
  #[test]
  fn test_info_readwrite_1() {
    let xml = r#"<INFO ID="VERSION" name="votable-version" value="1.99+ (14-Oct-2013)"/>"#;
    // Test read
    let mut info = test_info_read(xml);
    assert_eq!(info.id.as_ref().map(|s| s.as_str()), Some("VERSION"));
    assert_eq!(info.name.as_str(), "votable-version");
    assert_eq!(info.value.as_str(), "1.99+ (14-Oct-2013)");
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    info.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }

  #[test]
  fn test_info_readwrite_2() {
    let xml = r#"<INFO name="queryParameters" value="25">
  -oc.form=dec
  -out.max=50
  -out.all=2
  -nav=cat:J/ApJ/701/1219&amp;tab:{J/ApJ/701/1219/table4}&amp;key:source=J/ApJ/701/1219&amp;HTTPPRM:&amp;
  -c.eq=J2000
  -c.r=  2
  -c.u=arcmin
  -c.geom=r
  -source=J/ApJ/701/1219/table4
  -order=I
  -out=ID
  -out=RAJ2000
  -out=DEJ2000
  -out=Sep
  -out=Dist
  -out=Bmag
  -out=e_Bmag
  -out=Rmag
  -out=e_Rmag
  -out=Imag
  -out=e_Imag
  -out=z
  -out=Type
  -out=RMag
  -out.all=2
  </INFO>"#;
    // Test read
    let mut info = test_info_read(xml);
    assert_eq!(info.name.as_str(), "queryParameters");
    assert_eq!(info.value.as_str(), "25");
    assert_eq!(info.content.as_ref().map(|s| s.as_str()), Some(r#"
  -oc.form=dec
  -out.max=50
  -out.all=2
  -nav=cat:J/ApJ/701/1219&tab:{J/ApJ/701/1219/table4}&key:source=J/ApJ/701/1219&HTTPPRM:&
  -c.eq=J2000
  -c.r=  2
  -c.u=arcmin
  -c.geom=r
  -source=J/ApJ/701/1219/table4
  -order=I
  -out=ID
  -out=RAJ2000
  -out=DEJ2000
  -out=Sep
  -out=Dist
  -out=Bmag
  -out=e_Bmag
  -out=Rmag
  -out=e_Rmag
  -out=Imag
  -out=e_Imag
  -out=z
  -out=Type
  -out=RMag
  -out.all=2
  "#));
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    info.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }
  
}