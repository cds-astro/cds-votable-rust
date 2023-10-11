extern crate core;

use std::error::Error;

use crate::error::VOTableError;
use crate::mivot::templates::instance::primary_key::PrimaryKeyDyn;
use crate::mivot::templates::instance::reference::foreign_key::ForeignKey;

#[macro_use]
pub mod macros;

pub mod attribute;
pub mod globals;
pub mod join;
pub mod model;
pub mod report;
pub mod templates;
pub mod vodml;

use self::{
  vodml::Vodml, report::Report, templates::Templates, model::Model, globals::{
    Globals,
    instance::primary_key::PrimaryKeyStatic,
  },
           attribute::{AttributeChildOfCollection, AttributeChildOfInstance}
};

/*
pub trait ElemType {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError>;
}
pub trait ElemImpl<T: ElemType> {
  fn push_elems(&mut self, elem: T);
}

pub trait InstanceType {
  fn push_pk(&mut self, pk: PrimaryKeyDyn);
}

pub trait CollectionType {
  fn push_to_checker(&mut self, str: String);
  fn check_elems(&mut self) -> bool;
}
*/


pub trait VodmlVisitor {
  type E: Error;

  /// Give access to VODML attributes, the VODML sub-elements
  /// are then automatically visited.
  fn visit_vodml(&mut self, vodml: &mut Vodml) -> Result<(), Self::E>;
  
  fn visit_report(&mut self, report: &mut Report) -> Result<(), Self::E>;
  
  fn visit_model(&mut self, model: &mut Model) -> Result<(), Self::E>;
  
  fn visit_globals(&mut self, globals: &mut Globals) -> Result<(), Self::E>;
  
  fn visit_templates(&mut self, templates: &mut Templates) -> Result<(), Self::E>;
  
  // Globals
  fn visit_instance_childof_globals(&mut self, instance: &mut globals::instance::Instance) -> Result<(), Self::E>;
  fn visit_instance_childof_instance_in_globals(&mut self, instance: &mut globals::instance::instance::Instance) -> Result<(), Self::E>;
  fn visit_instance_childof_collection_in_globals(&mut self, instance: &mut globals::collection::instance::Instance) -> Result<(), Self::E>;

  fn visit_collection_childof_instance_in_globals(&mut self, instance: &mut globals::instance::collection::Collection) -> Result<(), Self::E>;
  fn visit_collection_childof_globals(&mut self, collection: &mut globals::collection::Collection) -> Result<(), Self::E>;
  fn visit_collection_childof_collection_in_globals(&mut self, collection: &mut globals::instance::collection::collection::Collection) -> Result<(), Self::E>;
  
  // Templates
  fn visit_instance_childof_templates(&mut self, instance: &mut templates::instance::Instance) -> Result<(), Self::E>;
  fn visit_instance_childof_instance_in_templates(&mut self, instance: &mut templates::instance::instance::Instance) -> Result<(), Self::E>;

  fn visit_collection_childof_instance_in_templates(&mut self, collection: &mut templates::instance::collection::Collection) -> Result<(), Self::E>;
  fn visit_collection_childof_collection_in_templates(&mut self, collection: &mut templates::instance::collection::collection::Collection) -> Result<(), Self::E>;

  fn visit_reference_dynamic_childof_instance_in_templates(&mut self, instance: &mut templates::instance::reference::ReferenceDyn) -> Result<(), Self::E>;
  fn visit_reference_dynamic_childof_collection_in_templates(&mut self, instance: &mut templates::instance::collection::reference::ReferenceDyn) -> Result<(), Self::E>;
  
  // Common 

  fn visit_attribute_childof_instance(&mut self, attr: &mut AttributeChildOfInstance) -> Result<(), Self::E>;
  fn visit_attribute_childof_collection(&mut self, attr: &mut AttributeChildOfCollection) -> Result<(), Self::E>;
  
  /// Either in globals or in templates
  fn visit_reference_static_childof_instance(&mut self, reference: &mut globals::instance::reference::Reference) -> Result<(), Self::E>;
  /// Either in globals or in templates
  fn visit_reference_static_childof_collection(&mut self, reference: &mut globals::collection::reference::Reference) -> Result<(), Self::E>;
  
  fn visit_primarykey_static(&mut self, pk: &mut PrimaryKeyStatic) -> Result<(), Self::E>;
  fn visit_primarykey_dynamic(&mut self, pk: &mut PrimaryKeyDyn) -> Result<(), Self::E>;

  fn visit_foreign_key(&mut self, pk: &mut ForeignKey) -> Result<(), Self::E>;
  
  fn visit_join(&mut self, join: &mut join::Join)-> Result<(), Self::E>;
  fn visit_where_childof_join(&mut self, r#where: &mut join::r#where::Where)-> Result<(), Self::E>;
  fn visit_where_childof_templates(&mut self, r#where: &mut templates::r#where::Where)-> Result<(), Self::E>;

}


pub(crate) fn value_checker(value: &str, attribute: &str) -> Result<(), VOTableError> {
  if value.is_empty() {
    Err(VOTableError::Custom(format!(
      "If attribute {} is present it cannot be empty",
      attribute
    )))
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::QuickXmlReadWrite;
  use quick_xml::{events::Event, Reader};
  use std::{
    fs::File,
    io::{Cursor, Read},
  };

  pub(crate) fn test_error<X: QuickXmlReadWrite<Context = ()>>(xml: &str, special_cond: bool) {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == X::TAG_BYTES => {
          if !special_cond {
            let mut info = X::from_attributes(e.attributes()).unwrap();
            assert!(info
              .read_sub_elements_and_clean(reader.clone(), &mut buff, &())
              .is_err());
          } else {
            assert!(X::from_attributes(e.attributes()).is_err())
          }
          break;
        }
        Event::Empty(ref mut e) if e.local_name() == X::TAG_BYTES => {
          if special_cond {
            let mut info = X::from_attributes(e.attributes()).unwrap();
            assert!(info
              .read_sub_elements_and_clean(reader.clone(), &mut buff, &())
              .is_err());
          } else {
            assert!(X::from_attributes(e.attributes()).is_err())
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
