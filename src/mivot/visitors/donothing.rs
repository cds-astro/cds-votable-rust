use crate::mivot::*;
use std::marker::PhantomData;

pub struct DoNothing<E: Error>(PhantomData<E>);

impl<E: Error> DoNothing<E> {
  pub fn new() -> Self {
    Self(PhantomData)
  }
}

impl<E: Error> Default for DoNothing<E> {
  fn default() -> Self {
    Self::new()
  }
}

impl<E: Error> VodmlVisitor for DoNothing<E> {
  type E = E;

  fn visit_vodml_start(&mut self, _: &mut Vodml) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_vodml_ended(&mut self, _: &mut Vodml) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_report(&mut self, _: &mut Report) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_model(&mut self, _: &mut Model) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_globals_start(&mut self, _: &mut Globals) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_globals_ended(&mut self, _: &mut Globals) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_templates_start(&mut self, _: &mut Templates) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_templates_ended(&mut self, _: &mut Templates) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_instance_childof_globals_start(&mut self, _: &mut InstanceGI) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_instance_childof_globals_ended(&mut self, _: &mut InstanceGI) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_instance_childof_instance_in_globals_start(
    &mut self,
    _: &mut InstanceGII,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_instance_childof_instance_in_globals_ended(
    &mut self,
    _: &mut InstanceGII,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_instance_childof_collection_in_globals_start(
    &mut self,
    _: &mut InstanceGCI,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_instance_childof_collection_in_globals_ended(
    &mut self,
    _: &mut InstanceGCI,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_collection_childof_instance_in_globals_start(
    &mut self,
    _: &mut CollectionGIC,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_collection_childof_instance_in_globals_ended(
    &mut self,
    _: &mut CollectionGIC,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_collection_childof_globals_start(
    &mut self,
    _: &mut CollectionGC,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_collection_childof_globals_ended(
    &mut self,
    _: &mut CollectionGC,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_collection_childof_collection_in_globals_start(
    &mut self,
    _: &mut CollectionGICC,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_collection_childof_collection_in_globals_ended(
    &mut self,
    _: &mut CollectionGICC,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_instance_childof_templates_start(&mut self, _: &mut InstanceTI) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_instance_childof_templates_ended(&mut self, _: &mut InstanceTI) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_instance_childof_instance_in_templates_start(
    &mut self,
    _: &mut InstanceTII,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_instance_childof_instance_in_templates_ended(
    &mut self,
    _: &mut InstanceTII,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_collection_childof_instance_in_templates_start(
    &mut self,
    _: &mut CollectionTIC,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_collection_childof_instance_in_templates_ended(
    &mut self,
    _: &mut CollectionTIC,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_collection_childof_collection_in_templates_start(
    &mut self,
    _: &mut CollectionTICC,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_collection_childof_collection_in_templates_ended(
    &mut self,
    _: &mut CollectionTICC,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_reference_dynamic_childof_instance_in_templates_start(
    &mut self,
    _: &mut RefDynTIR,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_reference_dynamic_childof_instance_in_templates_ended(
    &mut self,
    _: &mut RefDynTIR,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_reference_dynamic_childof_collection_in_templates_start(
    &mut self,
    _: &mut RefDynTICR,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_reference_dynamic_childof_collection_in_templates_ended(
    &mut self,
    _: &mut RefDynTICR,
  ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_attribute_childof_instance(&mut self, _: &mut AttributeI) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_attribute_childof_collection(&mut self, _: &mut AttributeC) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_reference_static_childof_instance(&mut self, _: &mut RefGIR) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_reference_static_childof_collection(&mut self, _: &mut RefGCR) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_primarykey_static(&mut self, _: &mut PrimaryKeyS) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_primarykey_dynamic(&mut self, _: &mut PrimaryKeyDyn) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_foreign_key(&mut self, _: &mut ForeignKey) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_join_start(&mut self, _: &mut Join) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_join_ended(&mut self, _: &mut Join) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_where_childof_join(&mut self, _: &mut WhereJ) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_where_childof_templates(&mut self, _: &mut WhereT) -> Result<(), Self::E> {
    Ok(())
  }
}
