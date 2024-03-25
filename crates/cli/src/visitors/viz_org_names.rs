//! Retrieve original column name from VizieR column description.

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
  VOTableVisitor,
};

use super::StringError;

/// Retrieve original column name from VizieR column description and put it
/// in the 'vizier:org_namee' attribute.
/// # Warning
/// * Based on the content of the parenthesis at the end of the description.
/// * Use **only** if you are sure that the table description contain the original names
/// * Else, the ending parenthesis may contain other information (like 'J2000') so you will get rubbish...
struct ExplicitVizierOrgNamesVisitor;

impl VOTableVisitor<VoidTableDataContent> for ExplicitVizierOrgNamesVisitor {
  type E = StringError;

  type M = DoNothing<Self::E>;

  fn visit_votable_start(
    &mut self,
    _votable: &mut VOTable<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_votable_ended(
    &mut self,
    _votable: &mut VOTable<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
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
    if let Some(desc) = &field.description {
      let desc = desc.get_content_unwrapped().trim();
      if desc.ends_with(')') {
        // Unwrap ok because we tested before that it ended with a ')'.
        if let Some((_, colname)) = desc.strip_suffix(')').unwrap().rsplit_once('(') {
          let colname = colname.trim();
          // No space (unlike in comments) and not an integer (unlike in note reference)
          if !colname.contains(' ') && colname.parse::<u16>().is_err() && colname != "J2000" {
            field.insert_extra_by_ref("vizier:org_name", colname.into());
          }
        }
      }
    }
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

  fn visit_resource_start(
    &mut self,
    _resource: &mut Resource<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_resource_ended(
    &mut self,
    _resource: &mut Resource<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
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

  fn visit_table_start(&mut self, _table: &mut Table<VoidTableDataContent>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_table_ended(&mut self, _table: &mut Table<VoidTableDataContent>) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_data_start(&mut self, _table: &mut Data<VoidTableDataContent>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_data_ended(&mut self, _table: &mut Data<VoidTableDataContent>) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_tabledata(
    &mut self,
    _table: &mut TableData<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_binary_stream(
    &mut self,
    _stream: &mut Stream<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_binary2_stream(
    &mut self,
    _stream: &mut Stream<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
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
