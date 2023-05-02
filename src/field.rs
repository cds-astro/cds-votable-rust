use std::{
  fmt,
  num::ParseIntError,
  collections::HashMap,
  io::{BufRead, Write},
  str::{self, FromStr},
};

use serde_json::Value;
use serde::{
  Serialize, Serializer, 
  de, Deserialize, Deserializer
};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use paste::paste;
use crate::is_empty;

use super::{
  QuickXmlReadWrite,
  desc::Description,
  values::Values,
  link::Link,
  datatype::Datatype,
  error::VOTableError,
};

#[derive(Copy, Clone, Debug)]
pub enum Precision{
  /// Significant figures after the decimal (for the decimal notation)
  F {
    n_decimal: u8 
  },
  /// Number of significant figures (for the scientific notation)
  E {
    n_significant: u8
  },
}
impl Precision {
  pub fn new_dec(n_decimal: u8) -> Self {
    Precision::F{ n_decimal }
  }
  pub fn new_sci(n_significant: u8) -> Self {
    Precision::E { n_significant }
  }
  // Add engineer?!! (Not in the VO standard)
}

impl FromStr for Precision {
  type Err = ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(
      match &s[0..1] {
        "E" => Precision::new_sci(s[1..].parse()?),
        "F" => Precision::new_dec(s[1..].parse()?),
        _ => Precision::new_dec(s.parse()?),
      }
    )
  }
}

impl fmt::Display for Precision {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Precision::F{ n_decimal} => write!(f, "{}", n_decimal),
      Precision::E{ n_significant } => write!(f, "E{}", n_significant),
    }
  }
}
impl Serialize for Precision {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
      S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for Precision {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
  {
    let s = String::deserialize(deserializer)?;
    FromStr::from_str(&s).map_err(de::Error::custom)
  }
}


/// From the VOTable [official document](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html#sec:values),
/// the 'null' attribute in VALUES is rserved to integer types:
/// "This mechanism is only intended for use with integer types; it should not be used for floating point types, which can use NaN instead."
/// "This mechanism for representing null values is required for integer columns in the BINARY serialization [but not in BINARY2]"
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Field {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub datatype: Datatype,           // part of the schema
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub precision: Option<Precision>, // part of the schema
  #[serde(skip_serializing_if = "Option::is_none")]
  pub width: Option<u16>,           // part of the schema
  #[serde(skip_serializing_if = "Option::is_none")]
  pub xtype: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arraysize: Option<String>,    // part of the schema ?
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // sub-elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub values: Option<Values>,       // part of the schema (null attribute or Enum coder)
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub links: Vec<Link>
}
// new_from_schema (may loose information on complex types, like prefix, suffix, ...)
// to_schema (lose information like units, ucds, min, max, ...)

impl Field {
  pub fn new<N: Into<String>>(name: N, datatype: Datatype) -> Self {
    Field {
      id: None,
      name: name.into(),
      datatype,
      unit: None,
      precision: None,
      width: None,
      xtype: None,
      ref_: None,
      ucd: None,
      utype: None,
      arraysize: None,
      extra: Default::default(),
      description: None,
      values: None,
      links: vec![]
    }
  }
  
  /// Look for a NULL value and returns it
  pub fn null_value(&self) -> Option<&String> {
    self.values.as_ref().and_then(|values| values.null.as_ref())
  }

  // attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(unit);
  impl_builder_opt_attr!(precision, Precision);
  impl_builder_opt_attr!(width, u16);
  impl_builder_opt_string_attr!(xtype);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  impl_builder_opt_string_attr!(arraysize);
  // extra attributes
  impl_builder_insert_extra!();
  // sub-elements
  impl_builder_opt_attr!(description, Description);
  impl_builder_opt_attr!(values, Values);
  impl_builder_push!(Link);
  
}

impl QuickXmlReadWrite for Field {
  const TAG: &'static str = "FIELD";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    const NULL_DT: Datatype = Datatype::Logical;
    let mut field = Self::new(NULL, NULL_DT);
    let mut has_datatype = false;
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      field = match attr.key {
        b"ID" => field.set_id(value),
        b"name" => { field.name = value.to_string(); field },
        b"datatype" =>  { 
          field.datatype = value.parse::<Datatype>().map_err(VOTableError::ParseDatatype)?;
          has_datatype = true;
          field
        },
        b"unit" => field.set_unit(value),
        b"precision" if !value.is_empty() => field.set_precision(value.parse::<Precision>().map_err(VOTableError::ParseInt)?),
        b"width" if !value.is_empty() => field.set_width(value.parse().map_err(VOTableError::ParseInt)?),
        b"xtype" => field.set_xtype(value),
        b"ref" => field.set_ref(value),
        b"ucd" => field.set_ucd(value),
        b"utype" => field.set_utype(value),
        b"arraysize" => field.set_arraysize(value),
        _ => field.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    if field.name.as_str() == NULL || !has_datatype {
      Err(VOTableError::Custom(format!("Attributes 'name' and 'datatype' are mandatory in tag '{}'", Self::TAG)))
    } else {
      Ok(field)
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    /*loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES => from_event_start_desc!(self, Description, reader, reader_buff, e),
            Values::TAG_BYTES => self.values = Some(from_event_start!(Values, reader, reader_buff, e)),
            Link::TAG_BYTES => self.links.push(from_event_start!(Link, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Values::TAG_BYTES => self.values = Some(Values::from_event_empty(e)?),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }*/
    self.read_sub_elements_by_ref(&mut reader, reader_buff, _context).map(|()| reader)
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES => from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e),
            Values::TAG_BYTES => self.values = Some(from_event_start_by_ref!(Values, reader, reader_buff, e)),
            Link::TAG_BYTES => self.links.push(from_event_start_by_ref!(Link, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Values::TAG_BYTES => self.values = Some(Values::from_event_empty(e)?),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(()),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, ID);
    tag.push_attribute(("name", self.name.as_str()));
    tag.push_attribute(("datatype", self.datatype.to_string().as_str()));
    push2write_opt_string_attr!(self, tag, unit);
    push2write_opt_tostring_attr!(self, tag, precision);
    push2write_opt_tostring_attr!(self, tag, width);
    push2write_opt_string_attr!(self, tag, xtype);
    push2write_opt_string_attr!(self, tag, ref_, ref);
    push2write_opt_string_attr!(self, tag, ucd);
    push2write_opt_string_attr!(self, tag, utype);
    push2write_opt_string_attr!(self, tag, arraysize);
    push2write_extra!(self, tag);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem!(self, description, writer, context);
    write_elem!(self, values, writer, context);
    write_elem_vec!(self, links, writer, context);
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}
