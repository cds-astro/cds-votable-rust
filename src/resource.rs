use std::{
  str,
  collections::HashMap,
  io::{BufRead, Write},
};

use paste::paste;

use quick_xml::{
  Reader, Writer,
  events::{Event, BytesStart, attributes::Attributes},
};

use serde_json::Value;
use crate::data::binary2::Binary2;
use crate::data::binary::Binary;
use crate::data::Data;
use crate::impls::mem::VoidTableDataContent;

use super::{
  is_empty,
  QuickXmlReadWrite, TableDataContent,
  error::VOTableError,
  coosys::CooSys,
  timesys::TimeSys,
  group::Group,
  param::Param,
  link::Link,
  info::Info,
  desc::Description,
  table::Table,
};

#[derive(Debug)]
pub enum ResourceOrTable<C: TableDataContent> {
  Resource(Resource<C>),
  Table(Table<C>),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Resource<C: TableDataContent> {
  // attributes
  #[serde(rename = "ID", default, skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
  pub type_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // sub elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub infos: Vec<Info>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<ResourceElem>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub links: Vec<Link>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub tables: Vec<Table<C>>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<Resource<C>>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub post_infos: Vec<Info>,
}

impl<C: TableDataContent> Resource<C> {
  pub fn new() -> Self {
    Default::default()
  }

  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(type_, type);
  impl_builder_opt_string_attr!(utype);

  impl_builder_opt_attr!(description, Description);

  impl_builder_insert_extra!();

  impl_builder_push!(Info);

  impl_builder_push_elem!(CooSys, ResourceElem);
  impl_builder_push_elem!(TimeSys, ResourceElem);
  impl_builder_push_elem!(Group, ResourceElem);
  impl_builder_push_elem!(Param, ResourceElem);

  impl_builder_push!(Link);
  impl_builder_push!(Table, C);
  impl_builder_push!(Resource, C);

  impl_builder_push_post_info!();

  pub(crate) fn read_till_next_resource_or_table_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<ResourceOrTable<C>>, VOTableError> {
    reader = reader.check_end_names(false);
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES =>
              from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() =>
              self.infos.push(from_event_start_by_ref!(Info, reader, reader_buff, e)),
            Group::TAG_BYTES => self.elems.push(ResourceElem::Group(from_event_start_by_ref!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES => self.elems.push(ResourceElem::Param(from_event_start_by_ref!(Param, reader, reader_buff, e))),
            Link::TAG_BYTES => self.links.push(from_event_start_by_ref!(Link, reader, reader_buff, e)),
            Table::<C>::TAG_BYTES => {
              let table = Table::<C>::from_attributes(e.attributes())?;
              return Ok(Some(ResourceOrTable::Table(table)));
            },
            Resource::<C>::TAG_BYTES => {
              let resource = Resource::<C>::from_attributes(e.attributes())?;
              return Ok(Some(ResourceOrTable::Resource(resource)));
            },
            Info::TAG_BYTES => self.post_infos.push(from_event_start_by_ref!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() => self.infos.push(Info::from_event_empty(e)?),
            TimeSys::TAG_BYTES => self.elems.push(ResourceElem::TimeSys(TimeSys::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(ResourceElem::CooSys(CooSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(ResourceElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(ResourceElem::Param(Param::from_event_empty(e)?)),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Binary::<VoidTableDataContent>::TAG_BYTES
          || e.local_name() == Binary2::<VoidTableDataContent>::TAG_BYTES
          || e.local_name() == Data::<VoidTableDataContent>::TAG_BYTES
          || e.local_name() == Table::<VoidTableDataContent>::TAG_BYTES=> return Ok(None),
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(None),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }
  
  /*pub(crate) fn read_till_next_resource_or_table<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<(Reader<R>, Option<ResourceOrTable<C>>), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES =>
              from_event_start_desc!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() =>
              self.infos.push(from_event_start!(Info, reader, reader_buff, e)),
            Group::TAG_BYTES => self.elems.push(ResourceElem::Group(from_event_start!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES => self.elems.push(ResourceElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            Link::TAG_BYTES => self.links.push(from_event_start!(Link, reader, reader_buff, e)),
            Table::<C>::TAG_BYTES => {
              let table = Table::<C>::from_attributes(e.attributes())?;
              return Ok((reader, Some(ResourceOrTable::Table(table))));
            },
            Resource::<C>::TAG_BYTES => {
              let resource = Resource::<C>::from_attributes(e.attributes())?;
              return Ok((reader, Some(ResourceOrTable::Resource(resource))));
            },
            Info::TAG_BYTES => self.post_infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() => self.infos.push(Info::from_event_empty(e)?),
            TimeSys::TAG_BYTES => self.elems.push(ResourceElem::TimeSys(TimeSys::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(ResourceElem::CooSys(CooSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(ResourceElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(ResourceElem::Param(Param::from_event_empty(e)?)),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok((reader, None)),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }*/
}

impl<C: TableDataContent> QuickXmlReadWrite for Resource<C> {
  const TAG: &'static str = "RESOURCE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut resource = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
      resource = match attr.key {
        b"ID" => resource.set_id(value),
        b"name" => resource.set_name(value),
        b"type" => resource.set_type(value),
        b"utype" => resource.set_utype(value),
        _ => resource.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    Ok(resource)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES =>  
              from_event_start_desc!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() => 
              self.infos.push(from_event_start!(Info, reader, reader_buff, e)),
            Group::TAG_BYTES => self.elems.push(ResourceElem::Group(from_event_start!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES => self.elems.push(ResourceElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            Link::TAG_BYTES => self.links.push(from_event_start!(Link, reader, reader_buff, e)),
            Table::<C>::TAG_BYTES => self.tables.push(from_event_start!(Table, reader, reader_buff, e)),
            Resource::<C>::TAG_BYTES => self.resources.push(from_event_start!(Resource, reader, reader_buff, e)),
            Info::TAG_BYTES => self.post_infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() => self.infos.push(Info::from_event_empty(e)?),
            TimeSys::TAG_BYTES => self.elems.push(ResourceElem::TimeSys(TimeSys::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(ResourceElem::CooSys(CooSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(ResourceElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(ResourceElem::Param(Param::from_event_empty(e)?)),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => { },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(reader),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    todo!()
  }
  
  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, ID);
    push2write_opt_string_attr!(self, tag, name);
    push2write_opt_string_attr!(self, tag, type_, type);
    push2write_opt_string_attr!(self, tag, utype);
    push2write_extra!(self, tag);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem!(self, description, writer, context);
    write_elem_vec!(self, infos, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, links, writer, context);
    write_elem_vec!(self, tables, writer, context);
    write_elem_vec!(self, resources, writer, context);
    write_elem_vec!(self, post_infos, writer, context);
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}
