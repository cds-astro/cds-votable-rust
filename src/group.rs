//! Struct dedicated to the `GROUP` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use log::warn;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use super::{
  desc::Description,
  error::VOTableError,
  fieldref::FieldRef,
  param::Param,
  paramref::ParamRef,
  utils::{discard_comment, discard_event, unexpected_attr_warn},
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum GroupElem {
  ParamRef(ParamRef),
  Param(Param),
  Group(Group),
}

impl GroupElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      GroupElem::ParamRef(elem) => elem.write(writer, &()),
      GroupElem::Param(elem) => elem.write(writer, &()),
      GroupElem::Group(elem) => elem.write(writer, &()),
    }
  }
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      GroupElem::ParamRef(e) => visitor.visit_paramref(e),
      GroupElem::Param(e) => e.visit(visitor),
      GroupElem::Group(e) => e.visit(visitor),
    }
  }
}

/// Struct corresponding to the `GROUP` XML tag when it is in a `VOTABLE` or a `RESOURCE`.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Group {
  // attributes
  #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  // Sub-elems
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<GroupElem>,
}

impl Default for Group {
  fn default() -> Self {
    Group::new()
  }
}

impl Group {
  pub fn new() -> Self {
    Group {
      id: None,
      name: None,
      ref_: None,
      ucd: None,
      utype: None,
      description: None,
      elems: vec![],
    }
  }

  // attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  // sub-elements
  impl_builder_opt_subelem!(description, Description);
  impl_builder_push_elem!(ParamRef, GroupElem);
  impl_builder_push_elem!(Param, GroupElem);
  impl_builder_push_elem!(Group, GroupElem);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_group_start(self)?;
    if let Some(descrition) = &mut self.description {
      visitor.visit_description(descrition)?;
    }
    for elem in &mut self.elems {
      elem.visit(visitor)?;
    }
    visitor.visit_group_ended(self)
  }
}

impl VOTableElement for Group {
  const TAG: &'static str = "GROUP";

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
        "name" => self.set_name_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        "ucd" => self.set_ucd_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
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
    if let Some(name) = &self.name {
      f("name", name.as_str());
    }
    if let Some(r) = &self.ref_ {
      f("ref", r.as_str());
    }
    if let Some(ucd) = &self.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
  }
}

impl HasSubElements for Group {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.description.is_none() && self.elems.is_empty()
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
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          ParamRef::TAG_BYTES => push_from_event_start!(self, ParamRef, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          Group::TAG_BYTES => push_from_event_start!(self, Group, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          ParamRef::TAG_BYTES => push_from_event_empty!(self, ParamRef, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
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
    write_elem!(self, description, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    Ok(())
  }
}

/// The only difference with the Group than can be in a VOTable or in a Resource
/// is that the TableGroup can contain FieldRef!
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum TableGroupElem {
  FieldRef(FieldRef),
  ParamRef(ParamRef),
  Param(Param),
  TableGroup(TableGroup),
}

impl TableGroupElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      TableGroupElem::FieldRef(elem) => elem.write(writer, &()),
      TableGroupElem::ParamRef(elem) => elem.write(writer, &()),
      TableGroupElem::Param(elem) => elem.write(writer, &()),
      TableGroupElem::TableGroup(elem) => elem.write(writer, &()),
    }
  }
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      TableGroupElem::FieldRef(e) => visitor.visit_fieldref(e),
      TableGroupElem::ParamRef(e) => visitor.visit_paramref(e),
      TableGroupElem::Param(e) => e.visit(visitor),
      TableGroupElem::TableGroup(e) => e.visit(visitor),
    }
  }
}

/// Struct corresponding to the `GROUP` XML tag when it is in a `TABLE`.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "Group")]
pub struct TableGroup {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  // Sub-elems
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<TableGroupElem>,
}

impl Default for TableGroup {
  fn default() -> Self {
    TableGroup::new()
  }
}

impl TableGroup {
  pub fn new() -> Self {
    TableGroup {
      id: None,
      name: None,
      ref_: None,
      ucd: None,
      utype: None,
      description: None,
      elems: vec![],
    }
  }

  // attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  // sub-elements
  impl_builder_opt_subelem!(description, Description);
  impl_builder_push_elem!(FieldRef, TableGroupElem);
  impl_builder_push_elem!(ParamRef, TableGroupElem);
  impl_builder_push_elem!(Param, TableGroupElem);
  impl_builder_push_elem!(TableGroup, TableGroupElem);

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_table_group_start(self)?;
    if let Some(desc) = &mut self.description {
      visitor.visit_description(desc)?;
    }
    for e in &mut self.elems {
      e.visit(visitor)?;
    }
    visitor.visit_table_group_ended(self)
  }
}

impl VOTableElement for TableGroup {
  const TAG: &'static str = "GROUP";

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
        "name" => self.set_name_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        "ucd" => self.set_ucd_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
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
    if let Some(name) = &self.name {
      f("name", name.as_str());
    }
    if let Some(r) = &self.ref_ {
      f("ref", r.as_str());
    }
    if let Some(ucd) = &self.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
  }
}

impl HasSubElements for TableGroup {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.description.is_none() && self.elems.is_empty()
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
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          FieldRef::TAG_BYTES => push_from_event_start!(self, FieldRef, reader, reader_buff, e),
          ParamRef::TAG_BYTES => push_from_event_start!(self, ParamRef, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          TableGroup::TAG_BYTES => push_from_event_start!(self, TableGroup, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          FieldRef::TAG_BYTES => push_from_event_empty!(self, FieldRef, e),
          ParamRef::TAG_BYTES => push_from_event_empty!(self, ParamRef, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
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
    write_elem!(self, description, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    group::Group,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_group_read_write() {
    let xml = r#"<GROUP ID="flux" name="Flux" ucd="phot.flux;em.radio.200-400MHz"><DESCRIPTION>Flux measured at 352MHz</DESCRIPTION><PARAM name="Freq" datatype="float" value="352" ucd="em.freq" utype="MHz"/><PARAMref ref="col4"/><PARAMref ref="col5"/></GROUP>"#;
    let group = test_read::<Group>(xml);
    assert_eq!(group.id, Some("flux".to_string()));
    assert_eq!(group.name, Some("Flux".to_string()));
    assert_eq!(group.ucd, Some("phot.flux;em.radio.200-400MHz".to_string()));
    assert_eq!(
      group.description.as_ref().unwrap().get_content_unwrapped(),
      "Flux measured at 352MHz"
    );
    assert_eq!(group.elems.len(), 3);
    test_writer(group, xml);
  }
}
