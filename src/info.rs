//! Struct dedicated to the `INFO` tag.

use std::{collections::HashMap, str};

use paste::paste;
use serde_json::Value;

use super::{error::VOTableError, HasContent, HasContentElem, VOTableElement};

/// Struct corresponding to the `INFO` XML tag.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Info {
  // attributes
  #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub value: String,
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

  // attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_mandatory_string_attr!(name);
  impl_builder_mandatory_string_attr!(value);
  impl_builder_opt_string_attr!(xtype);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(unit);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  // extra attributes
  impl_builder_insert_extra!();
}

impl_has_content!(Info);

impl VOTableElement for Info {
  const TAG: &'static str = "INFO";

  type MarkerType = HasContentElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::new(DEFAULT_VALUE, DEFAULT_VALUE)
      .set_attrs(attrs)
      .and_then(|info| {
        if info.name.as_str() == DEFAULT_VALUE || info.value.as_str() == DEFAULT_VALUE {
          Err(VOTableError::Custom(format!(
            "Attributes 'name' and 'value' are mandatory in tag '{}'",
            Self::TAG
          )))
        } else {
          Ok(info)
        }
      })
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "ID" => self.set_id_by_ref(val),
        "name" => self.set_name_by_ref(val),
        "value" => self.set_value_by_ref(val),
        "xtype" => self.set_xtype_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        "unit" => self.set_unit_by_ref(val),
        "ucd" => self.set_ucd_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
        _ => self.insert_extra_str_by_ref(key, val),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(id) = &self.id {
      f("ID", id.as_str());
    }
    f("name", self.name.as_str());
    f("value", self.value.as_str());
    if let Some(xtype) = &self.xtype {
      f("xtype", xtype.to_string().as_str());
    }
    if let Some(r) = &self.ref_ {
      f("ref", r.as_str());
    }
    if let Some(unit) = &self.unit {
      f("unit", unit.as_str());
    }
    if let Some(ucd) = &self.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
    for_each_extra_attribute!(self, f);
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{events::Event, Reader, Writer};

  use crate::{info::Info, QuickXmlReadWrite, VOTableElement};

  fn test_info_read(xml: &str) -> Info {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == Info::TAG_BYTES => {
          let info = Info::from_event_start(&e)
            .and_then(|info| info.read_content(&mut reader, &mut buff, &()))
            .unwrap();
          return info;
        }
        Event::Empty(ref mut e) if e.local_name() == Info::TAG_BYTES => {
          let info = Info::from_event_empty(&e).unwrap();
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
    assert_eq!(
      info.content.as_ref().map(|s| s.as_str()),
      Some(
        r#"
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
  "#
      )
    );
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    info.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }
}
