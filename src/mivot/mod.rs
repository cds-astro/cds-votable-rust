//! Module dedicated to the `VODML` tag.

use std::error::Error;

pub mod attribute;
pub mod globals;
pub mod join;
pub mod model;
pub mod report;
pub mod templates;
pub mod visitors;
pub mod vodml;

pub use self::{
  attribute::{AttributeChildOfCollection as AttributeC, AttributeChildOfInstance as AttributeI},
  globals::{
    collection::{
      instance::Instance as InstanceGCI, reference::Reference as RefGCR, Collection as CollectionGC,
    },
    instance::{
      collection::{collection::Collection as CollectionGICC, Collection as CollectionGIC},
      instance::Instance as InstanceGII,
      primary_key::PrimaryKeyStatic as PrimaryKeyS,
      reference::Reference as RefGIR,
      Instance as InstanceGI,
    },
    Globals,
  },
  join::{r#where::Where as WhereJ, Join},
  model::Model,
  report::Report,
  templates::{
    instance::{
      collection::{
        collection::Collection as CollectionTICC, reference::ReferenceDyn as RefDynTICR,
        Collection as CollectionTIC,
      },
      instance::Instance as InstanceTII,
      primary_key::PrimaryKeyDyn,
      reference::{foreign_key::ForeignKey, ReferenceDyn as RefDynTIR},
      Instance as InstanceTI,
    },
    r#where::Where as WhereT,
    Templates,
  },
  vodml::Vodml,
};

pub trait VodmlVisitor {
  type E: Error;

  /// Give access to VODML attributes, the VODML sub-elements
  /// are then automatically visited.
  fn visit_vodml_start(&mut self, vodml: &mut Vodml) -> Result<(), Self::E>; // start/ended
  fn visit_vodml_ended(&mut self, vodml: &mut Vodml) -> Result<(), Self::E>; // start/ended

  fn visit_report(&mut self, report: &mut Report) -> Result<(), Self::E>;

  fn visit_model(&mut self, model: &mut Model) -> Result<(), Self::E>;

  fn visit_globals_start(&mut self, globals: &mut Globals) -> Result<(), Self::E>;
  fn visit_globals_ended(&mut self, globals: &mut Globals) -> Result<(), Self::E>;

  fn visit_templates_start(&mut self, templates: &mut Templates) -> Result<(), Self::E>;
  fn visit_templates_ended(&mut self, templates: &mut Templates) -> Result<(), Self::E>;

  // Globals
  fn visit_instance_childof_globals_start(
    &mut self,
    instance: &mut InstanceGI,
  ) -> Result<(), Self::E>;
  fn visit_instance_childof_globals_ended(
    &mut self,
    instance: &mut InstanceGI,
  ) -> Result<(), Self::E>;

  fn visit_instance_childof_instance_in_globals_start(
    &mut self,
    instance: &mut InstanceGII,
  ) -> Result<(), Self::E>;
  fn visit_instance_childof_instance_in_globals_ended(
    &mut self,
    instance: &mut InstanceGII,
  ) -> Result<(), Self::E>;

  fn visit_instance_childof_collection_in_globals_start(
    &mut self,
    instance: &mut InstanceGCI,
  ) -> Result<(), Self::E>;
  fn visit_instance_childof_collection_in_globals_ended(
    &mut self,
    instance: &mut InstanceGCI,
  ) -> Result<(), Self::E>;

  fn visit_collection_childof_instance_in_globals_start(
    &mut self,
    instance: &mut CollectionGIC,
  ) -> Result<(), Self::E>;
  fn visit_collection_childof_instance_in_globals_ended(
    &mut self,
    instance: &mut CollectionGIC,
  ) -> Result<(), Self::E>;

  fn visit_collection_childof_globals_start(
    &mut self,
    collection: &mut CollectionGC,
  ) -> Result<(), Self::E>;
  fn visit_collection_childof_globals_ended(
    &mut self,
    collection: &mut CollectionGC,
  ) -> Result<(), Self::E>;

  fn visit_collection_childof_collection_in_globals_start(
    &mut self,
    collection: &mut CollectionGICC,
  ) -> Result<(), Self::E>;
  fn visit_collection_childof_collection_in_globals_ended(
    &mut self,
    collection: &mut CollectionGICC,
  ) -> Result<(), Self::E>;

  // Templates
  fn visit_instance_childof_templates_start(
    &mut self,
    instance: &mut InstanceTI,
  ) -> Result<(), Self::E>;
  fn visit_instance_childof_templates_ended(
    &mut self,
    instance: &mut InstanceTI,
  ) -> Result<(), Self::E>;

  fn visit_instance_childof_instance_in_templates_start(
    &mut self,
    instance: &mut InstanceTII,
  ) -> Result<(), Self::E>;
  fn visit_instance_childof_instance_in_templates_ended(
    &mut self,
    instance: &mut InstanceTII,
  ) -> Result<(), Self::E>;

  fn visit_collection_childof_instance_in_templates_start(
    &mut self,
    collection: &mut CollectionTIC,
  ) -> Result<(), Self::E>;
  fn visit_collection_childof_instance_in_templates_ended(
    &mut self,
    collection: &mut CollectionTIC,
  ) -> Result<(), Self::E>;

  fn visit_collection_childof_collection_in_templates_start(
    &mut self,
    collection: &mut CollectionTICC,
  ) -> Result<(), Self::E>;
  fn visit_collection_childof_collection_in_templates_ended(
    &mut self,
    collection: &mut CollectionTICC,
  ) -> Result<(), Self::E>;

  fn visit_reference_dynamic_childof_instance_in_templates_start(
    &mut self,
    instance: &mut RefDynTIR,
  ) -> Result<(), Self::E>;
  fn visit_reference_dynamic_childof_instance_in_templates_ended(
    &mut self,
    instance: &mut RefDynTIR,
  ) -> Result<(), Self::E>;

  fn visit_reference_dynamic_childof_collection_in_templates_start(
    &mut self,
    instance: &mut RefDynTICR,
  ) -> Result<(), Self::E>;
  fn visit_reference_dynamic_childof_collection_in_templates_ended(
    &mut self,
    instance: &mut RefDynTICR,
  ) -> Result<(), Self::E>;

  // Common

  fn visit_attribute_childof_instance(&mut self, attr: &mut AttributeI) -> Result<(), Self::E>;
  fn visit_attribute_childof_collection(&mut self, attr: &mut AttributeC) -> Result<(), Self::E>;

  /// Either in globals or in templates
  fn visit_reference_static_childof_instance(
    &mut self,
    reference: &mut RefGIR,
  ) -> Result<(), Self::E>;
  /// Either in globals or in templates
  fn visit_reference_static_childof_collection(
    &mut self,
    reference: &mut RefGCR,
  ) -> Result<(), Self::E>;

  fn visit_primarykey_static(&mut self, pk: &mut PrimaryKeyS) -> Result<(), Self::E>;
  fn visit_primarykey_dynamic(&mut self, pk: &mut PrimaryKeyDyn) -> Result<(), Self::E>;

  fn visit_foreign_key(&mut self, pk: &mut ForeignKey) -> Result<(), Self::E>;

  fn visit_join_start(&mut self, join: &mut join::Join) -> Result<(), Self::E>;
  fn visit_join_ended(&mut self, join: &mut join::Join) -> Result<(), Self::E>;

  fn visit_where_childof_join(&mut self, r#where: &mut WhereJ) -> Result<(), Self::E>;
  fn visit_where_childof_templates(&mut self, r#where: &mut WhereT) -> Result<(), Self::E>;
}

#[cfg(test)]
mod test {
  use std::{
    fs::File,
    io::{Cursor, Read},
  };

  use quick_xml::{events::Event, Reader};

  use crate::{QuickXmlReadWrite, VOTableElement};

  pub(crate) fn test_error<X>(xml: &str, special_cond: bool)
  where
    X: VOTableElement + QuickXmlReadWrite<<X as VOTableElement>::MarkerType, Context = ()>,
  {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == X::TAG_BYTES => {
          if !special_cond {
            let res = X::from_event_start(e)
              .and_then(|info| info.read_content(&mut reader, &mut buff, &()));
            assert!(res.is_err());
          } else {
            assert!(X::from_event_start(e).is_err())
          }
          break;
        }
        Event::Empty(ref mut e) if e.local_name() == X::TAG_BYTES => {
          if special_cond {
            let info = X::from_event_empty(e);
            assert!(info.is_err());
          } else {
            assert!(X::from_event_start(e).is_err())
          };
          break;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        Event::Comment(_) => (),
        Event::DocType(_) => (),
        Event::Decl(_) => (),
        _ => {
          println!("{:?}", event);
          unreachable!()
        }
      }
    }
  }

  pub(crate) fn get_xml(path: &str) -> String {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut xml = String::new();
    file
      .read_to_string(&mut xml)
      .expect("Unable to read the file");
    xml.replace(&['\n', '\t', '\r'][..], "")
  }
}
