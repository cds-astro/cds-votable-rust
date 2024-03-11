//! Struct dedicated to the `GROUP` tag.

use std::{
  io::{BufRead, Write},
  str,
};

use log::warn;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use super::{
  desc::Description,
  error::VOTableError,
  fieldref::FieldRef,
  param::Param,
  paramref::ParamRef,
  utils::{discard_comment, discard_event},
  QuickXmlReadWrite, TableDataContent, VOTableVisitor,
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

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  impl_builder_opt_attr!(description, Description);
  impl_builder_push_elem!(ParamRef, GroupElem);
  impl_builder_push_elem!(Param, GroupElem);
  impl_builder_push_elem!(Group, GroupElem);

  /// Calls a closure on each (key, value) attribute pairs.
  pub fn for_each_attribute<F>(&self, mut f: F)
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

impl QuickXmlReadWrite for Group {
  const TAG: &'static str = "GROUP";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut group = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      group = match attr.key {
        b"ID" => group.set_id(value),
        b"name" => group.set_name(value),
        b"ref" => group.set_ref(value),
        b"ucd" => group.set_ucd(value),
        b"utype" => group.set_utype(value),
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    Ok(group)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, context)
      .map(|()| reader)
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
            from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e);
          }
          ParamRef::TAG_BYTES => {
            self.push_paramref_by_ref(from_event_start_by_ref!(ParamRef, reader, reader_buff, e))
          }
          Param::TAG_BYTES => {
            self.push_param_by_ref(from_event_start_by_ref!(Param, reader, reader_buff, e))
          }
          Group::TAG_BYTES => {
            self.push_group_by_ref(from_event_start_by_ref!(Group, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          ParamRef::TAG_BYTES => self.push_paramref_by_ref(ParamRef::from_event_empty(e)?),
          Param::TAG_BYTES => self.push_param_by_ref(Param::from_event_empty(e)?),
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

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    if self.description.is_none() && self.elems.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      write_opt_string_attr!(self, elem_writer, ID);
      write_opt_string_attr!(self, elem_writer, name);
      write_opt_string_attr!(self, elem_writer, ref_, "ref");
      write_opt_string_attr!(self, elem_writer, ucd);
      write_opt_string_attr!(self, elem_writer, utype);
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      push2write_opt_string_attr!(self, tag, ID);
      push2write_opt_string_attr!(self, tag, name);
      push2write_opt_string_attr!(self, tag, ref_, ref);
      push2write_opt_string_attr!(self, tag, ucd);
      push2write_opt_string_attr!(self, tag, utype);
      writer
        .write_event(Event::Start(tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem!(self, description, writer, context);
      write_elem_vec_no_context!(self, elems, writer);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    }
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

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  impl_builder_opt_attr!(description, Description);
  impl_builder_push_elem!(FieldRef, TableGroupElem);
  impl_builder_push_elem!(ParamRef, TableGroupElem);
  impl_builder_push_elem!(Param, TableGroupElem);
  impl_builder_push_elem!(TableGroup, TableGroupElem);

  /// Calls a closure on each (key, value) attribute pairs.
  pub fn for_each_attribute<F>(&self, mut f: F)
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

impl QuickXmlReadWrite for TableGroup {
  const TAG: &'static str = "GROUP";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut group = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      group = match attr.key {
        b"ID" => group.set_id(value),
        b"name" => group.set_name(value),
        b"ref" => group.set_ref(value),
        b"ucd" => group.set_ucd(value),
        b"utype" => group.set_utype(value),
        _ => {
          return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
        }
      }
    }
    Ok(group)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, context)
      .map(|()| reader)
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
            from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e);
          }
          FieldRef::TAG_BYTES => {
            self.push_fieldref_by_ref(from_event_start_by_ref!(FieldRef, reader, reader_buff, e))
          }
          ParamRef::TAG_BYTES => {
            self.push_paramref_by_ref(from_event_start_by_ref!(ParamRef, reader, reader_buff, e))
          }
          Param::TAG_BYTES => {
            self.push_param_by_ref(from_event_start_by_ref!(Param, reader, reader_buff, e))
          }
          TableGroup::TAG_BYTES => self.push_tablegroup_by_ref(from_event_start_by_ref!(
            TableGroup,
            reader,
            reader_buff,
            e
          )),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          FieldRef::TAG_BYTES => self
            .elems
            .push(TableGroupElem::FieldRef(FieldRef::from_event_empty(e)?)),
          ParamRef::TAG_BYTES => self
            .elems
            .push(TableGroupElem::ParamRef(ParamRef::from_event_empty(e)?)),
          Param::TAG_BYTES => self
            .elems
            .push(TableGroupElem::Param(Param::from_event_empty(e)?)),
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

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    if self.description.is_none() && self.elems.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      write_opt_string_attr!(self, elem_writer, ID);
      write_opt_string_attr!(self, elem_writer, name);
      write_opt_string_attr!(self, elem_writer, ref_, "ref");
      write_opt_string_attr!(self, elem_writer, ucd);
      write_opt_string_attr!(self, elem_writer, utype);
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      push2write_opt_string_attr!(self, tag, ID);
      push2write_opt_string_attr!(self, tag, name);
      push2write_opt_string_attr!(self, tag, ref_, ref);
      push2write_opt_string_attr!(self, tag, ucd);
      push2write_opt_string_attr!(self, tag, utype);
      writer
        .write_event(Event::Start(tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem!(self, description, writer, context);
      write_elem_vec_no_context!(self, elems, writer);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::group::Group;
  use crate::tests::{test_read, test_writer};

  #[test]
  fn test_group_read_write() {
    let xml = r#"<GROUP ID="flux" name="Flux" ucd="phot.flux;em.radio.200-400MHz"><DESCRIPTION>Flux measured at 352MHz</DESCRIPTION><PARAM name="Freq" datatype="float" value="352" ucd="em.freq" utype="MHz"></PARAM><PARAMref ref="col4"/><PARAMref ref="col5"/></GROUP>"#;
    let group = test_read::<Group>(xml);
    assert_eq!(group.id, Some("flux".to_string()));
    assert_eq!(group.name, Some("Flux".to_string()));
    assert_eq!(group.ucd, Some("phot.flux;em.radio.200-400MHz".to_string()));
    assert_eq!(
      group.description.as_ref().unwrap().0,
      "Flux measured at 352MHz"
    );
    assert_eq!(group.elems.len(), 3);
    test_writer(group, xml);
  }
}
