//! Module dedicated to the `RESOURCE` tag.

use std::{
  collections::HashMap,
  io::{BufRead, Write},
  mem, str,
};

use log::warn;
use paste::paste;
use quick_xml::{
  events::{BytesStart, Event},
  Reader, Writer,
};
use serde_json::Value;

#[cfg(feature = "mivot")]
use super::mivot::vodml::Vodml;
use super::{
  coosys::CooSys,
  data::{binary::Binary, binary2::Binary2, Data},
  desc::Description,
  error::VOTableError,
  group::Group,
  impls::mem::VoidTableDataContent,
  info::Info,
  link::Link,
  param::Param,
  table::Table,
  timesys::TimeSys,
  utils::{discard_comment, discard_event, is_empty},
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum ResourceElem {
  CooSys(CooSys),
  TimeSys(TimeSys),
  Group(Group),
  Param(Param),
}

impl ResourceElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      ResourceElem::CooSys(elem) => elem.write(writer, &()),
      ResourceElem::TimeSys(elem) => elem.write(writer, &()),
      ResourceElem::Group(elem) => elem.write(writer, &()),
      ResourceElem::Param(elem) => elem.write(writer, &()),
    }
  }
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      ResourceElem::CooSys(e) => e.visit(visitor),
      ResourceElem::TimeSys(e) => visitor.visit_timesys(e),
      ResourceElem::Group(e) => e.visit(visitor),
      ResourceElem::Param(e) => e.visit(visitor),
    }
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum ResourceOrTable<C: TableDataContent> {
  Resource(Resource<C>),
  Table(Table<C>),
}
impl<C: TableDataContent> ResourceOrTable<C> {
  /// # WARNING
  /// Call only for a resource, not for a table with actual data.
  fn from_void_table_data_content(value: ResourceOrTable<VoidTableDataContent>) -> Self {
    match value {
      ResourceOrTable::<VoidTableDataContent>::Resource(e) => {
        Self::Resource(Resource::from_void_table_data_content(e))
      }
      ResourceOrTable::<VoidTableDataContent>::Table(_) => todo!(), // No supposed to call this!!
    }
  }
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      ResourceOrTable::Resource(elem) => elem.write(writer, &()),
      ResourceOrTable::Table(elem) => elem.write(writer, &()),
    }
  }
  pub fn visit<V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    V: VOTableVisitor<C>,
  {
    match self {
      ResourceOrTable::Resource(e) => e.visit(visitor),
      ResourceOrTable::Table(e) => e.visit(visitor),
    }
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResourceSubElem<C: TableDataContent> {
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub links: Vec<Link>,
  pub resource_or_table: ResourceOrTable<C>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub infos: Vec<Info>,
}
impl<C: TableDataContent> ResourceSubElem<C> {
  pub fn from_resource(resource: Resource<C>) -> Self {
    Self {
      links: Default::default(),
      resource_or_table: ResourceOrTable::Resource(resource),
      infos: Default::default(),
    }
  }

  pub fn from_table(table: Table<C>) -> Self {
    Self {
      links: Default::default(),
      resource_or_table: ResourceOrTable::Table(table),
      infos: Default::default(),
    }
  }

  pub fn from_void_table_data_content(value: ResourceSubElem<VoidTableDataContent>) -> Self {
    let ResourceSubElem {
      links,
      resource_or_table,
      infos,
    } = value;
    Self {
      links,
      resource_or_table: ResourceOrTable::from_void_table_data_content(resource_or_table),
      infos,
    }
  }

  impl_builder_push!(Link);
  impl_builder_push!(Info);

  pub fn set_links(mut self, links: Vec<Link>) -> Self {
    self.links = links;
    self
  }

  pub fn is_table(&self) -> bool {
    matches!(self.resource_or_table, ResourceOrTable::Table(_))
  }

  pub(crate) fn push_sub_elem_by_ref(
    &mut self,
    resource_sub_elem: ResourceSubElem<C>,
  ) -> Result<(), VOTableError> {
    match &mut self.resource_or_table {
      ResourceOrTable::Resource(resource_ref_mut) => {
        resource_ref_mut.push_sub_elem_by_ref(resource_sub_elem);
        Ok(())
      }
      ResourceOrTable::Table(_) => Err(VOTableError::Custom(String::from(
        "Algo error: not supposed to try to put a resource in a table!",
      ))),
    }
  }

  pub fn visit<V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    V: VOTableVisitor<C>,
  {
    visitor.visit_resource_sub_elem_start()?;
    for l in &mut self.links {
      visitor.visit_link(l)?;
    }
    self.resource_or_table.visit(visitor)?;
    for i in &mut self.infos {
      visitor.visit_info(i)?;
    }
    visitor.visit_resource_sub_elem_ended()
  }

  /// Transforms the BINARY or BINARY2 tag in this element into TABLEDATA.
  /// Do nothing if it already contains a TABLEDATA or if it contains a FITS.
  pub fn to_tabledata(&mut self) -> Result<(), VOTableError> {
    match &mut self.resource_or_table {
      ResourceOrTable::Resource(_) => Ok(()),
      ResourceOrTable::Table(table) => table.to_tabledata(),
    }
  }

  /// Transforms the TABLEDATA or BINARY2 tag in this element into BINARY.
  /// Do nothing if it already contains a BINARY or if it contains a FITS.
  pub fn to_binary(&mut self) -> Result<(), VOTableError> {
    match &mut self.resource_or_table {
      ResourceOrTable::Resource(_) => Ok(()),
      ResourceOrTable::Table(table) => table.to_binary(),
    }
  }

  /// Transforms the TABLEDATA or BINARY tag in this element into BINARY2.
  /// Do nothing if it already contains a BINARY2 or if it contains a FITS.
  pub fn to_binary2(&mut self) -> Result<(), VOTableError> {
    match &mut self.resource_or_table {
      ResourceOrTable::Resource(_) => Ok(()),
      ResourceOrTable::Table(table) => table.to_binary2(),
    }
  }

  /*pub(crate) fn get_table_mut(&mut self) -> Result<&mut Table<C>, VOTableError> {
    match &mut self.resource_or_table {
      ResourceOrTable::Table(table) => Ok(table),
      ResourceOrTable::Resource(_) => Err(VOTableError::Custom(String::from(
        "Algo error: is a resource, not a table!",
      ))),
    }
  }*/

  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    write_elem_vec_empty_context!(self, links, writer);
    self.resource_or_table.write(writer)?;
    write_elem_vec_empty_context!(self, infos, writer);
    Ok(())
  }
}

