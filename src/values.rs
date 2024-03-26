//! Module dedicated to the `VALUES` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use log::warn;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use super::{
  error::VOTableError,
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_warn},
  EmptyElem, HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement,
  VOTableVisitor,
};

/// Struct corresponding to the `MIN` XML tag.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Min {
  pub value: String,
  pub inclusive: bool, // true by default
}

impl Min {
  pub fn new<S: Into<String>>(value: S) -> Self {
    Self {
      value: value.into(),
      inclusive: true,
    }
  }

  impl_builder_mandatory_string_attr!(value);
  impl_builder_mandatory_attr!(inclusive, bool);
}

impl VOTableElement for Min {
  const TAG: &'static str = "MIN";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::new(DEFAULT_VALUE).set_attrs(attrs).and_then(|min| {
      if min.value.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'value' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(min)
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
        "value" => self.set_value_by_ref(val),
        "inclusive" => {
          self.set_inclusive_by_ref(val.as_ref().parse().map_err(VOTableError::ParseBool)?)
        }
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("value", self.value.as_str());
    if !self.inclusive {
      f("inclusive", self.inclusive.to_string().as_str());
    }
  }
}

/// Struct corresponding to the `MAX` XML tag.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Max {
  pub value: String,
  pub inclusive: bool, // true by default
}

impl Max {
  pub fn new<S: Into<String>>(value: S) -> Self {
    Self {
      value: value.into(),
      inclusive: true,
    }
  }

  impl_builder_mandatory_string_attr!(value);
  impl_builder_mandatory_attr!(inclusive, bool);
}

impl VOTableElement for Max {
  const TAG: &'static str = "MAX";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::new(DEFAULT_VALUE).set_attrs(attrs).and_then(|max| {
      if max.value.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'value' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(max)
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
        "value" => self.set_value_by_ref(val),
        "inclusive" => self.inclusive = val.as_ref().parse().map_err(VOTableError::ParseBool)?,
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("value", self.value.as_str());
    if !self.inclusive {
      f("inclusive", self.inclusive.to_string().as_str());
    }
  }
}

/// Struct corresponding to the `OPTION` XML tag.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "OPTION")]
pub struct Opt {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  pub value: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub opts: Vec<Opt>,
}

impl Opt {
  pub fn new<S: Into<String>>(value: S) -> Self {
    Self {
      name: None,
      value: value.into(),
      opts: vec![],
    }
  }

  impl_builder_opt_string_attr!(name);
  impl_builder_mandatory_string_attr!(value);
  impl_builder_push!(Opt);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_values_opt_start(self)?;
    for opt in &mut self.opts {
      opt.visit(visitor)?;
    }
    visitor.visit_values_opt_ended(self)
  }
}

impl VOTableElement for Opt {
  const TAG: &'static str = "OPTION";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::new(DEFAULT_VALUE).set_attrs(attrs).and_then(|opt| {
      if opt.value.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'value' not found in tag '{}'",
          Self::TAG
        )))
      } else {
        Ok(opt)
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
        "name" => self.set_name_by_ref(val),
        "value" => self.set_value_by_ref(val),
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(name) = &self.name {
      f("name", name.as_str());
    }
    f("value", self.value.as_str());
  }
}

impl HasSubElements for Opt {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.opts.is_empty()
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
        Event::Start(ref e) => match e.name() {
          Self::TAG_BYTES => push_from_event_start!(self, Opt, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.name() {
          Self::TAG_BYTES => push_from_event_empty!(self, Opt, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.name() == Self::TAG_BYTES => return Ok(()),
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec!(self, opts, writer, context);
    Ok(())
  }
}

/// Struct corresponding to the `VALUES` XML tag.
#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Values {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub null: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  // sub-elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub min: Option<Min>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max: Option<Max>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub opts: Vec<Opt>,
}

impl Values {
  pub fn new() -> Self {
    Default::default()
  }

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(type_, type);
  impl_builder_opt_string_attr!(null);
  impl_builder_opt_string_attr!(ref_, ref);

  impl_builder_opt_subelem!(min, Min);
  impl_builder_opt_subelem!(max, Max);

  impl_builder_push!(Opt);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_values_start(self)?;
    if let Some(min) = &mut self.min {
      visitor.visit_values_min(min)?;
    }
    if let Some(max) = &mut self.max {
      visitor.visit_values_max(max)?;
    }
    for opt in &mut self.opts {
      opt.visit(visitor)?;
    }
    visitor.visit_values_ended(self)
  }
}

impl VOTableElement for Values {
  const TAG: &'static str = "VALUES";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new().set_attrs(attrs)
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
        "type" => self.set_type_by_ref(val),
        "null" => self.set_null_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        _ => unexpected_attr_warn(key, Self::TAG),
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
    if let Some(type_) = &self.type_ {
      f("type", type_.as_str());
    }
    if let Some(null) = &self.null {
      f("null", null.as_str());
    }
    if let Some(ref_) = &self.ref_ {
      f("ref", ref_.as_str());
    }
  }
}

impl HasSubElements for Values {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.min.is_none() && self.max.is_none() && self.opts.is_empty()
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
        Event::Start(ref e) => match e.local_name() {
          Opt::TAG_BYTES => push_from_event_start!(self, Opt, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Min::TAG_BYTES => set_from_event_empty!(self, Min, e),
          Max::TAG_BYTES => set_from_event_empty!(self, Max, e),
          Opt::TAG_BYTES => push_from_event_empty!(self, Opt, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(()),
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem!(self, min, writer, context);
    write_elem!(self, max, writer, context);
    write_elem_vec!(self, opts, writer, context);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    tests::{test_read, test_writer},
    values::Values,
  };

  #[test]
  fn test_values_read_write() {
    let xml = r#"<VALUES ID="RAdomain" null="NaN"><MIN value="0"/><MAX value="360" inclusive="false"/></VALUES>"#; // Test read
    let value = test_read::<Values>(xml);
    assert_eq!(value.id.as_ref().map(|s| s.as_str()), Some("RAdomain"));
    assert_eq!(value.min.as_ref().unwrap().value, "0");
    assert_eq!(value.max.as_ref().unwrap().value, "360");
    assert_eq!(value.max.as_ref().unwrap().inclusive, false);
    assert_eq!(value.null, Some("NaN".to_string()));

    // Test write
    test_writer(value, xml)
  }

  #[test]
  fn test_values_read_write2() {
    let xml = r#"<VALUES ref="RAdomain"/>"#; // Test read
    let value = test_read::<Values>(xml);
    assert_eq!(value.ref_, Some("RAdomain".to_string()));
    // Test write
    test_writer(value, xml)
  }
}
