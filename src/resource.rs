use std::{
  collections::HashMap,
  io::{BufRead, Write},
  str,
};

use paste::paste;

use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use serde_json::Value;

use crate::{
  data::{binary::Binary, binary2::Binary2, Data},
  impls::mem::VoidTableDataContent,
};

use super::{
  coosys::CooSys, desc::Description, error::VOTableError, group::Group, info::Info, is_empty,
  link::Link, mivot::vodml::Vodml, param::Param, table::Table, timesys::TimeSys, QuickXmlReadWrite,
  TableDataContent,
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub vodml: Option<Vodml>,
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

  impl_builder_opt_attr!(vodml, Vodml);

  impl_builder_push!(Table, C);
  impl_builder_push!(Resource, C);

  impl_builder_push_post_info!();

  pub fn get_first_resource_containing_a_table(&self) -> Option<&Self> {
    if !self.tables.is_empty() {
      Some(self)
    } else {
      for resource in self.resources.iter() {
        let first_resource_containing_a_table = resource.get_first_resource_containing_a_table();
        if first_resource_containing_a_table.is_some() {
          return first_resource_containing_a_table;
        }
      }
      None
    }
  }

  pub fn get_first_resource_containing_a_table_mut(&mut self) -> Option<&mut Self> {
    if !self.tables.is_empty() {
      Some(self)
    } else {
      for resource in self.resources.iter_mut() {
        let first_resource_containing_a_table =
          resource.get_first_resource_containing_a_table_mut();
        if first_resource_containing_a_table.is_some() {
          return first_resource_containing_a_table;
        }
      }
      None
    }
  }

  pub fn get_first_table(&self) -> Option<&Table<C>> {
    if !self.tables.is_empty() {
      self.tables.first()
    } else {
      for resource in self.resources.iter() {
        let first_table = resource.get_first_table();
        if first_table.is_some() {
          return first_table;
        }
      }
      None
    }
  }

  pub fn get_first_table_mut(&mut self) -> Option<&mut Table<C>> {
    if !self.tables.is_empty() {
      self.tables.first_mut()
    } else {
      for resource in self.resources.iter_mut() {
        let first_table = resource.get_first_table_mut();
        if first_table.is_some() {
          return first_table;
        }
      }
      None
    }
  }

  pub(crate) fn read_till_next_resource_or_table_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<ResourceOrTable<C>>, VOTableError> {
    reader = reader.check_end_names(false);
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => {
            from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e)
          }
          Info::TAG_BYTES
            if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() =>
          {
            self
              .infos
              .push(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          CooSys::TAG_BYTES => self
            .elems
            .push(ResourceElem::CooSys(from_event_start_by_ref!(
              CooSys,
              reader,
              reader_buff,
              e
            ))),
          Group::TAG_BYTES => self
            .elems
            .push(ResourceElem::Group(from_event_start_by_ref!(
              Group,
              reader,
              reader_buff,
              e
            ))),
          Param::TAG_BYTES => self
            .elems
            .push(ResourceElem::Param(from_event_start_by_ref!(
              Param,
              reader,
              reader_buff,
              e
            ))),
          Link::TAG_BYTES => {
            self
              .links
              .push(from_event_start_by_ref!(Link, reader, reader_buff, e))
          }
          Vodml::TAG_BYTES => {
            from_event_start_vodml_by_ref!(self, Vodml, reader, reader_buff, e)
          }
          Table::<C>::TAG_BYTES => {
            let table = Table::<C>::from_attributes(e.attributes())?;
            return Ok(Some(ResourceOrTable::Table(table)));
          }
          Resource::<C>::TAG_BYTES => {
            let resource = Resource::<C>::from_attributes(e.attributes())?;
            return Ok(Some(ResourceOrTable::Resource(resource)));
          }
          Info::TAG_BYTES => {
            self
              .post_infos
              .push(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Info::TAG_BYTES
            if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() =>
          {
            self.infos.push(Info::from_event_empty(e)?)
          }
          TimeSys::TAG_BYTES => self
            .elems
            .push(ResourceElem::TimeSys(TimeSys::from_event_empty(e)?)),
          CooSys::TAG_BYTES => self
            .elems
            .push(ResourceElem::CooSys(CooSys::from_event_empty(e)?)),
          Group::TAG_BYTES => self
            .elems
            .push(ResourceElem::Group(Group::from_event_empty(e)?)),
          Param::TAG_BYTES => self
            .elems
            .push(ResourceElem::Param(Param::from_event_empty(e)?)),
          Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
          Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
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
            || e.local_name() == Table::<VoidTableDataContent>::TAG_BYTES =>
        {
          return Ok(None)
        }
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(None),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  /// Returns `true` if a table has been found.
  pub(crate) fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
  ) -> Result<bool, VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, ID);
    push2write_opt_string_attr!(self, tag, name);
    push2write_opt_string_attr!(self, tag, type_, type);
    push2write_opt_string_attr!(self, tag, utype);
    push2write_extra!(self, tag);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem!(self, description, writer, context);
    write_elem_vec!(self, infos, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, links, writer, context);

    if self.tables.len() > 1 {
      Err(VOTableError::Custom(String::from(
        "In resource, more than one table not adapted for write_to_data_beginning!",
      )))
    } else if let Some(table) = self.tables.first_mut() {
      table.write_to_data_beginning(writer, &()).map(|()| true)
    } else {
      for resource in self.resources.iter_mut() {
        if resource.write_to_data_beginning(writer, &())? {
          return Ok(true);
        }
      }
      // No table found, so write everything
      write_elem_vec!(self, post_infos, writer, context);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
        .map(|()| false)
    }
  }

  /// Returns `true` if the resource contained a table (so that other resources have not been written)
  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
  ) -> Result<bool, VOTableError> {
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    if self.tables.len() > 1 {
      Err(VOTableError::Custom(String::from(
        "In resource, more than one table not adapted for write_to_data_beginning!",
      )))
    } else if let Some(table) = self.tables.first_mut() {
      table.write_from_data_end(writer, &())?;
      // Here we assume no table in sub-ressources!!
      write_elem_vec!(self, resources, writer, context);
      write_elem_vec!(self, post_infos, writer, context);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
        .map(|()| true)
    } else {
      let mut write = false;
      for resource in self.resources.iter_mut() {
        if write {
          resource.write(writer, context)?;
        } else {
          write = resource.write_from_data_end(writer, &())?;
        }
      }
      if write {
        write_elem_vec!(self, post_infos, writer, context);
        // Close tag
        writer
          .write_event(Event::End(tag.to_end()))
          .map_err(VOTableError::Write)
          .map(|()| true)
      } else {
        Ok(false)
      }
    }
  }
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
            from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e)
          }
          Info::TAG_BYTES
            if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() =>
          {
            self
              .infos
              .push(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          Group::TAG_BYTES => self
            .elems
            .push(ResourceElem::Group(from_event_start_by_ref!(
              Group,
              reader,
              reader_buff,
              e
            ))),
          Param::TAG_BYTES => self
            .elems
            .push(ResourceElem::Param(from_event_start_by_ref!(
              Param,
              reader,
              reader_buff,
              e
            ))),
          Link::TAG_BYTES => {
            self
              .links
              .push(from_event_start_by_ref!(Link, reader, reader_buff, e))
          }
          Vodml::TAG_BYTES => {
            from_event_start_vodml_by_ref!(self, Vodml, reader, reader_buff, e)
          }
          Table::<C>::TAG_BYTES => {
            self
              .tables
              .push(from_event_start_by_ref!(Table, reader, reader_buff, e))
          }
          Resource::<C>::TAG_BYTES => {
            self
              .resources
              .push(from_event_start_by_ref!(Resource, reader, reader_buff, e))
          }
          Info::TAG_BYTES => {
            self
              .post_infos
              .push(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Info::TAG_BYTES
            if self.elems.is_empty()
              && self.links.is_empty()
              && self.tables.is_empty()
              && self.resources.is_empty() =>
          {
            self.infos.push(Info::from_event_empty(e)?)
          }
          TimeSys::TAG_BYTES => self
            .elems
            .push(ResourceElem::TimeSys(TimeSys::from_event_empty(e)?)),
          CooSys::TAG_BYTES => self
            .elems
            .push(ResourceElem::CooSys(CooSys::from_event_empty(e)?)),
          Group::TAG_BYTES => self
            .elems
            .push(ResourceElem::Group(Group::from_event_empty(e)?)),
          Param::TAG_BYTES => self
            .elems
            .push(ResourceElem::Param(Param::from_event_empty(e)?)),
          Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
          Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(()),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, ID);
    push2write_opt_string_attr!(self, tag, name);
    push2write_opt_string_attr!(self, tag, type_, type);
    push2write_opt_string_attr!(self, tag, utype);
    push2write_extra!(self, tag);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem!(self, description, writer, context);
    write_elem_vec!(self, infos, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, links, writer, context);
    write_elem_vec!(self, vodml, writer, context);
    write_elem_vec!(self, tables, writer, context);
    write_elem_vec!(self, resources, writer, context);
    write_elem_vec!(self, post_infos, writer, context);
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
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
    let xml = r#"<RESOURCE ID="yCat_5147" name="V/147"><DESCRIPTION>The SDSS Photometric Catalogue, Release 12 (Alam+, 2015)</DESCRIPTION><INFO name="matches" value="50">matching records</INFO><INFO name="warning" value="No center provided++++"/><TABLE ID="V_147_sdss12" name="V/147/sdss12"><FIELD name="RA_ICRS" datatype="char" ucd="pos.eq.ra;meta.main"></FIELD><DATA><TABLEDATA><TR><TD>a</TD></TR></TABLEDATA></DATA></TABLE><TABLE ID="V_148_sdss12" name="V/148/sdss12"><FIELD name="DE_ICRS" datatype="char" ucd="pos.eq.dec;meta.main"></FIELD><DATA><TABLEDATA><TR><TD>b</TD></TR></TABLEDATA></DATA></TABLE></RESOURCE>"#; // Test read
    let resource = test_read::<Resource<InMemTableDataRows>>(xml);
    assert_eq!(resource.id.as_ref().unwrap().as_str(), "yCat_5147");
    assert_eq!(resource.name.as_ref().unwrap().as_str(), "V/147");
    assert_eq!(
      resource.description.as_ref().unwrap().0,
      "The SDSS Photometric Catalogue, Release 12 (Alam+, 2015)"
    );
    assert_eq!(resource.tables.len(), 2);
    assert_eq!(resource.infos.get(0).unwrap().name, "matches");
    assert_eq!(resource.infos.get(1).unwrap().name, "warning");
    // Test write
    test_writer(resource, xml);
  }

  #[test]
  fn test_resource_read_write_w_end_info() {
    let xml = r#"<RESOURCE ID="yCat_5147" name="V/147"><DESCRIPTION>The SDSS Photometric Catalogue, Release 12 (Alam+, 2015)</DESCRIPTION><TABLE ID="V_148_sdss12" name="V/148/sdss12"><FIELD name="DE_ICRS" datatype="char" ucd="pos.eq.dec;meta.main"></FIELD><DATA><TABLEDATA><TR><TD>b</TD></TR></TABLEDATA></DATA></TABLE><INFO name="matches" value="50">matching records</INFO><INFO name="warning" value="No center provided++++"/></RESOURCE>"#; // Test read
    let resource = test_read::<Resource<InMemTableDataRows>>(xml);
    assert_eq!(resource.post_infos.get(0).unwrap().name, "matches");
    assert_eq!(resource.post_infos.get(1).unwrap().name, "warning");
  }
}