/// Struct corresponding to the `RESOURCE` XML tag.
#[derive(Default, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Resource<C: TableDataContent> {
  // Attributes
  #[serde(rename = "ID", default, skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  /// Extra attributes implemented here to support this piece of `xsd` in
  /// [the VOTable standard](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html):
  /// ```xml
  ///   <!-- Suggested Doug Tody, to include new RESOURCE attributes -->
  ///   <xs:anyAttribute namespace="##other" processContents="lax"/>
  /// ```
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // Sequence elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub infos: Vec<Info>,
  // - choice elements
  /// Elements in the XSD `choice`
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<ResourceElem>,
  // - sub-sequence elements
  /// Elements in the XSD sub-`sequence`
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub sub_elems: Vec<ResourceSubElem<C>>,
  // - optional extra element
  /// The VODML tag is here so that a VOTable with a VODML section pass the VOTable validator.
  /// Thanks to this `xsd` piece in
  /// [the VOTable standard](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html):
  /// allowing to create arbitrary `RESSOURCE`s.
  /// ```xml
  ///    <!-- Suggested Doug Tody, to include new RESOURCE types -->
  ///    <xs:any namespace="##other" processContents="lax" minOccurs="0" maxOccurs="unbounded"/>`
  /// ```
  #[cfg(feature = "mivot")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub vodml: Option<Vodml>,
}

impl<C: TableDataContent> Resource<C> {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn from_void_table_data_content(value: Resource<VoidTableDataContent>) -> Self {
    let Resource {
      id,
      name,
      type_,
      utype,
      extra,
      description,
      infos,
      elems,
      mut sub_elems,
      vodml,
    } = value;
    Self {
      id,
      name,
      type_,
      utype,
      extra,
      description,
      infos,
      elems,
      sub_elems: sub_elems
        .drain(..)
        .map(ResourceSubElem::from_void_table_data_content)
        .collect(),
      vodml,
    }
  }

