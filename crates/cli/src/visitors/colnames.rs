//! Print table colnames.

use votable::{
  coosys::CooSys,
  data::{fits::Fits, stream::Stream, tabledata::TableData, Data},
  definitions::Definitions,
  desc::Description,
  field::Field,
  fieldref::FieldRef,
  group::{Group, TableGroup},
  impls::mem::VoidTableDataContent,
  info::Info,
  link::Link,
  mivot::visitors::donothing::DoNothing,
  param::Param,
  paramref::ParamRef,
  resource::Resource,
  table::Table,
  timesys::TimeSys,
  values::{Max, Min, Opt, Values},
  votable::VOTable,
  TableDataContent, VOTableVisitor,
};

use super::StringError;

/// For each table, write `sep` separated colnames.
pub struct ColnamesVisitor {
  sep: char,
  first_table: bool,
  first_col: bool,
}
impl ColnamesVisitor {
  pub fn new(sep: char) -> Self {
    Self {
      sep,
      first_table: true,
      first_col: true,
    }
  }
}

impl<C: TableDataContent> VOTableVisitor<C> for ColnamesVisitor {
  type E = StringError;

  type M = DoNothing<Self::E>;

  fn visit_votable_start(&mut self, _votable: &mut VOTable<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_votable_ended(&mut self, _votable: &mut VOTable<C>) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_description(&mut self, _description: &mut Description) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_coosys_start(&mut self, _coosys: &mut CooSys) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_coosys_ended(&mut self, _coosys: &mut CooSys) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_timesys(&mut self, _timesys: &mut TimeSys) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_group_start(&mut self, _group: &mut Group) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_group_ended(&mut self, _group: &mut Group) -> Result<(), Self::E> {
    Ok(())
  }

  fn get_mivot_visitor(&mut self) -> Self::M {
    Self::M::new()
  }

  fn visit_table_group_start(&mut self, _group: &mut TableGroup) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_table_group_ended(&mut self, _group: &mut TableGroup) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_paramref(&mut self, _paramref: &mut ParamRef) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_fieldref(&mut self, _fieldref: &mut FieldRef) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_param_start(&mut self, _param: &mut Param) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_param_ended(&mut self, _param: &mut Param) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_field_start(&mut self, field: &mut Field) -> Result<(), Self::E> {
    match (self.first_col, field.name.contains(self.sep)) {
      (true, false) => print!("{}", field.name),
      (true, true) => print!("\"{}\"", field.name),
      (false, false) => print!("{}{}", self.sep, field.name),
      (false, true) => print!("{}\"{}\"", self.sep, field.name),
    }
    self.first_col = false;
    Ok(())
  }
  fn visit_field_ended(&mut self, _field: &mut Field) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_info(&mut self, _info: &mut Info) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_definitions_start(&mut self, _coosys: &mut Definitions) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_definitions_ended(&mut self, _coosys: &mut Definitions) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_resource_start(&mut self, _resource: &mut Resource<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_resource_ended(&mut self, _resource: &mut Resource<C>) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_post_info(&mut self, _info: &mut Info) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_resource_sub_elem_start(&mut self) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_resource_sub_elem_ended(&mut self) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_link(&mut self, _link: &mut Link) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_table_start(&mut self, _table: &mut Table<C>) -> Result<(), Self::E> {
    if !self.first_table {
      println!()
    }
    Ok(())
  }
  fn visit_table_ended(&mut self, _table: &mut Table<C>) -> Result<(), Self::E> {
    self.first_table = false;
    self.first_col = true;
    Ok(())
  }

  fn visit_data_start(&mut self, _table: &mut Data<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_data_ended(&mut self, _table: &mut Data<C>) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_tabledata(&mut self, _table: &mut TableData<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_binary_stream(&mut self, _stream: &mut Stream<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_binary2_stream(&mut self, _stream: &mut Stream<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_fits_start(&mut self, _fits: &mut Fits) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_fits_stream(
    &mut self,
    _stream: &mut Stream<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_fits_ended(&mut self, _fits: &mut Fits) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_values_start(&mut self, _values: &mut Values) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_values_min(&mut self, _min: &mut Min) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_values_max(&mut self, _max: &mut Max) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_values_opt_start(&mut self, _opt: &mut Opt) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_values_opt_ended(&mut self, _opt: &mut Opt) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_values_ended(&mut self, _values: &mut Values) -> Result<(), Self::E> {
    Ok(())
  }
}
