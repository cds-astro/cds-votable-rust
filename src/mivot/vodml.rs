//! In a `VODML` block is made to map data models to tables data in a VOTable.
//! In a `VODML` block we find the following sub-blocks:
//! * Mapping block structure
//!     + `REPORT`: telling if the annotation process succeeded or not;
//!     + `MODEL`: containing the URLs of the Data Models used in the mapping;
//!     + `GLOBALS`: containing instances not related to any table data (can be seen as global static instances);
//!     + `TEMPLATES`: containing instances which values, for each row of a table, are filled from the table(s) row fields (basically describe a table)
//! * Data model, or object, structure:
//!     + `INSTANCE`: can be seen as a complex object (or e.g. a row)
//!     + `ATTRIBUTE`: is an atomic value (having possibly a unit, e.g. a single column)
//!     + `COLLECTION`: an array of item of same type
//! * Data reference and identification:
//!     + `REFERENCE`: link to an instance or a collection
//!     + `WHERE`: defines a filter to select rows, or define a join condition
//!     + `JOIN`: defines a join, to populate a collection with instances elements from another collection
//!     + `PRIMARY_KEY`: defines a unique instance identifier
//!     + `FOREIGN_KEY`: link to the primary key of another instance
//! Possibly containing the following attributes:
//! * Model related:
//!     + `name`: name of the model
//!     + `url`: url of the model
//!     + `dmrole`: name in the data model
//!     + `dmtype`: dataype in the data model
//! * Attribute related:
//!     + `value`: constant value of the attribute
//!     + `unit`: unit of the attribute
//!     + `arrayindex`: index of the value of the attribute in case the value or the ref are arrays
//! * VOTable related:
//!     + `ref`: reference pointing to a FIELD ID or a PARAM ID. `ref` are possible in `GLOABLAS`
//!              but they point to a `PARAM`
//!     + `tableref`: reference pointing to a TABLE
//! * Mapping elements:
//!     + `dmref`: reference to the `dmid` of an `INSTANCE` or a `COLLECTION`
//!     + `dmid`: unique identifier of the element
//!     + `sourceref`: reference to the `dmid` of a `COLLECTION` or a `TEMPLATES`
//!     + `primarykey`:
//!     + `foreignkey`:
//! * In this module (and its sub-modules):
//!     + TAG **child of** TAG means direct child;
//!     + TAG **in** TAG mean direct child or sub-child at any depth;
//!     + `dmrole`:
//!         - all childs of a `COLLECTION` have no `dmrole`;
//!         - is mandatory in `INSTANCE` child of `INSTANCE`
//! Look at:
//! * the MIVOT [spec](https://github.com/ivoa-std/ModelInstanceInVot)
//! * the [parser code](https://github.com/ivoa/modelinstanceinvot-code)
//!
//! See also the Astropy API implementation [working group wiki](https://github.com/ivoa/modelinstanceinvot-code/wiki)
//! and [guidline](https://github.com/ivoa/modelinstanceinvot-code/wiki/guideline)
//!
//! and the [Meas data model](https://ivoa.net/documents/Meas/20211019/index.html)
//!

use std::{collections::HashMap, io::Write, str};

use log::warn;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};
use serde_json::Value;

use crate::{
  error::VOTableError,
  utils::{discard_comment, discard_event, is_empty},
  HasSubElements, HasSubElems, QuickXmlReadWrite, VOTableElement,
};

use super::{globals::Globals, model::Model, report::Report, templates::Templates, VodmlVisitor};

/// Structure storing the content of the `VODML` tag.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vodml {
  pub xmlns: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // Sub-elements
  /// Tells the client whether the annotation process succeeded or not.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub report: Option<Report>,
  /// List of used data models.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub models: Vec<Model>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub globals: Option<Globals>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub templates: Vec<Templates>,
}
impl Vodml {
  pub fn new() -> Self {
    Self {
      xmlns: None,
      report: None,
      models: vec![],
      globals: None,
      templates: vec![],
      extra: HashMap::default(),
    }
  }