  // Optional attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(type_, type);
  impl_builder_opt_string_attr!(utype);
  // - extra attributes
  impl_builder_insert_extra!();
  // Sequence elements
  impl_builder_opt_subelem!(description, Description);
  impl_builder_push!(Info);
  // - choice elements
  impl_builder_push_elem!(CooSys, ResourceElem);
  impl_builder_push_elem!(TimeSys, ResourceElem);
  impl_builder_push_elem!(Group, ResourceElem);
  impl_builder_push_elem!(Param, ResourceElem);

  pub fn push_elem(mut self, elem: ResourceElem) -> Self {
    self.push_elem_by_ref(elem);
    self
  }
  pub fn push_elem_by_ref(&mut self, elem: ResourceElem) {
    self.elems.push(elem);
  }

  // - sub-sequence elements
  pub fn push_sub_elem(mut self, sub_elem: ResourceSubElem<C>) -> Self {
    self.sub_elems.push(sub_elem);
    self
  }
  pub fn push_sub_elem_by_ref(&mut self, sub_elem: ResourceSubElem<C>) {
    self.sub_elems.push(sub_elem);
  }

  /// Create and push a simple sub-elem only containing a table.
  pub fn push_table(mut self, table: Table<C>) -> Self {
    self.push_table_by_ref(table);
    self
  }
  /// Create and push a simple sub-elem only containing a table.
  pub fn push_table_by_ref(&mut self, table: Table<C>) {
    self.sub_elems.push(ResourceSubElem::from_table(table));
  }
  /// Create and push a simple sub-elem only containing a resource.
  pub fn push_resource(mut self, resource: Resource<C>) -> Self {
    self.push_resource_by_ref(resource);
    self
  }
  /// Create and push a simple sub-elem only containing a resource.
  pub fn push_resource_by_ref(&mut self, resource: Resource<C>) {
    self
      .sub_elems
      .push(ResourceSubElem::from_resource(resource));
  }
  /// Create and prepend a simple sub-elem only containing a resource.
  pub fn prepend_resource(mut self, resource: Resource<C>) -> Self {
    self.prepend_resource_by_ref(resource);
    self
  }
  /// Create and prepend a simple sub-elem only containing a resource.
  pub fn prepend_resource_by_ref(&mut self, resource: Resource<C>) {
    self
      .sub_elems
      .insert(0, ResourceSubElem::from_resource(resource));
  }
  /// Push the given link into the last sub-element.
  pub fn push_link(mut self, link: Link) -> Result<Self, VOTableError> {
    self.push_link_by_ref(link).map(|()| self)
  }
  /// Push the given link into the last sub-element
  pub fn push_link_by_ref(&mut self, link: Link) -> Result<(), VOTableError> {
    self
      .sub_elems
      .last_mut()
      .map(|sub_elem| sub_elem.push_link_by_ref(link))
      .ok_or(VOTableError::Custom(String::from(
        "No sub-resource in which a link can be added",
      )))
  }

  // - extra optional element
  #[cfg(feature = "mivot")]
  impl_builder_opt_subelem!(vodml, Vodml);

  pub fn visit<V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    V: VOTableVisitor<C>,
  {
    visitor.visit_resource_start(self)?;
    if let Some(desc) = &mut self.description {
      visitor.visit_description(desc)?;
    }
    for i in &mut self.infos {
      visitor.visit_info(i)?;
    }
    for e in self.elems.iter_mut() {
      e.visit(visitor)?;
    }
    for e in self.sub_elems.iter_mut() {
      e.visit(visitor)?;
    }
    #[cfg(feature = "mivot")]
    if let Some(vodml) = &mut self.vodml {
      let mut vodml_visitor = visitor.get_mivot_visitor();
      vodml.visit(&mut vodml_visitor)?;
    }
    visitor.visit_resource_ended(self)
  }

  pub(crate) fn ensures_consistency(&mut self) -> Result<(), String> {
    for elem in self.sub_elems.iter_mut() {
      if let ResourceOrTable::Table(table) = &mut elem.resource_or_table {
        table.ensures_consistency()?;
      }
    }
    Ok(())
  }

