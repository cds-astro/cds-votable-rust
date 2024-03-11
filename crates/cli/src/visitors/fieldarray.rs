//! Print table colnames.
use std::fmt::Alignment;

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

use super::{super::get::FieldElem, StringError};

struct FieldArrayCol {
  align: Option<Alignment>,
  label: String,
  width: usize,
  elems: Vec<String>,
}
impl FieldArrayCol {
  pub fn new(align: Option<Alignment>, label: String) -> Self {
    let width = label.len();
    Self {
      align,
      label,
      width,
      elems: Vec::with_capacity(200),
    }
  }
  pub fn push<E: ToString>(&mut self, elem: E) {
    let elem = elem.to_string();
    let len = elem.len();
    if len > self.width {
      self.width = len;
    }
    self.elems.push(elem);
  }
  pub fn push_opt<E: ToString + Clone>(&mut self, opt: &Option<E>) {
    match opt.as_ref().cloned() {
      Some(e) => self.push(e),
      None => self.push(String::from("")),
    }
  }
  pub fn fmt_label(&self) -> String {
    self.fmt_str(self.label.as_str())
  }
  pub fn fmt_elem(&self, i: usize) -> String {
    self.fmt_str(self.elems[i].as_str())
  }
  fn fmt_str(&self, s: &str) -> String {
    match self.align {
      None => s.to_owned(),
      Some(Alignment::Left) => format!("{s:<w$}", w = self.width, s = s),
      Some(Alignment::Right) => format!("{s:>w$}", w = self.width, s = s),
      Some(Alignment::Center) => format!("{s:^w$}", w = self.width, s = s),
    }
  }
}

struct FieldAttributesArray {
  sep: char,
  all_fields: [FieldArrayCol; FieldElem::len()],
  to_be_printed_fields: Vec<FieldElem>,
  n_rows: usize,
}
impl FieldAttributesArray {
  fn new(sep: char, to_be_printed_fields: Vec<FieldElem>, aligned: bool) -> Self {
    let all_fields = if aligned {
      FieldElem::array()
        .map(|field| FieldArrayCol::new(Some(field.default_alignment()), field.label().to_owned()))
    } else {
      FieldElem::array().map(|field| FieldArrayCol::new(None, field.label().to_owned()))
    };
    Self {
      sep,
      all_fields,
      to_be_printed_fields,
      n_rows: 0,
    }
  }

  fn push(&mut self, field: &Field) {
    self.all_fields[FieldElem::Index.index()].push(self.n_rows);
    self.all_fields[FieldElem::Id.index()].push_opt(&field.id);
    self.all_fields[FieldElem::Name.index()].push(&field.name);
    self.all_fields[FieldElem::Datatype.index()].push(field.datatype);
    self.all_fields[FieldElem::Arraysize.index()].push_opt(&field.arraysize);
    self.all_fields[FieldElem::Width.index()].push_opt(&field.width);
    self.all_fields[FieldElem::Precision.index()].push_opt(&field.precision);
    self.all_fields[FieldElem::Unit.index()].push_opt(&field.unit);
    self.all_fields[FieldElem::Ucd.index()].push_opt(&field.ucd);
    self.all_fields[FieldElem::Null.index()]
      .push_opt(&field.values.as_ref().and_then(|v| v.null.as_ref()));
    self.all_fields[FieldElem::Min.index()].push_opt(
      &field
        .values
        .as_ref()
        .and_then(|v| v.min.as_ref())
        .map(|min| min.value.clone()),
    );
    self.all_fields[FieldElem::Max.index()].push_opt(
      &field
        .values
        .as_ref()
        .and_then(|v| v.max.as_ref())
        .map(|max| max.value.clone()),
    );
    self.all_fields[FieldElem::Link.index()]
      .push_opt(&field.links.first().and_then(|link| link.href.as_ref()));
    self.all_fields[FieldElem::Description.index()].push_opt(&field.description);
    self.n_rows += 1;
  }

  fn print(&self) {
    let mut first = true;
    for i in self.to_be_printed_fields.iter().map(|e| e.index()) {
      if first {
        first = false;
      } else {
        print!("{}", self.sep);
      }
      print!("{}", self.all_fields[i].fmt_label());
    }
    println!();
    for r in 0..self.n_rows {
      first = true;
      for i in self.to_be_printed_fields.iter().map(|e| e.index()) {
        if first {
          first = false;
        } else {
          print!("{}", self.sep);
        }
        print!("{}", self.all_fields[i].fmt_elem(r));
      }
      println!();
    }
  }
}

/// For each table, prints given FIELDS attrivutes making an array.
pub struct FieldArrayVisitor {
  sep: char,
  fields: Vec<FieldElem>,
  aligned: bool,
  attrs_array: FieldAttributesArray,
}
impl FieldArrayVisitor {
  pub fn new(sep: char, fields: Vec<FieldElem>, aligned: bool) -> Self {
    let attrs_array = FieldAttributesArray::new(sep, fields.clone(), aligned);
    Self {
      sep,
      fields,
      aligned,
      attrs_array,
    }
  }
}

impl<C: TableDataContent> VOTableVisitor<C> for FieldArrayVisitor {
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
    self.attrs_array.push(field);
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
    Ok(())
  }
  fn visit_table_ended(&mut self, _table: &mut Table<C>) -> Result<(), Self::E> {
    self.attrs_array.print();
    println!();
    self.attrs_array = FieldAttributesArray::new(self.sep, self.fields.clone(), self.aligned);
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