  // Attributes
  impl_builder_opt_string_attr!(xmlns);
  // Extra attributes
  impl_builder_insert_extra!();
  // Sub-elements
  impl_builder_opt_subelem!(report, Report);
  impl_builder_push!(Model);
  impl_builder_opt_subelem!(globals, Globals);
  impl_builder_push_no_s!(Templates);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_vodml_start(self)?;
    if let Some(report) = self.report.as_mut() {
      report.visit(visitor)?;
    }
    for model in self.models.iter_mut() {
      model.visit(visitor)?;
    }
    if let Some(globals) = self.globals.as_mut() {
      globals.visit(visitor)?;
    }
    for template in self.templates.iter_mut() {
      template.visit(visitor)?;
    }
    visitor.visit_vodml_ended(self)
  }
}

impl VOTableElement for Vodml {
  const TAG: &'static str = "VODML";

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
        "xmlns" => self.set_xmlns_by_ref(val),
        _ => self.insert_extra_str_by_ref(key, val),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(xmlns) = &self.xmlns {
      f("xmlns", xmlns.to_string().as_str());
    }
    for_each_extra_attribute!(self, f);
  }
}

impl HasSubElements for Vodml {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    // Note: Should always be true to be a valid VODML element
    self.report.is_none()
      && self.models.is_empty()
      && self.globals.is_none()
      && self.extra.is_empty()
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Report::TAG_BYTES => set_from_event_start!(self, Report, reader, reader_buff, e),
          Model::TAG_BYTES => push_from_event_start!(self, Model, reader, reader_buff, e),
          Globals::TAG_BYTES => set_from_event_start!(self, Globals, reader, reader_buff, e),
          Templates::TAG_BYTES => push_from_event_start!(self, Templates, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Report::TAG_BYTES => set_from_event_empty!(self, Report, e),
          Model::TAG_BYTES => push_from_event_empty!(self, Model, e),
          Globals::TAG_BYTES => set_from_event_empty!(self, Globals, e),
          Templates::TAG_BYTES => push_from_event_empty!(self, Templates, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          if !self.models.is_empty() {
            return Ok(());
          } else {
            return Err(VOTableError::Custom(
              "Expected a <MODEL> tag, none was found".to_owned(),
            ));
          }
        }
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
    write_elem!(self, report, writer, context);
    write_elem_vec!(self, models, writer, context);
    write_elem!(self, globals, writer, context);
    write_elem_vec!(self, templates, writer, context);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::test::test_error,
    mivot::{test::get_xml, vodml::Vodml},
    tests::test_read,
  };

  #[test]
  fn test_vodml_read() {
    // OK VODMLS
    let xml = get_xml("./resources/mivot/1/test_1_ok_1.1.xml");
    println!("testing 1.1");
    test_read::<Vodml>(&xml);
    let xml = get_xml("./resources/mivot/1/test_1_ok_1.2.xml");
    println!("testing 1.2");
    test_read::<Vodml>(&xml);
    let xml = get_xml("./resources/mivot/1/test_1_ok_1.3.xml");
    println!("testing 1.3");
    test_read::<Vodml>(&xml);
    let xml = get_xml("./resources/mivot/1/test_1_ok_1.4.xml");
    println!("testing 1.4");
    test_read::<Vodml>(&xml);
    let xml = get_xml("./resources/mivot/1/test_1_ok_1.8.xml");
    println!("testing 1.8");
    test_read::<Vodml>(&xml);
    let xml = get_xml("./resources/mivot/1/test_1_ok_1.9.xml");
    println!("testing 1.9");
    test_read::<Vodml>(&xml);

    // KO VODMLS
    let xml = get_xml("./resources/mivot/1/test_1_ko_1.5.xml");
    println!("testing 1.5"); // MODEL required
    test_error::<Vodml>(&xml, false);
    let xml = get_xml("./resources/mivot/1/test_1_ko_1.6.xml");
    println!("testing 1.6"); // MODEL subnode must be first (parser can overlook this and write it correctly later)
    test_read::<Vodml>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/1/test_1_ko_1.7.xml");
    println!("testing 1.7"); // GLOBALS must be after MODEL and before TEMPLATES (parser can overlook this and write it correctly later)
    test_read::<Vodml>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/1/test_1_ko_1.10.xml");
    println!("testing 1.10"); // Only 1 GLOBALS subnode allowed.
    test_read::<Vodml>(&xml); // It is OK, since we produce a WARNING
    let xml = get_xml("./resources/mivot/1/test_1_ko_1.11.xml");
    println!("testing 1.11"); // Includes invalid subnode.
    test_error::<Vodml>(&xml, false);
  }
}