  /// Transforms the BINARY or BINARY2 tag in this RESOURCE into TABLEDATA.
  /// Do nothing if it already contains a TABLEDATA or if it contains a FITS.
  pub fn to_tabledata(&mut self) -> Result<(), VOTableError> {
    for sub_elem in self.sub_elems.iter_mut() {
      sub_elem.to_tabledata()?;
    }
    Ok(())
  }

  /// Transforms the TABLEDATA or BINARY2 tag in this RESOURCE into BINARY.
  /// Do nothing if it already contains a BINARY or if it contains a FITS.
  pub fn to_binary(&mut self) -> Result<(), VOTableError> {
    for sub_elem in self.sub_elems.iter_mut() {
      sub_elem.to_binary()?;
    }
    Ok(())
  }

  /// Transforms the TABLEDATA or BINARY tag in this RESOURCE into BINARY2.
  /// Do nothing if it already contains a BINARY2 or if it contains a FITS.
  pub fn to_binary2(&mut self) -> Result<(), VOTableError> {
    for sub_elem in self.sub_elems.iter_mut() {
      sub_elem.to_binary2()?;
    }
    Ok(())
  }

  pub fn get_first_resource_containing_a_table(&self) -> Option<&Self> {
    for elem in &self.sub_elems {
      match &elem.resource_or_table {
        ResourceOrTable::Table(_) => return Some(self),
        ResourceOrTable::Resource(resource) => {
          let first_resource_containing_a_table = resource.get_first_resource_containing_a_table();
          if first_resource_containing_a_table.is_some() {
            return first_resource_containing_a_table;
          }
        }
      }
    }
    None
  }

  /*pub fn get_first_resource_containing_a_table_mut(&mut self) -> Option<&mut Self> {
    for elem in self.sub_elems.iter_mut() {
      match elem.resource_or_table {
        ResourceOrTable::Table(_) => return Some(self),
        ResourceOrTable::Resource(ref mut resource) => {
          let first_resource_containing_a_table =
            resource.get_first_resource_containing_a_table_mut();
          if first_resource_containing_a_table.is_some() {
            return first_resource_containing_a_table;
          }
        }
      }
    }
    None
  }*/

  pub fn get_first_table(&self) -> Option<&Table<C>> {
    for elem in self.sub_elems.iter() {
      match &elem.resource_or_table {
        ResourceOrTable::Table(table) => return Some(table),
        ResourceOrTable::Resource(resource) => {
          let first_table = resource.get_first_table();
          if first_table.is_some() {
            return first_table;
          }
        }
      }
    }
    None
  }

  pub fn get_first_table_mut(&mut self) -> Option<&mut Table<C>> {
    for elem in self.sub_elems.iter_mut() {
      match &mut elem.resource_or_table {
        ResourceOrTable::Table(table) => return Some(table),
        ResourceOrTable::Resource(resource) => {
          let first_table = resource.get_first_table_mut();
          if first_table.is_some() {
            return first_table;
          }
        }
      }
    }
    None
  }

  pub fn get_last_table(&self) -> Option<&Table<C>> {
    for elem in self.sub_elems.iter().rev() {
      match &elem.resource_or_table {
        ResourceOrTable::Table(table) => return Some(table),
        ResourceOrTable::Resource(resource) => {
          let last_table = resource.get_last_table();
          if last_table.is_some() {
            return last_table;
          }
        }
      }
    }
    None
  }

  pub fn get_last_table_mut(&mut self) -> Option<&mut Table<C>> {
    for elem in self.sub_elems.iter_mut().rev() {
      match &mut elem.resource_or_table {
        ResourceOrTable::Table(table) => return Some(table),
        ResourceOrTable::Resource(resource) => {
          let last_table = resource.get_last_table_mut();
          if last_table.is_some() {
            return last_table;
          }
        }
      }
    }
    None
  }

