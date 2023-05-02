use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use serde_json::Value;

use super::{
  QuickXmlReadWrite,
  field::{Precision, Field},
  desc::Description,
  values::Values,
  link::Link,
  datatype::Datatype,
  error::VOTableError,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Param {
  #[serde(flatten)]
  field: Field,
  value: String,
}

impl Param {
  pub fn new<N: Into<String>, V: Into<String>>(name: N, datatype: Datatype, value: V) -> Self {
    Param {
      field: Field::new(name, datatype),
      value: value.into(),
    }
  }

  // copy/paste + modified from `cargo expand field`

  pub fn set_id<I: Into<String>>(mut self, id: I) -> Self {
    self.field.id = Some(id.into());
    self
  }
  pub fn set_unit<I: Into<String>>(mut self, unit: I) -> Self {
    self.field.unit = Some(unit.into());
    self
  }
  pub fn set_precision(mut self, precision: Precision) -> Self {
    self.field.precision = Some(precision);
    self
  }
  pub fn set_width(mut self, width: u16) -> Self {
    self.field.width = Some(width);
    self
  }
  pub fn set_xtype<I: Into<String>>(mut self, xtype: I) -> Self {
    self.field.xtype = Some(xtype.into());
    self
  }
  pub fn set_ref<I: Into<String>>(mut self, ref_: I) -> Self {
    self.field.ref_ = Some(ref_.into());
    self
  }
  pub fn set_ucd<I: Into<String>>(mut self, ucd: I) -> Self {
    self.field.ucd = Some(ucd.into());
    self
  }
  pub fn set_utype<I: Into<String>>(mut self, utype: I) -> Self {
    self.field.utype = Some(utype.into());
    self
  }
  pub fn set_arraysize<I: Into<String>>(mut self, arraysize: I) -> Self {
    self.field.arraysize = Some(arraysize.into());
    self
  }
  pub fn insert_extra<S: Into<String>>(mut self, key: S, value: Value) -> Self {
    self.field.extra.insert(key.into(), value);
    self
  }
  pub fn set_description(mut self, description: Description) -> Self {
    self.field.description = Some(description);
    self
  }
  pub fn set_values(mut self, values: Values) -> Self {
    self.field.values = Some(values);
    self
  }
  pub fn push_link(mut self, link: Link) -> Self {
    self.field.links.push(link);
    self
  }
}


impl QuickXmlReadWrite for Param {
  const TAG: &'static str = "PARAM";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    const NULL_DT: Datatype = Datatype::Logical;
    let mut param = Self::new(NULL, NULL_DT, NULL);
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      param = match attr.key {
        b"ID" => param.set_id(value),
        b"name" => {
          param.field.name = value.to_string();
          param
        }
        b"datatype" => {
          param.field.datatype = value.parse::<Datatype>().map_err(VOTableError::ParseDatatype)?;
          param
        }
        b"unit" => param.set_utype(value),
        b"precision" if !value.is_empty() => param.set_precision(value.parse::<Precision>().map_err(VOTableError::ParseInt)?),
        b"width"  if !value.is_empty() => param.set_width(value.parse().map_err(VOTableError::ParseInt)?),
        b"xtype" => param.set_xtype(value),
        b"ref" => param.set_ref(value),
        b"ucd" => param.set_ucd(value),
        b"utype" => param.set_utype(value),
        b"arraysize" => param.set_arraysize(value),
        b"value" => {
          param.value = value.to_string();
          param
        }
        _ => param.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    if param.field.name.as_str() == NULL || param.field.datatype == NULL_DT || param.value.as_str() == NULL {
      Err(VOTableError::Custom(format!("Attributes 'name', 'datatype' and 'value' are mandatory in tag '{}'", Self::TAG)))
    } else {
      Ok(param)
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES => self.field.description = Some(from_event_start!(Description, reader, reader_buff, e)),
            Values::TAG_BYTES => self.field.values = Some(from_event_start!(Values, reader, reader_buff, e)),
            Link::TAG_BYTES => self.field.links.push(from_event_start!(Link, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Values::TAG_BYTES => self.field.values = Some(Values::from_event_empty(e)?),
            Link::TAG_BYTES => self.field.links.push(Link::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    todo!()
  }

  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    // copy/paste + modified from `cargo expand field`
    
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    if let Some(id) = &self.field.id {
      tag.push_attribute(("ID", id.as_str()));
    };
    tag.push_attribute(("name", self.field.name.as_str()));
    tag.push_attribute(("datatype", self.field.datatype.to_string().as_str()));
    tag.push_attribute(("value", self.value.as_str()));
    if let Some(unit) = &self.field.unit {
      tag.push_attribute(("unit", unit.as_str()));
    };
    if let Some(precision) = &self.field.precision {
      tag.push_attribute(("precision", precision.to_string().as_str()));
    };
    if let Some(width) = &self.field.width {
      tag.push_attribute(("width", width.to_string().as_str()));
    };
    if let Some(xtype) = &self.field.xtype {
      tag.push_attribute(("xtype", xtype.as_str()));
    };
    if let Some(ref_) = &self.field.ref_ {
      tag.push_attribute(("ref", ref_.as_str()));
    };
    if let Some(ucd) = &self.field.ucd {
      tag.push_attribute(("ucd", ucd.as_str()));
    };
    if let Some(utype) = &self.field.utype {
      tag.push_attribute(("utype", utype.as_str()));
    };
    if let Some(arraysize) = &self.field.arraysize {
      tag.push_attribute(("arraysize", arraysize.as_str()));
    };
    for (key, val) in &self.field.extra {
      tag.push_attribute((key.as_str(), val.to_string().as_str()));
    }
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    if let Some(elem) = &mut self.field.description {
      elem.write(writer, &())?;
    };
    if let Some(elem) = &mut self.field.values {
      elem.write(writer, &())?;
    };
    for elem in &mut self.field.links {
      elem.write(writer, &())?;
    }
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}