
use std::{str, io::{BufRead, Write}};

use quick_xml::{Reader, Writer, events::{Event, BytesStart, attributes::Attributes}};

use paste::paste;

use super::{
  QuickXmlReadWrite,
  error::VOTableError,
};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Min {
  pub value: String,
  pub inclusive: bool, // true by default
}

impl Min {
  
  pub fn new<S: Into<String>>(value: S) -> Self {
    Self {
      value: value.into(),
      inclusive: true
    }
  }

  pub fn set_inclusive(mut self, inclusive: bool) -> Self {
    self.inclusive = inclusive;
    self
  }
}

impl QuickXmlReadWrite for Min {
  const TAG: &'static str = "MIN";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut value: Option<String> = None;
    let mut inclusive = true;
    // Look for attributes
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let val = String::from_utf8(unescaped.as_ref().to_vec()).map_err(VOTableError::FromUtf8)?;
      match attr.key {
        b"value" => value = Some(val),
        b"inclusive" => inclusive = val.parse::<bool>().map_err(VOTableError::ParseBool)?,
        _ => { eprintln!("WARNING: attribute {:?} in {} is ignored", std::str::from_utf8(attr.key), Self::TAG); }
      }
    }
    // Set from found attributes
    if let Some(value) = value {
      Ok(Self::new(value).set_inclusive(inclusive))
    } else {
      Err(VOTableError::Custom(format!("Attributes 'value' is mandatory in tag '{}'", Self::TAG)))
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    _reader: Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    unreachable!()
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    unreachable!()
  }
  
  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("value", self.value.as_str()));
    if !self.inclusive {
      elem_writer = elem_writer.with_attribute(("inclusive", "false"));
    }
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}



#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Max {
  pub value: String,
  pub inclusive: bool, // true by default
}

impl Max {
  
  pub fn new<S: Into<String>>(value: S) -> Self {
    Self {
      value: value.into(),
      inclusive: true
    }
  }

  pub fn set_inclusive(mut self, inclusive: bool) -> Self {
    self.inclusive = inclusive;
    self
  }
}

impl QuickXmlReadWrite for Max {
  const TAG: &'static str = "MAX";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut value: Option<String> = None;
    let mut inclusive = true;
    // Look for attributes
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let val = String::from_utf8(attr.value.as_ref().to_vec()).map_err(VOTableError::FromUtf8)?;
      match attr.key {
        b"value" => value = Some(val),
        b"inclusive" => inclusive = val.parse::<bool>().map_err(VOTableError::ParseBool)?,
        _ => { eprintln!("WARNING: attribute {:?} in {} is ignored", std::str::from_utf8(attr.key), Self::TAG); }
      }
    }
    // Set from found attributes
    if let Some(value) = value {
      Ok(Self::new(value).set_inclusive(inclusive))
    } else {
      Err(VOTableError::Custom(format!("Attributes 'value' is mandatory in tag '{}'", Self::TAG)))
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    _reader: Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    unreachable!()
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    unreachable!()
  }
  
  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("value", self.value.as_str()));
    if !self.inclusive {
      elem_writer = elem_writer.with_attribute(("inclusive", "false"));
    }
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename = "OPTION")]
pub struct Opt {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  pub value: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub opts: Vec<Opt>
}

impl Opt {

  pub fn new<S: Into<String>>(value: S) -> Self {
    Self {
      name: None,
      value: value.into(),
      opts: vec![]
    }
  }
  
  impl_builder_opt_string_attr!(name);
  impl_builder_push!(Opt);
}

impl QuickXmlReadWrite for Opt {
  const TAG: &'static str = "OPTION";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut name: Option<String> = None;
    let mut value: Option<String> = None;
    // Look for attributes
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let val = String::from_utf8(attr.value.as_ref().to_vec()).map_err(VOTableError::FromUtf8)?;
      match attr.key {
        b"name" => name = Some(val),
        b"value" => value = Some(val),
        _ => { eprintln!("WARNING: attribute {:?} in {} is ignored", std::str::from_utf8(attr.key), Self::TAG); }
      }
    }
    // Set from found attributes
    if let Some(value) = value {
      let mut opt = Self::new(value);
      opt.name = name;
      Ok(opt)
    } else {
      Err(VOTableError::Custom(format!("Attributes 'value' is mandatory in tag '{}'", Self::TAG)))
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
          match e.name() {
            Self::TAG_BYTES =>
              self.opts.push(from_event_start!(Opt, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.name() {
            Self::TAG_BYTES => self.opts.push(Self::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(reader),
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
    context: &Self::Context
  ) -> Result<(), VOTableError> {
    if  self.opts.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      write_opt_string_attr!(self, elem_writer, name);
      elem_writer = elem_writer.with_attribute(("value", self.value.as_str()));
      elem_writer.write_empty().map_err(VOTableError::Write)?;
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      push2write_opt_string_attr!(self, tag, name);
      tag.push_attribute(("value", self.value.as_str()));
      writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
      // Write sub-elements
      write_elem_vec!(self, opts, writer, context);
      // Close tag
      writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)?;
    }
    Ok(())
  }
}




#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Values {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_:  Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub null: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_:  Option<String>,
  // sub-elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub min: Option<Min>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max: Option<Max>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub opts: Vec<Opt>
}

impl Values {
  
  pub fn new() -> Self {
    Default::default()
  }

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(type_, type);
  impl_builder_opt_string_attr!(null);
  impl_builder_opt_string_attr!(ref_, ref);

  impl_builder_opt_attr!(min, Min);
  impl_builder_opt_attr!(max, Max);

  impl_builder_push!(Opt);
  
}

impl QuickXmlReadWrite for Values {
  const TAG: &'static str = "VALUES";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut values = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      values = match attr.key {
        b"ID" => values.set_id(value),
        b"type" => values.set_type(value),
        b"null" => values.set_null(value),
        b"ref" => values.set_ref(value),
        _ => {
          eprintln!("WARNING: attribute {:?} in {} is ignored", std::str::from_utf8(attr.key), Self::TAG);
          values
        }
      }
    }
    Ok(values)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
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
            Opt::TAG_BYTES => self.opts.push(from_event_start_by_ref!(Opt, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Min::TAG_BYTES => self.min = Some(Min::from_event_empty(e)?),
            Max::TAG_BYTES => self.max = Some(Max::from_event_empty(e)?),
            Opt::TAG_BYTES => self.opts.push(Opt::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
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
    if self.min.is_none() && self.max.is_none() && self.opts.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      write_opt_string_attr!(self, elem_writer, ID);
      write_opt_string_attr!(self, elem_writer, type_, "type");
      write_opt_string_attr!(self, elem_writer, null);
      write_opt_string_attr!(self, elem_writer, ref_, "ref");
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      push2write_opt_string_attr!(self, tag, ID);
      push2write_opt_string_attr!(self, tag, type_, type);
      push2write_opt_string_attr!(self, tag, null);
      push2write_opt_string_attr!(self, tag, ref_, ref);
      writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
      // Write sub-elements
      write_elem!(self, min, writer, context);
      write_elem!(self, max, writer, context);
      write_elem_vec!(self, opts, writer, context);
      // Close tag
      writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
    }
  }
}