use std::{
  str,
  io::{BufRead, Write},
  collections::HashMap,
};

use quick_xml::{
  Reader, Writer,
  events::{
    BytesStart, Event,
    attributes::Attributes
  }
};

use paste::paste;
use serde_json::Value;
use crate::is_empty;

use super::{
  QuickXmlReadWrite, TableDataContent,
  field::Field,
  param::Param,
  group::TableGroup,
  link::Link,
  desc::Description,
  info::Info,
  data::Data,
  error::VOTableError,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum TableElem {
  Field(Field),
  Param(Param),
  TableGroup(TableGroup),
}
impl TableElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      TableElem::Field(elem) => elem.write(writer, &()),
      TableElem::Param(elem) => elem.write(writer, &()),
      TableElem::TableGroup(elem) => elem.write(writer, &()),
    }
  }
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Table<C: TableDataContent> {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  utype: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  nrows: Option<u64>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // sub-elements
  #[serde(skip_serializing_if = "Option::is_none")]
  description: Option<Description>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<TableElem>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  links: Vec<Link>,
  #[serde(skip_serializing_if = "Option::is_none")]
  data: Option<Data<C>>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  infos: Vec<Info>,
}

impl<C: TableDataContent> Table<C> {

  pub fn new() -> Self {
    Self::default()
  }
  
  impl_builder_opt_string_attr!(id);
  impl_builder_opt_string_attr!(name);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_attr!(nrows, u64);

  impl_builder_insert_extra!();

  impl_builder_opt_attr!(description, Description);
  
  impl_builder_push_elem!(Field, TableElem);
  impl_builder_push_elem!(Param, TableElem);
  impl_builder_push_elem!(TableGroup, TableElem);

  impl_builder_push!(Link);

  pub fn set_data(mut self, data: Data<C>) -> Self {
    self.data = Some(data);
    self
  }

  pub fn set_data_by_ref(&mut self, data: Data<C>) {
    self.data = Some(data);
  }
  
  impl_builder_push!(Info);

  pub fn read_till_data_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<Data<C>>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES => from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e),
            Field::TAG_BYTES => self.elems.push(TableElem::Field(from_event_start_by_ref!(Field, reader, reader_buff, e))),
            Param::TAG_BYTES => self.elems.push(TableElem::Param(from_event_start_by_ref!(Param, reader, reader_buff, e))),
            TableGroup::TAG_BYTES => self.elems.push(TableElem::TableGroup(from_event_start_by_ref!(TableGroup, reader, reader_buff, e))),
            Link::TAG_BYTES => self.links.push(from_event_start_by_ref!(Link, reader, reader_buff, e)),
            Data::<C>::TAG_BYTES => {
              let data = Data::from_attributes(e.attributes())?;
              return Ok(Some(data))
            },
            Info::TAG_BYTES => self.infos.push(from_event_start_by_ref!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Field::TAG_BYTES => self.elems.push(TableElem::Field(Field::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(TableElem::Param(Param::from_event_empty(e)?)),
            TableGroup::TAG_BYTES => self.elems.push(TableElem::TableGroup(TableGroup::from_event_empty(e)?)),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            Info::TAG_BYTES => self.infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Text(e) if is_empty(e) => {},
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(None),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }
}

impl<C: TableDataContent> QuickXmlReadWrite for Table<C> {

  const TAG: &'static str = "TABLE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut table = Self::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      table = match attr.key {
        b"ID" => table.set_id(value),
        b"name" => table.set_name(value),
        b"ucd" => table.set_ucd(value),
        b"utype" => table.set_ucd(value),
        b"ref" => table.set_ref(value),
        b"nrows" => table.set_nrows(value.parse().map_err(VOTableError::ParseInt)?),
        _ => table.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    Ok(table)
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    // If the full document is in memory, we could have use a Reader<'a [u8]> and then the method 
    // `read_event_unbuffered` to avoid a copy.
    // But are more generic that this to be able to read in streaming mode
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES => from_event_start_desc!(self, Description, reader, reader_buff, e),
            Field::TAG_BYTES => self.elems.push(TableElem::Field(from_event_start!(Field, reader, reader_buff, e))),
            Param::TAG_BYTES => self.elems.push(TableElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            TableGroup::TAG_BYTES => self.elems.push(TableElem::TableGroup(from_event_start!(TableGroup, reader, reader_buff, e))),
            Link::TAG_BYTES => self.links.push(from_event_start!(Link, reader, reader_buff, e)),
            Data::<C>::TAG_BYTES => self.data = Some(from_event_start!(Data, reader, reader_buff, e, self.elems)),
            Info::TAG_BYTES => self.infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Field::TAG_BYTES => self.elems.push(TableElem::Field(Field::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(TableElem::Param(Param::from_event_empty(e)?)),
            TableGroup::TAG_BYTES => self.elems.push(TableElem::TableGroup(TableGroup::from_event_empty(e)?)),
            Link::TAG_BYTES => self.links.push(Link::from_event_empty(e)?),
            Info::TAG_BYTES => self.infos.push(Info::from_event_empty(e)?),
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
    push2write_opt_string_attr!(self, tag, ucd);
    push2write_opt_string_attr!(self, tag, utype);
    push2write_opt_string_attr!(self, tag, ref_, ref);
    push2write_opt_tostring_attr!(self, tag, nrows);
    push2write_extra!(self, tag);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-elems
    write_elem!(self, description, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, links, writer, context);
    if let Some(elem) = &mut self.data {
      elem.write(writer, &self.elems)?;
    }
    // write_elem!(self, data, writer, self.elems);
    write_elem_vec!(self, infos, writer, context);
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}