  /// Assume the input stream has been read till (including) the end of a
  /// `TABLEDATA` or `BINARY` or `BINARY2` tag.
  /// Then continue reading (and storing) the remaining of the VOTable (assuming
  /// it will not contains another table).
  /// If no table is found, return `false`.
  pub(crate) fn read_from_data_end_to_end<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
  ) -> Result<bool, VOTableError> {
    if let Some(sub_elem) = self.sub_elems.last_mut() {
      match &mut sub_elem.resource_or_table {
        ResourceOrTable::Table(table) => table
          .data
          .as_mut()
          .unwrap()
          .read_sub_elements_by_ref(reader, reader_buff, &Vec::default())
          .map(|()| true),
        ResourceOrTable::Resource(resource) => {
          resource.read_from_data_end_to_end(reader, reader_buff)
        }
      }
      .and_then(|table_found| {
        if table_found {
          self
            .read_sub_elements_by_ref(reader, reader_buff, &())
            .map(|()| true)
        } else {
          Ok(false)
        }
      })
    } else {
      Ok(false)
    }
  }

  pub(crate) fn read_till_next_table_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<ResourceSubElem<C>>, VOTableError> {
    let mut links: Vec<Link> = Default::default();
    reader = reader.check_end_names(false);
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          Info::TAG_BYTES => {
            let info = from_event_start_by_ref!(Info, reader, reader_buff, e);
            if let Some(sub_elem) = self.sub_elems.last_mut() {
              sub_elem.push_info_by_ref(info)
            } else {
              self.push_info_by_ref(info)
            }
          }
          CooSys::TAG_BYTES => push_from_event_start!(self, CooSys, reader, reader_buff, e),
          Group::TAG_BYTES => push_from_event_start!(self, Group, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          Link::TAG_BYTES => links.push(from_event_start_by_ref!(Link, reader, reader_buff, e)),
          #[cfg(feature = "mivot")]
          Vodml::TAG_BYTES => set_from_event_start!(self, Vodml, reader, reader_buff, e),
          Table::<C>::TAG_BYTES => {
            return Table::<C>::from_event_start(e).map(|table| {
              Some(ResourceSubElem::from_table(table).set_links(mem::take(&mut links)))
            });
          }
          Resource::<C>::TAG_BYTES => {
            let resource = from_event_start_by_ref!(Resource, reader, reader_buff, e);
            self.push_sub_elem_by_ref(
              ResourceSubElem::from_resource(resource).set_links(mem::take(&mut links)),
            );
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Info::TAG_BYTES => {
            let info = Info::from_event_empty(e)?;
            if let Some(sub_elem) = self.sub_elems.last_mut() {
              sub_elem.push_info_by_ref(info)
            } else {
              self.infos.push(info)
            }
          }
          TimeSys::TAG_BYTES => push_from_event_empty!(self, TimeSys, e),
          CooSys::TAG_BYTES => push_from_event_empty!(self, CooSys, e),
          Group::TAG_BYTES => push_from_event_empty!(self, Group, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
          Link::TAG_BYTES => links.push(Link::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e)
          if e.local_name() == Binary::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Binary2::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Data::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Table::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Self::TAG_BYTES =>
        {
          return Ok(None)
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  pub(crate) fn read_till_next_resource_or_table_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<ResourceSubElem<C>>, VOTableError> {
    let mut links: Vec<Link> = Default::default();
    reader = reader.check_end_names(false);
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          Info::TAG_BYTES => {
            let info = from_event_start_by_ref!(Info, reader, reader_buff, e);
            if let Some(sub_elem) = self.sub_elems.last_mut() {
              sub_elem.push_info_by_ref(info)
            } else {
              self.push_info_by_ref(info)
            }
          }
          CooSys::TAG_BYTES => push_from_event_start!(self, CooSys, reader, reader_buff, e),
          Group::TAG_BYTES => push_from_event_start!(self, Group, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          Link::TAG_BYTES => links.push(from_event_start_by_ref!(Link, reader, reader_buff, e)),
          #[cfg(feature = "mivot")]
          Vodml::TAG_BYTES => set_from_event_start!(self, Vodml, reader, reader_buff, e),
          Table::<C>::TAG_BYTES => {
            return Table::<C>::from_event_start(e).map(|table| {
              Some(ResourceSubElem::from_table(table).set_links(mem::take(&mut links)))
            });
          }
          Resource::<C>::TAG_BYTES => {
            return Resource::<C>::from_event_start(e).map(|resource| {
              Some(ResourceSubElem::from_resource(resource).set_links(mem::take(&mut links)))
            });
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Info::TAG_BYTES => {
            let info = Info::from_event_empty(e)?;
            if let Some(sub_elem) = self.sub_elems.last_mut() {
              sub_elem.push_info_by_ref(info)
            } else {
              self.infos.push(info)
            }
          }
          TimeSys::TAG_BYTES => push_from_event_empty!(self, TimeSys, e),
          CooSys::TAG_BYTES => push_from_event_empty!(self, CooSys, e),
          Group::TAG_BYTES => push_from_event_empty!(self, Group, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
          Link::TAG_BYTES => links.push(Link::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e)
          if e.local_name() == Binary::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Binary2::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Data::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Table::<VoidTableDataContent>::TAG_BYTES
            || e.local_name() == Self::TAG_BYTES =>
        {
          return Ok(None)
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  /// Returns `true` if a table has been found.
  /// We may want to stop just before writting the  `<DATA>` tag in case:
  /// * the table contains only metadata and no data (e.g. streamming mode)
  /// * we want to convert from TABLEDATA to BINARY
  pub(crate) fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
    stop_before_data: bool,
  ) -> Result<bool, VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    self.for_each_attribute(|k, v| tag.push_attribute((k, v)));
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem!(self, description, writer, context);
    write_elem_vec!(self, infos, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    for se in self.sub_elems.iter_mut() {
      for link in se.links.iter_mut() {
        link.write(writer, context)?;
      }
      match &mut se.resource_or_table {
        ResourceOrTable::Resource(resource) => {
          if resource.write_to_data_beginning(writer, &(), stop_before_data)? {
            return Ok(true);
          }
        }
        ResourceOrTable::Table(table) => {
          return table
            .write_to_data_beginning(writer, &(), stop_before_data)
            .map(|()| true)
        }
      }
      for info in se.infos.iter_mut() {
        info.write(writer, context)?;
      }
    }
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
      .map(|()| false)
  }

  /// Returns `true` if the resource contained a table (so that other resources have not been written)
  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
    start_after_data: bool,
  ) -> Result<bool, VOTableError> {
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    let mut iter = self.sub_elems.iter_mut();
    let mut table_found = false;
    for se in iter.by_ref() {
      if match &mut se.resource_or_table {
        ResourceOrTable::Resource(resource) => {
          resource.write_from_data_end(writer, &(), start_after_data)
        }
        ResourceOrTable::Table(table) => table
          .write_from_data_end(writer, &(), start_after_data)
          .map(|()| true),
      }? {
        table_found = true;
        for info in se.infos.iter_mut() {
          info.write(writer, context)?;
        }
        break;
      }
    }
    if table_found {
      for se in iter {
        se.write(writer)?;
      }
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
        .map(|()| true)
    } else {
      // Tag has already been closed by the call to write_to_data_beginning
      Ok(false)
    }
  }
}

impl<C: TableDataContent> VOTableElement for Resource<C> {
  const TAG: &'static str = "RESOURCE";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::default().set_attrs(attrs)
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key_str = key.as_ref();
      match key_str {
        "ID" => self.set_id_by_ref(val),
        "name" => self.set_name_by_ref(val),
        "type" => self.set_type_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
        _ => self.insert_extra_str_by_ref(key, val),
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
    if let Some(type_) = &self.type_ {
      f("type", type_.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
    for_each_extra_attribute!(self, f);
  }
}

impl<C: TableDataContent> HasSubElements for Resource<C> {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    #[cfg(not(feature = "mivot"))]
    {
      self.description.is_none()
        && self.infos.is_empty()
        && self.elems.is_empty()
        && self.sub_elems.is_empty()
    }
    #[cfg(feature = "mivot")]
    {
      self.description.is_none()
        && self.infos.is_empty()
        && self.elems.is_empty()
        && self.sub_elems.is_empty()
        && self.vodml.is_none()
    }
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut links: Vec<Link> = Default::default();
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          Info::TAG_BYTES => {
            let info = from_event_start_by_ref!(Info, reader, reader_buff, e);
            if let Some(sub_elem) = self.sub_elems.last_mut() {
              sub_elem.push_info_by_ref(info)
            } else {
              self.push_info_by_ref(info)
            }
          }
          Group::TAG_BYTES => push_from_event_start!(self, Group, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          Link::TAG_BYTES => links.push(from_event_start_by_ref!(Link, reader, reader_buff, e)),
          Table::<C>::TAG_BYTES => {
            let table = from_event_start_by_ref!(Table, reader, reader_buff, e);
            self.push_sub_elem_by_ref(
              ResourceSubElem::from_table(table).set_links(mem::take(&mut links)),
            );
          }
          Resource::<C>::TAG_BYTES => {
            let resource = from_event_start_by_ref!(Resource, reader, reader_buff, e);
            self.push_sub_elem_by_ref(
              ResourceSubElem::from_resource(resource).set_links(mem::take(&mut links)),
            );
          }
          #[cfg(feature = "mivot")]
          Vodml::TAG_BYTES => set_from_event_start!(self, Vodml, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Info::TAG_BYTES => {
            let info = Info::from_event_empty(e)?;
            if let Some(sub_elem) = self.sub_elems.last_mut() {
              sub_elem.push_info_by_ref(info)
            } else {
              self.infos.push(info)
            }
          }
          TimeSys::TAG_BYTES => push_from_event_empty!(self, TimeSys, e),
          CooSys::TAG_BYTES => push_from_event_empty!(self, CooSys, e),
          Group::TAG_BYTES => push_from_event_empty!(self, Group, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
          Link::TAG_BYTES => links.push(Link::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          return if links.is_empty() {
            Ok(())
          } else {
            Err(VOTableError::Custom(String::from("Link list not emtpy!")))
          };
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
    write_elem!(self, description, writer, context);
    write_elem_vec!(self, infos, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec_no_context!(self, sub_elems, writer);
    #[cfg(feature = "mivot")]
    write_elem_vec!(self, vodml, writer, context);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    impls::mem::InMemTableDataRows,
    resource::Resource,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_resource_read_write() {
    let xml = r#"<RESOURCE ID="yCat_5147" name="V/147"><DESCRIPTION>The SDSS Photometric Catalogue, Release 12 (Alam+, 2015)</DESCRIPTION><INFO name="matches" value="50">matching records</INFO><INFO name="warning" value="No center provided++++"/><TABLE ID="V_147_sdss12" name="V/147/sdss12"><FIELD name="RA_ICRS" datatype="char" ucd="pos.eq.ra;meta.main"/><DATA><TABLEDATA><TR><TD>a</TD></TR></TABLEDATA></DATA></TABLE><TABLE ID="V_148_sdss12" name="V/148/sdss12"><FIELD name="DE_ICRS" datatype="char" ucd="pos.eq.dec;meta.main"/><DATA><TABLEDATA><TR><TD>b</TD></TR></TABLEDATA></DATA></TABLE></RESOURCE>"#; // Test read
    let resource = test_read::<Resource<InMemTableDataRows>>(xml);
    assert_eq!(resource.id.as_ref().unwrap().as_str(), "yCat_5147");
    assert_eq!(resource.name.as_ref().unwrap().as_str(), "V/147");
    assert_eq!(
      resource
        .description
        .as_ref()
        .unwrap()
        .get_content_unwrapped(),
      "The SDSS Photometric Catalogue, Release 12 (Alam+, 2015)"
    );
    assert_eq!(resource.sub_elems.len(), 2);
    assert_eq!(resource.infos.get(0).unwrap().name, "matches");
    assert_eq!(resource.infos.get(1).unwrap().name, "warning");
    // Test write
    test_writer(resource, xml);
  }

  #[test]
  fn test_resource_read_write_w_end_info() {
    let xml = r#"<RESOURCE ID="yCat_5147" name="V/147"><DESCRIPTION>The SDSS Photometric Catalogue, Release 12 (Alam+, 2015)</DESCRIPTION><TABLE ID="V_148_sdss12" name="V/148/sdss12"><FIELD name="DE_ICRS" datatype="char" ucd="pos.eq.dec;meta.main"/><DATA><TABLEDATA><TR><TD>b</TD></TR></TABLEDATA></DATA></TABLE><INFO name="matches" value="50">matching records</INFO><INFO name="warning" value="No center provided++++"/></RESOURCE>"#; // Test read
    let resource = test_read::<Resource<InMemTableDataRows>>(xml);
    assert_eq!(resource.sub_elems[0].infos[0].name, "matches");
    assert_eq!(resource.sub_elems[0].infos[1].name, "warning");
  }
}
