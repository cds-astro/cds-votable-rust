//! Module dedicated to the `TABLE` tag.

use std::{
  collections::HashMap,
  io::{BufRead, Write},
  str,
};

use log::warn;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};
use serde_json::Value;

use super::{
  data::Data,
  desc::Description,
  error::VOTableError,
  field::Field,
  group::TableGroup,
  info::Info,
  link::Link,
  param::Param,
  utils::{discard_comment, discard_event, is_empty},
  QuickXmlReadWrite, TableDataContent, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      TableElem::Field(e) => e.visit(visitor),
      TableElem::Param(e) => e.visit(visitor),
      TableElem::TableGroup(e) => e.visit(visitor),
    }
  }
}

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Table<C: TableDataContent> {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nrows: Option<u64>,
  // extra attributes
  /// Warning: using extra attributes in `Table` is not compatible with the VOTable schema.
  ///   use at your own risks.
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // sub-elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<TableElem>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub links: Vec<Link>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<Data<C>>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub infos: Vec<Info>,
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
    if let Some(ucd) = &self.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
    if let Some(ref_) = &self.ref_ {
      f("ref", ref_.as_str());
    }
    if let Some(nrows) = &self.nrows {
      f("nrows", nrows.to_string().as_str());
    }
    for (k, v) in &self.extra {
      f(k.as_str(), v.to_string().as_str());
    }
  }

  pub fn visit<V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    V: VOTableVisitor<C>,
  {
    visitor.visit_table_start(self)?;
    if let Some(description) = &mut self.description {
      visitor.visit_description(description)?;
    }
    for e in &mut self.elems {
      e.visit(visitor)?;
    }
    for l in &mut self.links {
      visitor.visit_link(l)?;
    }
    if let Some(data) = &mut self.data {
      data.visit(visitor)?;
    }
    for i in &mut self.infos {
      visitor.visit_info(i)?;
    }
    visitor.visit_table_ended(self)
  }

  pub(crate) fn ensures_consistency(&mut self) -> Result<(), String> {
    if let Some(data) = &mut self.data {
      data.ensures_consistency(self.elems.as_slice())
    } else {
      Ok(())
    }
  }

  pub fn read_till_data_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<Data<C>>, VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => {
            from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e)
          }
          Field::TAG_BYTES => {
            self.push_field_by_ref(from_event_start_by_ref!(Field, reader, reader_buff, e))
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
          Link::TAG_BYTES => {
            self.push_link_by_ref(from_event_start_by_ref!(Link, reader, reader_buff, e))
          }
          Data::<C>::TAG_BYTES => {
            let data = Data::from_attributes(e.attributes())?;
            return Ok(Some(data));
          }
          Info::TAG_BYTES => {
            self.push_info_by_ref(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Field::TAG_BYTES => self.push_field_by_ref(Field::from_event_empty(e)?),
          Param::TAG_BYTES => self.push_param_by_ref(Param::from_event_empty(e)?),
          TableGroup::TAG_BYTES => self.push_tablegroup_by_ref(TableGroup::from_event_empty(e)?),
          Link::TAG_BYTES => self.push_link_by_ref(Link::from_event_empty(e)?),
          Info::TAG_BYTES => self.push_info_by_ref(Info::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(None),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  /// We may want to stop just before writting the  `<DATA>` tag in case:
  /// * the table contains only metadata and no data (e.g. streamming mode)
  /// * we want to convert from TABLEDATA to BINARY
  pub(crate) fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
    stop_before_data: bool,
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
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elems
    write_elem!(self, description, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, links, writer, context);
    if !stop_before_data {
      if let Some(elem) = &mut self.data {
        elem.write_to_data_beginning(writer)?;
      }
    }
    Ok(())
  }

  /// In case we used `stop_before_data` in `write_to_data_beginning`.
  pub(crate) fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
    start_after_data: bool,
  ) -> Result<(), VOTableError> {
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);

    if !start_after_data {
      if let Some(elem) = &mut self.data {
        elem.write_from_data_end(writer)?;
      }
    }

    write_elem_vec!(self, infos, writer, context);
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }

  /// Transforms the BINARY or BINARY2 tag in this TABLE into TABLEDATA.
  /// Do nothing if it already contains a TABLEDATA or if it contains a FITS.
  pub fn to_tabledata(&mut self) -> Result<(), VOTableError> {
    if let Some(data) = self.data.take() {
      data.to_tabledata().map(|data| {
        self.data.replace(data);
      })
    } else {
      Ok(())
    }
  }

  /// Transforms the TABLEDATA or BINARY2 tag in this TABLE into BINARY.
  /// Do nothing if it already contains a BINARY or if it contains a FITS.
  pub fn to_binary(&mut self) -> Result<(), VOTableError> {
    if let Some(data) = self.data.take() {
      data.to_binary().map(|data| {
        self.data.replace(data);
      })
    } else {
      Ok(())
    }
  }

  /// Transforms the TABLEDATA or BINARY tag in this TABLE into BINARY2.
  /// Do nothing if it already contains a BINARY2 or if it contains a FITS.
  pub fn to_binary2(&mut self) -> Result<(), VOTableError> {
    if let Some(data) = self.data.take() {
      data.to_binary2().map(|data| {
        self.data.replace(data);
      })
    } else {
      Ok(())
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
    // If the full document is in memory, we could have use a Reader<'a [u8]> and then the method
    // `read_event_unbuffered` to avoid a copy.
    // But are more generic that this to be able to read in streaming mode
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => {
            from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e)
          }
          Field::TAG_BYTES => {
            self.push_field_by_ref(from_event_start_by_ref!(Field, reader, reader_buff, e))
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
          Link::TAG_BYTES => {
            self.push_link_by_ref(from_event_start_by_ref!(Link, reader, reader_buff, e))
          }
          Data::<C>::TAG_BYTES => {
            self.data = Some(from_event_start_by_ref!(
              Data,
              reader,
              reader_buff,
              e,
              self.elems
            ))
          }
          Info::TAG_BYTES => {
            self.push_info_by_ref(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Field::TAG_BYTES => self.push_field_by_ref(Field::from_event_empty(e)?),
          Param::TAG_BYTES => self.push_param_by_ref(Param::from_event_empty(e)?),
          TableGroup::TAG_BYTES => self.push_tablegroup_by_ref(TableGroup::from_event_empty(e)?),
          Link::TAG_BYTES => self.push_link_by_ref(Link::from_event_empty(e)?),
          Info::TAG_BYTES => self.push_info_by_ref(Info::from_event_empty(e)?),
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
        _ => discard_event(event, Self::TAG),
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
    push2write_opt_string_attr!(self, tag, ucd);
    push2write_opt_string_attr!(self, tag, utype);
    push2write_opt_string_attr!(self, tag, ref_, ref);
    push2write_opt_tostring_attr!(self, tag, nrows);
    push2write_extra!(self, tag);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
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
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    impls::mem::InMemTableDataRows,
    table::Table,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_table_read_write() {
    let xml = r#"<TABLE ID="V_147_sdss12" name="V/147/sdss12" nrows="2"><FIELD name="RA_ICRS" datatype="char" ucd="pos.eq.ra;meta.main"></FIELD><FIELD name="RA_ICRS" datatype="char" ucd="pos.eq.ra;meta.main"></FIELD><DATA><TABLEDATA><TR><TD>a</TD><TD>b</TD></TR><TR><TD>a</TD><TD>b</TD></TR></TABLEDATA></DATA></TABLE>"#; // Test read
    let table = test_read::<Table<InMemTableDataRows>>(xml);
    assert_eq!(table.id.as_ref().unwrap().as_str(), "V_147_sdss12");
    assert_eq!(table.name.as_ref().unwrap().as_str(), "V/147/sdss12");
    assert_eq!(table.nrows, Some(2));
    assert_eq!(table.elems.len(), 2);
    // Test write
    test_writer(table, xml);
  }
}
