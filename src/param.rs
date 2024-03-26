//! Struct dedicated to the `PARAM` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use log::warn;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};
use serde_json::Value;

use super::{
  datatype::Datatype,
  desc::Description,
  error::VOTableError,
  field::{ArraySize, Field, Precision},
  link::Link,
  utils::{discard_comment, discard_event},
  values::Values,
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
};

/// Struct corresponding to the `PARAM` XML tag.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Param {
  #[serde(flatten)]
  pub field: Field,
  pub value: String,
}

impl Param {
  pub fn new<N: Into<String>, V: Into<String>>(name: N, datatype: Datatype, value: V) -> Self {
    Param {
      field: Field::new(name, datatype),
      value: value.into(),
    }
  }

  // attributes
  impl_builder_opt_string_attr_delegated!(id, field);
  impl_builder_mandatory_string_attr_delegated!(name, field);
  impl_builder_mandatory_attr_delegated!(datatype, Datatype, field);
  impl_builder_opt_string_attr_delegated!(unit, field);
  impl_builder_opt_attr_delegated!(precision, Precision, field);
  impl_builder_opt_attr_delegated!(width, u16, field);
  impl_builder_opt_string_attr_delegated!(xtype, field);
  impl_builder_opt_string_attr_delegated!(ref_, ref, field);
  impl_builder_opt_string_attr_delegated!(ucd, field);
  impl_builder_opt_string_attr_delegated!(utype, field);
  impl_builder_opt_attr_delegated!(arraysize, ArraySize, field);
  impl_builder_mandatory_string_attr!(value);
  // extra attributes
  impl_builder_insert_extra_delegated!(field);
  // sub-elements
  impl_builder_opt_subelem_delegated!(description, Description, field);
  impl_builder_opt_subelem_delegated!(values, Values, field);
  impl_builder_push_delegated!(Link, field);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_param_start(self)?;
    if let Some(description) = &mut self.field.description {
      visitor.visit_description(description)?;
    }
    if let Some(values) = &mut self.field.values {
      values.visit(visitor)?;
    }
    for l in &mut self.field.links {
      visitor.visit_link(l)?;
    }
    visitor.visit_param_ended(self)
  }
}

impl VOTableElement for Param {
  const TAG: &'static str = "PARAM";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    const DEFAULT_DT: Datatype = Datatype::Logical;
    let mut name_found = false;
    let mut dt_found = false;
    let mut val_found = false;
    Self::new(DEFAULT_VALUE, DEFAULT_DT, DEFAULT_VALUE)
      .set_attrs(attrs.map(|(k, v)| {
        match k.as_ref() {
          "name" => name_found = true,
          "datatype" => dt_found = true,
          "value" => val_found = true,
          _ => {}
        };
        (k, v)
      }))
      .and_then(|param| {
        if name_found && dt_found && val_found {
          Ok(param)
        } else {
          Err(VOTableError::Custom(format!(
            "Attributes 'name', 'datatype' and 'value' are mandatory in tag '{}'",
            Self::TAG
          )))
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
        "datatype" => {
          self.set_datatype_by_ref(val.as_ref().parse().map_err(VOTableError::ParseDatatype)?)
        }
        "unit" => self.set_unit_by_ref(val),
        "precision" => {
          if val.as_ref().is_empty() {
            warn!(
              "Emtpy 'precision' attribute in tag {}: attribute ignored",
              Self::TAG
            )
          } else {
            self.set_precision_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?)
          }
        }
        "width" => self.set_width_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?),
        "xtype" => self.set_xtype_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        "ucd" => self.set_ucd_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
        "arraysize" => {
          self.set_arraysize_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?)
        }
        "value" => self.set_value_by_ref(val),
        _ => self.insert_extra_str_by_ref(key, val),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(id) = &self.field.id {
      f("ID", id.as_str());
    }
    f("name", self.field.name.as_str());
    f("datatype", self.field.datatype.to_string().as_str());
    f("value", self.value.as_str());
    if let Some(arraysize) = &self.field.arraysize {
      f("arraysize", arraysize.to_string().as_str());
    }
    if let Some(width) = &self.field.width {
      f("width", width.to_string().as_str());
    }
    if let Some(precision) = &self.field.precision {
      f("precision", precision.to_string().as_str());
    }
    if let Some(unit) = &self.field.unit {
      f("unit", unit.as_str());
    }
    if let Some(ucd) = &self.field.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.field.utype {
      f("utype", utype.as_str());
    }
    if let Some(xtype) = &self.field.xtype {
      f("xtype", xtype.as_str());
    }
    if let Some(ref_) = &self.field.ref_ {
      f("ref", ref_.as_str());
    }
    for_each_extra_attribute_delegated!(self, field, f);
  }
}

impl HasSubElements for Param {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.field.has_no_sub_elements()
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
          Description::TAG_BYTES => {
            set_from_event_start!(self, Description, reader, reader_buff, e)
          }
          Values::TAG_BYTES => set_from_event_start!(self, Values, reader, reader_buff, e),
          Link::TAG_BYTES => push_from_event_start!(self, Link, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Values::TAG_BYTES => set_from_event_empty!(self, Values, e),
          Link::TAG_BYTES => push_from_event_empty!(self, Link, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(()),
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
    self.field.write_sub_elements_by_ref(writer, context)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    param::Param,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_params_read_write() {
    let xml = r#"<PARAM name="Freq" datatype="float" value="352" ucd="em.freq" utype="MHz"/>"#; // Test read
    let param = test_read::<Param>(xml);
    //Other parameters like name datatype etc... depend on Field reading, see Field read_write_test
    assert_eq!(param.value.as_str(), "352");
    // Test write
    test_writer(param, xml)
  }
}
