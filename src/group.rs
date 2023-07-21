use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use paste::paste;

use super::{
  desc::Description, error::VOTableError, fieldref::FieldRef, param::Param, paramref::ParamRef,
  QuickXmlReadWrite,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
          ParamRef::TAG_BYTES => self
            .elems
            .push(GroupElem::ParamRef(from_event_start_by_ref!(
              ParamRef,
              reader,
              reader_buff,
              e
            ))),
          Param::TAG_BYTES => self.elems.push(GroupElem::Param(from_event_start_by_ref!(
            Param,
            reader,
            reader_buff,
            e
          ))),
          Group::TAG_BYTES => self.elems.push(GroupElem::Group(from_event_start_by_ref!(
            Group,
            reader,
            reader_buff,
            e
          ))),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          ParamRef::TAG_BYTES => self
            .elems
            .push(GroupElem::ParamRef(ParamRef::from_event_empty(e)?)),
          Param::TAG_BYTES => self
            .elems
            .push(GroupElem::Param(Param::from_event_empty(e)?)),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
            self
              .elems
              .push(TableGroupElem::FieldRef(from_event_start_by_ref!(
                FieldRef,
                reader,
                reader_buff,
                e
              )))
          }
          ParamRef::TAG_BYTES => {
            self
              .elems
              .push(TableGroupElem::ParamRef(from_event_start_by_ref!(
                ParamRef,
                reader,
                reader_buff,
                e
              )))
          }
          Param::TAG_BYTES => self
            .elems
            .push(TableGroupElem::Param(from_event_start_by_ref!(
              Param,
              reader,
              reader_buff,
              e
            ))),
          TableGroup::TAG_BYTES => {
            self
              .elems
              .push(TableGroupElem::TableGroup(from_event_start_by_ref!(
                TableGroup,
                reader,
                reader_buff,
                e
              )))
          }
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
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
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
