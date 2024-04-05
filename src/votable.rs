//! Module dedicated to the `VOTABLE` tag.

use std::{
  collections::HashMap,
  fmt::{Display, Formatter},
  fs::File,
  io::{BufRead, BufReader, BufWriter, Write},
  path::Path,
  str::{self, FromStr},
};

use log::{debug, warn};
use paste::paste;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Re-export the quick_xml elements;
pub use quick_xml::{
  events::{attributes::Attributes, BytesDecl, BytesEnd, BytesStart, Event},
  Reader, Writer,
};

use super::{
  coosys::CooSys,
  definitions::Definitions,
  desc::Description,
  error::VOTableError,
  group::Group,
  info::Info,
  param::Param,
  resource::Resource,
  table::Table,
  timesys::TimeSys,
  utils::{discard_comment, discard_event, is_empty},
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
};

pub fn new_xml_writer<W: Write>(
  writer: W,
  indent_char: Option<u8>,
  indent_size: Option<usize>,
) -> Writer<W> {
  Writer::new_with_indent(
    writer,
    indent_char.unwrap_or(b' '),
    indent_size.unwrap_or(4),
  )
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Version {
  #[serde(rename = "1.0")]
  V1_0,
  #[serde(rename = "1.1")]
  V1_1,
  #[serde(rename = "1.2")]
  V1_2,
  #[serde(rename = "1.3")]
  V1_3,
  #[serde(rename = "1.4")]
  V1_4,
  #[serde(rename = "1.5")]
  V1_5,
}
impl Version {
  /// Returns the **XML** **N**ame**S**pace associated to this particular VOTable version.
  pub fn get_xmlns(&self) -> &'static str {
    match self {
      Self::V1_0 => "http://www.ivoa.net/xml/VOTable/v1.0",
      Self::V1_1 => "http://www.ivoa.net/xml/VOTable/v1.1",
      Self::V1_2 => "http://www.ivoa.net/xml/VOTable/v1.2",
      Self::V1_3 => "http://www.ivoa.net/xml/VOTable/v1.3",
      Self::V1_4 => "http://www.ivoa.net/xml/VOTable/v1.4",
      Self::V1_5 => "http://www.ivoa.net/xml/VOTable/v1.5",
    }
  }

  pub fn get_xsi_schema_loc(&self) -> &'static str {
    match self {
      Self::V1_0 => "http://www.ivoa.net/xml/VOTable/v1.0",
      Self::V1_1 => "http://www.ivoa.net/xml/VOTable/v1.1",
      Self::V1_2 => "http://www.ivoa.net/xml/VOTable/v1.2",
      Self::V1_3 => "http://www.ivoa.net/xml/VOTable/v1.3",
      Self::V1_4 => "http://www.ivoa.net/xml/VOTable/v1.4",
      Self::V1_5 => "http://www.ivoa.net/xml/VOTable/v1.5",
    }
  }
}
impl Display for Version {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.into())
  }
}

impl FromStr for Version {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "1.0" => Ok(Version::V1_0),
      "1.1" => Ok(Version::V1_1),
      "1.2" => Ok(Version::V1_2),
      "1.3" => Ok(Version::V1_3),
      "1.4" => Ok(Version::V1_4),
      "1.5" => Ok(Version::V1_5),
      _ => Err(format!(
        "Unrecognized version. Actual: '{}'. Expected: '1.1', '1.2', '1.3', '1.4' or '1.5'",
        s
      )),
    }
  }
}

impl From<&Version> for &'static str {
  fn from(version: &Version) -> Self {
    match version {
      Version::V1_0 => "1.0",
      Version::V1_1 => "1.1",
      Version::V1_2 => "1.2",
      Version::V1_3 => "1.3",
      Version::V1_4 => "1.4",
      Version::V1_5 => "1.5",
    }
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum VOTableElem {
  CooSys(CooSys),
  TimeSys(TimeSys),
  Group(Group),
  Param(Param),
  Info(Info),
}

impl VOTableElem {
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      VOTableElem::CooSys(elem) => elem.visit(visitor),
      VOTableElem::TimeSys(elem) => visitor.visit_timesys(elem),
      VOTableElem::Group(elem) => elem.visit(visitor),
      VOTableElem::Param(elem) => elem.visit(visitor),
      VOTableElem::Info(elem) => visitor.visit_info(elem),
    }
  }
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      VOTableElem::CooSys(elem) => elem.write(writer, &()),
      VOTableElem::TimeSys(elem) => elem.write(writer, &()),
      VOTableElem::Group(elem) => elem.write(writer, &()),
      VOTableElem::Param(elem) => elem.write(writer, &()),
      VOTableElem::Info(elem) => elem.write(writer, &()),
    }
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VOTableWrapper<C: TableDataContent> {
  pub votable: VOTable<C>,
}

impl<C: TableDataContent> VOTableWrapper<C> {
  /// Returns the inner `VOTable` element
  pub fn unwrap(self) -> VOTable<C> {
    self.votable
  }

  /// Transforms the BINARY or BINARY2 tag in this VOTABLE into TABLEDATA.
  /// Do nothing if it already contains a TABLEDATA or if it contains a FITS.
  pub fn to_tabledata(&mut self) -> Result<(), VOTableError> {
    self.votable.to_tabledata()
  }

  /// Transforms the TABLEDATA or BINARY2 tag in this VOTABLE into BINARY.
  /// Do nothing if it already contains a BINARY or if it contains a FITS.
  pub fn to_binary(&mut self) -> Result<(), VOTableError> {
    self.votable.to_binary()
  }

  /// Transforms the TABLEDATA or BINARY tag in this VOTABLE into BINARY2.
  /// Do nothing if it already contains a BINARY2 or if it contains a FITS.
  pub fn to_binary2(&mut self) -> Result<(), VOTableError> {
    self.votable.to_binary2()
  }

  pub(crate) fn ensures_consistency(mut self) -> Result<Self, VOTableError> {
    self
      .votable
      .ensures_consistency()
      .map_err(VOTableError::Custom)?;
    Ok(self)
  }

  // Manual parser

  pub fn manual_from_ivoa_xml_file<P: AsRef<Path>>(
    path: P,
    reader_buff: &mut Vec<u8>,
  ) -> Result<(VOTable<C>, Resource<C>, Reader<BufReader<File>>), VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    VOTable::from_reader_till_next_resource(reader, reader_buff)
  }

  // XML

  pub fn from_ivoa_xml_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_ivoa_xml_reader(reader)
  }

  pub fn from_ivoa_xml_str(s: &str) -> Result<Self, VOTableError> {
    Self::from_ivoa_xml_reader(s.as_bytes())
  }

  pub fn from_ivoa_xml_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    Self::from_ivoa_xml_reader(s)
  }

  pub fn from_ivoa_xml_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    VOTable::from_reader(reader).map(|vot| vot.wrap())
  }

  pub fn to_ivoa_xml_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), VOTableError> {
    let file = File::create(path).map_err(VOTableError::Io)?;
    let write = BufWriter::new(file);
    self.to_ivoa_xml_writer(write)
  }

  pub fn to_ivoa_xml_string(&mut self) -> Result<String, VOTableError> {
    let buff = self.to_ivoa_xml_bytes()?;
    String::from_utf8(buff).map_err(VOTableError::FromUtf8)
  }

  pub fn to_ivoa_xml_bytes(&mut self) -> Result<Vec<u8>, VOTableError> {
    let mut votable: Vec<u8> = Vec::new();
    self.to_ivoa_xml_writer(&mut votable)?;
    Ok(votable)
  }

  pub fn to_ivoa_xml_writer<W: Write>(&mut self, write: W) -> Result<(), VOTableError> {
    let mut write = Writer::new_with_indent(write, b' ', 4);
    write
      .write(
        r#"<?xml version="1.0" encoding="UTF-8"?>
"#
        .as_bytes(),
      )
      .map_err(VOTableError::Write)
      .and_then(|()| self.votable.write(&mut write, &()))
  }
}

impl<C> VOTableWrapper<C>
where
  C: TableDataContent + Serialize + for<'a> Deserialize<'a>,
{
  // JSON

  pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_json_reader(reader)
  }

  pub fn from_json_str(s: &str) -> Result<Self, VOTableError> {
    serde_json::from_str(s)
      .map_err(VOTableError::Json)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn from_json_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    serde_json::from_slice(s)
      .map_err(VOTableError::Json)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn from_json_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    serde_json::from_reader(reader)
      .map_err(VOTableError::Json)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn to_json_file<P: AsRef<Path>>(
    &mut self,
    path: P,
    pretty: bool,
  ) -> Result<(), VOTableError> {
    let file = File::create(path).map_err(VOTableError::Io)?;
    let write = BufWriter::new(file);
    self.to_json_writer(write, pretty)
  }

  pub fn to_json_string(&mut self, pretty: bool) -> Result<String, VOTableError> {
    if pretty {
      serde_json::ser::to_string_pretty(&self)
    } else {
      serde_json::ser::to_string(&self)
    }
    .map_err(VOTableError::Json)
  }

  pub fn to_json_bytes(&mut self, pretty: bool) -> Result<Vec<u8>, VOTableError> {
    if pretty {
      serde_json::ser::to_vec_pretty(&self)
    } else {
      serde_json::ser::to_vec(&self)
    }
    .map_err(VOTableError::Json)
  }

  pub fn to_json_writer<W: Write>(&mut self, write: W, pretty: bool) -> Result<(), VOTableError> {
    if pretty {
      serde_json::ser::to_writer_pretty(write, &self)
    } else {
      serde_json::ser::to_writer(write, &self)
    }
    .map_err(VOTableError::Json)
  }

  // YAML

  pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_yaml_reader(reader)
  }

  pub fn from_yaml_str(s: &str) -> Result<Self, VOTableError> {
    serde_yaml::from_str(s)
      .map_err(VOTableError::Yaml)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn from_yaml_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    serde_yaml::from_slice(s)
      .map_err(VOTableError::Yaml)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn from_yaml_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    serde_yaml::from_reader(reader)
      .map_err(VOTableError::Yaml)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn to_yaml_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), VOTableError> {
    let file = File::create(path).map_err(VOTableError::Io)?;
    let write = BufWriter::new(file);
    self.to_yaml_writer(write)
  }

  pub fn to_yaml_string(&mut self) -> Result<String, VOTableError> {
    serde_yaml::to_string(&self).map_err(VOTableError::Yaml)
  }

  pub fn to_yaml_bytes(&mut self) -> Result<Vec<u8>, VOTableError> {
    serde_yaml::to_string(&self)
      .map(|s| s.into())
      .map_err(VOTableError::Yaml)
  }

  pub fn to_yaml_writer<W: Write>(&mut self, write: W) -> Result<(), VOTableError> {
    serde_yaml::to_writer(write, &self).map_err(VOTableError::Yaml)
  }

  // TOML

  pub fn from_toml_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_toml_reader(reader)
  }

  pub fn from_toml_str(s: &str) -> Result<Self, VOTableError> {
    toml::from_str(s)
      .map_err(VOTableError::TomlDe)
      .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn from_toml_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    match str::from_utf8(s) {
      Ok(s) => toml::from_str(s).map_err(VOTableError::TomlDe),
      Err(e) => Err(VOTableError::Custom(e.to_string())),
    }
    .and_then(|r: Self| r.ensures_consistency())
  }

  pub fn from_toml_reader<R: BufRead>(mut reader: R) -> Result<Self, VOTableError> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(VOTableError::Io)?;
    Self::from_toml_bytes(data.as_slice())
  }

  pub fn to_toml_file<P: AsRef<Path>>(
    &mut self,
    path: P,
    pretty: bool,
  ) -> Result<(), VOTableError> {
    let content = self.to_toml_string(pretty)?;
    std::fs::write(path, content).map_err(VOTableError::Io)
    /*let file = File::create(path).map_err(VOTableError::Io)?;
    let write = BufWriter::new(file);
    self.to_toml_writer(write)*/
  }

  pub fn to_toml_string(&mut self, pretty: bool) -> Result<String, VOTableError> {
    if pretty {
      toml::ser::to_string_pretty(&self)
    } else {
      toml::ser::to_string(&self)
    }
    .map_err(VOTableError::TomlSer)
  }

  pub fn to_toml_bytes(&mut self, pretty: bool) -> Result<Vec<u8>, VOTableError> {
    // toml::ser::to_vec(&self).map_err(VOTableError::TomlSer)
    self.to_toml_string(pretty).map(|s| s.into_bytes())
  }

  pub fn to_toml_writer<W: Write>(
    &mut self,
    mut write: W,
    pretty: bool,
  ) -> Result<(), VOTableError> {
    let bytes = self.to_toml_bytes(pretty)?;
    write.write_all(bytes.as_slice()).map_err(VOTableError::Io)
  }
}

/// Struct corresponding to the `VOTABLE` XML tag.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VOTable<C: TableDataContent> {
  // attributes
  #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub version: Version,
  /// E.g. "http://www.ivoa.net/xml/VOTable/v1.3", see method in the `Version` enum.
  pub xmlns: String,
  #[serde(rename = "xmlns:xsi", skip_serializing_if = "Option::is_none")]
  /// E.g. "http://www.w3.org/2001/XMLSchema-instance"
  pub xmlns_xsi: Option<String>,
  /// E.g. ""http://www.ivoa.net/xml/VOTable/v1.3 http://www.ivoa.net/xml/VOTable/v1.3""
  #[serde(rename = "xsi:schemaLocation", skip_serializing_if = "Option::is_none")]
  pub xsi_schema_location: Option<String>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(skip_serializing_if = "Option::is_none")]
  /// `DEFINITIONS` is deprecated since VOTable 1.1
  pub definitions: Option<Definitions>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<VOTableElem>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<Resource<C>>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub post_infos: Vec<Info>,
}

/// Setters are here to simply the code (not having to create a `String` from a `&str`, to wrap
/// in an `Option`, ...
impl<C: TableDataContent> VOTable<C> {
  /// Decorate this VOTable by an object to get the following serialization:
  /// ```bash
  /// {
  ///   "votable": {
  ///     "version": "1.4",
  ///     ...
  ///   }
  /// }
  /// ```
  /// Instead of:
  /// ```bash
  /// {
  ///   "version": "1.4",
  ///   ...
  /// }
  /// ```
  pub fn wrap(self) -> VOTableWrapper<C> {
    VOTableWrapper { votable: self }
  }

  /// Not public because a VOTable is supposed to contains at least one Resource.
  fn new_empty(version: Version) -> Self {
    let xmlns = version.get_xmlns().to_string();
    Self {
      id: None,
      version,
      xmlns,
      xmlns_xsi: None,
      xsi_schema_location: None,
      extra: Default::default(),
      description: None,
      definitions: None,
      elems: Default::default(),
      resources: Default::default(),
      post_infos: Default::default(),
    }
  }

  /// Create a new VOTable implementing the given `Version` and containing the given `Resource`.
  /// # Note
  /// * the `xmlns` attribute is automatically generated from the given `Version`.
  pub fn new(version: Version, resource: Resource<C>) -> Self {
    let votable = Self::new_empty(version);
    votable.push_resource(resource)
  }

  pub(crate) fn ensures_consistency(&mut self) -> Result<(), String> {
    for ressource in self.resources.iter_mut() {
      ressource.ensures_consistency()?;
    }
    Ok(())
  }

  pub fn visit<V: VOTableVisitor<C>>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_votable_start(self)?;
    if let Some(desc) = &mut self.description {
      visitor.visit_description(desc)?;
    }
    if let Some(e) = &mut self.definitions {
      e.visit(visitor)?;
    }
    for e in self.elems.iter_mut() {
      e.visit(visitor)?;
    }
    for r in self.resources.iter_mut() {
      r.visit(visitor)?;
    }
    for i in self.post_infos.iter_mut() {
      visitor.visit_post_info(i)?;
    }
    visitor.visit_votable_ended(self)
  }

  pub(crate) fn from_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    let mut reader = Reader::from_reader(reader);
    let mut buff: Vec<u8> = Vec::with_capacity(1024);
    loop {
      let mut event = reader.read_event(&mut buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Decl(ref e) => check_declaration(e),
        Event::Start(ref mut e) if e.local_name() == VOTable::<C>::TAG_BYTES => {
          // Ignore the remaining of the reader !
          return VOTable::<C>::from_event_start(e)
            .and_then(|vot| vot.read_content(&mut reader, &mut buff, &()));
        }
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  /// Read (and store) the content of the VOTable till it finds the first `RESOURCE`.
  /// It then build the `RESOURCE` from its attributes, without adding it to the VOTable, and returns.
  pub(crate) fn from_reader_till_next_resource<R: BufRead>(
    reader: R,
    reader_buff: &mut Vec<u8>,
  ) -> Result<(VOTable<C>, Resource<C>, Reader<R>), VOTableError> {
    let mut reader = Reader::from_reader(reader);
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Decl(ref e) => check_declaration(e),
        Event::Start(ref mut e) if e.local_name() == VOTable::<C>::TAG_BYTES => {
          return VOTable::<C>::from_event_start(e).and_then(|mut votable| {
            votable
              .read_till_next_resource(reader, reader_buff)
              .map(|(resource, reader)| {
                reader_buff.clear();
                (votable, resource, reader)
              })
          });
        }
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  impl_builder_opt_string_attr!(id);
  impl_builder_mandatory_attr!(version, Version);
  impl_builder_mandatory_string_attr!(xmlns);
  impl_builder_opt_string_attr!(xmlns_xsi);
  impl_builder_opt_string_attr!(xsi_schema_location);

  impl_builder_insert_extra!();

  impl_builder_opt_subelem!(description, Description);
  impl_builder_opt_subelem!(definitions, Definitions);

  impl_builder_push_elem!(CooSys, VOTableElem);
  impl_builder_push_elem!(TimeSys, VOTableElem);
  impl_builder_push_elem!(Group, VOTableElem);
  impl_builder_push_elem!(Param, VOTableElem);
  impl_builder_push_elem!(Info, VOTableElem);

  pub fn push_elem(mut self, elem: VOTableElem) -> Self {
    self.push_elem_by_ref(elem);
    self
  }
  pub fn push_elem_by_ref(&mut self, elem: VOTableElem) {
    self.elems.push(elem);
  }

  impl_builder_push!(Resource, C);
  impl_builder_prepend!(Resource, C);

  impl_builder_push_post_info!();

  /// Transforms the BINARY or BINARY2 tag in this VOTABLE into TABLEDATA.
  /// Do nothing if it already contains a TABLEDATA or if it contains a FITS.
  pub fn to_tabledata(&mut self) -> Result<(), VOTableError> {
    for resource in self.resources.iter_mut() {
      resource.to_tabledata()?;
    }
    Ok(())
  }

  /// Transforms the TABLEDATA or BINARY2 tag in this VOTABLE into BINARY.
  /// Do nothing if it already contains a BINARY or if it contains a FITS.
  pub fn to_binary(&mut self) -> Result<(), VOTableError> {
    for resource in self.resources.iter_mut() {
      resource.to_binary()?;
    }
    Ok(())
  }

  /// Transforms the TABLEDATA or BINARY tag in this VOTABLE into BINARY2.
  /// Do nothing if it already contains a BINARY2 or if it contains a FITS.
  pub fn to_binary2(&mut self) -> Result<(), VOTableError> {
    for resource in self.resources.iter_mut() {
      resource.to_binary2()?;
    }
    Ok(())
  }

  pub fn get_first_resource_containing_a_table(&self) -> Option<&Resource<C>> {
    for resource in self.resources.iter() {
      let first_resource_containing_a_table = resource.get_first_resource_containing_a_table();
      if first_resource_containing_a_table.is_some() {
        return first_resource_containing_a_table;
      }
    }
    None
  }

  /*pub fn get_first_resource_containing_a_table_mut(&mut self) -> Option<&mut Resource<C>> {
    for resource in self.resources.iter_mut() {
      let first_resource_containing_a_table = resource.get_first_resource_containing_a_table_mut();
      if first_resource_containing_a_table.is_some() {
        return first_resource_containing_a_table;
      }
    }
    None
  }*/

  pub fn get_first_table(&self) -> Option<&Table<C>> {
    for resource in self.resources.iter() {
      let first_table = resource.get_first_table();
      if first_table.is_some() {
        return first_table;
      }
    }
    None
  }

  pub fn get_first_table_mut(&mut self) -> Option<&mut Table<C>> {
    for resource in self.resources.iter_mut() {
      let first_table = resource.get_first_table_mut();
      if first_table.is_some() {
        return first_table;
      }
    }
    None
  }

  /// Read (and store) the content of the VOTable till it reaches a `RESOURCE` tag **child of** `VOTABLE`.
  /// The newly found `RESOURCE` tag is built from its `attributes` but its content
  /// (the sub-elements it contains) is to be read and the `RESOURCE` is not added to the VOTable
  /// (`VOTable.push_resource` must be called externally).
  pub fn read_till_next_resource_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<Option<Resource<C>>, VOTableError> {
    // If the full document is in memory, we could have use a Reader<'a [u8]> and then the method
    // `read_event_unbuffered` to avoid a copy.
    // But are more generic that this to be able to read in streaming mode
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          Definitions::TAG_BYTES => {
            set_from_event_start!(self, Definitions, reader, reader_buff, e)
          }
          Info::TAG_BYTES if self.resources.is_empty() => {
            push_from_event_start!(self, Info, reader, reader_buff, e)
          }
          Group::TAG_BYTES => push_from_event_start!(self, Group, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          Resource::<C>::TAG_BYTES => {
            return Resource::<C>::from_event_start(e).map(Some);
          }
          Info::TAG_BYTES => {
            self.push_post_info_by_ref(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Definitions::TAG_BYTES => set_from_event_empty!(self, Definitions, e),
          Info::TAG_BYTES => {
            let info = Info::from_event_empty(e)?;
            if self.resources.is_empty() {
              self.push_info_by_ref(info);
            } else {
              self.push_post_info_by_ref(info);
            }
          }
          CooSys::TAG_BYTES => push_from_event_empty!(self, CooSys, e),
          TimeSys::TAG_BYTES => push_from_event_empty!(self, TimeSys, e),
          Group::TAG_BYTES => push_from_event_empty!(self, Group, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(None),
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  /// Read (and store) the content of the VOTable till it reaches a `RESOURCE` tag **child of** `VOTABLE`.
  /// The newly found `RESOURCE` tag is built from its `attributes` but its content.
  /// (the sub-elements it contains) are to be read.
  /// The `VOTable.push_resource` has to be done externally.
  pub(crate) fn read_till_next_resource<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
  ) -> Result<(Resource<C>, Reader<R>), VOTableError> {
    match self.read_till_next_resource_by_ref(&mut reader, reader_buff)? {
      Some(resource) => Ok((resource, reader)),
      None => Err(VOTableError::Custom(String::from(
        "No resource found in the VOTable",
      ))),
    }
  }

  /// Assume the input stream has been read till (including) the end of a
  /// `TABLEDATA` or `BINARY` or `BINARY2` tag.
  /// Then continue reading (and storing) the remaining of the VOTable (assuming
  /// it will not contains another table).
  pub(crate) fn read_from_data_end_to_end<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
  ) -> Result<(), VOTableError> {
    if let Some(last_resource) = self.resources.last_mut() {
      let data_found = last_resource.read_from_data_end_to_end(reader, reader_buff)?;
      if data_found {
        self.read_sub_elements_by_ref(reader, reader_buff, &())?;
      }
    }
    Ok(())
  }

  /// Returns `true` if a table has been found, else the full content of the VOTable is written.
  /// Write the VOTable from the beginning till:
  /// * either the first `DATA` tag si found (without writing the `<DATA>` tag).
  /// * or the end of the first `TABLE` tag is found (without writing the `<TABLE>` tag)
  pub fn write_to_data_beginning<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
    stop_before_data: bool,
  ) -> Result<bool, VOTableError> {
    writer
      .write(
        r#"<?xml version="1.0" encoding="UTF-8"?>
"#
        .as_bytes(),
      )
      .map_err(VOTableError::Write)?;
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    self.for_each_attribute(|k, v| tag.push_attribute((k, v)));
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elems
    write_elem!(self, description, writer, context);
    write_elem!(self, definitions, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    for resource in self.resources.iter_mut() {
      if resource.write_to_data_beginning(writer, &(), stop_before_data)? {
        return Ok(true);
      }
    }
    // Reach this point only if no table has been found
    write_elem_vec!(self, post_infos, writer, context);
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
      .map(|()| false)
  }

  pub fn write_from_data_end<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &(),
    start_after_data: bool,
  ) -> Result<(), VOTableError> {
    let mut write = false;
    for resource in self.resources.iter_mut() {
      if write {
        resource.write(writer, context)?;
      } else {
        write = resource.write_from_data_end(writer, &(), start_after_data)?;
      }
    }
    if write {
      write_elem_vec!(self, post_infos, writer, context);
      // Close tag
      let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    } else {
      Ok(())
    }
  }
}

fn check_declaration(decl: &BytesDecl) {
  let version = decl
    .version()
    .map(|v| unsafe { String::from_utf8_unchecked(v.as_ref().to_vec()) })
    .unwrap_or_else(|e| format!("Error: {:?}", e));
  let encoding = decl
    .encoding()
    .map(|r| {
      r.map(|r| unsafe { String::from_utf8_unchecked(r.as_ref().to_vec()) })
        .unwrap_or_else(|e| format!("Error: {:?}", e))
    })
    .unwrap_or_else(|| String::from("error"));
  let standalone = decl
    .standalone()
    .map(|r| {
      r.map(|r| unsafe { String::from_utf8_unchecked(r.as_ref().to_vec()) })
        .unwrap_or_else(|e| format!("Error: {:?}", e))
    })
    .unwrap_or_else(|| String::from("error"));
  debug!(
    "XML declaration. Version: {}; Encoding: {}; Standalone: {}.",
    version, encoding, standalone
  );
}

impl<C: TableDataContent> VOTableElement for VOTable<C> {
  const TAG: &'static str = "VOTABLE";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new_empty(Version::V1_4).set_attrs(attrs)
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
        "version" => self.set_version_by_ref(val.as_ref().parse().map_err(VOTableError::Custom)?),
        "xmlns" => self.set_xmlns_by_ref(val),
        "xmlns:xsi" => self.set_xmlns_xsi_by_ref(val),
        "xsi:schemaLocation" => self.set_xsi_schema_location_by_ref(val),
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
    f("version", (&self.version).into());
    f("xmlns", self.xmlns.as_str());
    if let Some(xmlns_xsi) = &self.xmlns_xsi {
      f("xmlns:xsi", xmlns_xsi.as_str());
    }
    if let Some(xsi_schema_location) = &self.xsi_schema_location {
      f("xsi:schemaLocation", xsi_schema_location.as_str());
    }
    for_each_extra_attribute!(self, f);
  }
}

impl<C: TableDataContent> HasSubElements for VOTable<C> {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    // Must contain at least one resource
    false
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
          Description::TAG_BYTES => set_desc_from_event_start!(self, reader, reader_buff, e),
          Definitions::TAG_BYTES => {
            set_from_event_start!(self, Definitions, reader, reader_buff, e)
          }
          Info::TAG_BYTES if self.resources.is_empty() => {
            push_from_event_start!(self, Info, reader, reader_buff, e)
          }
          Group::TAG_BYTES => push_from_event_start!(self, Group, reader, reader_buff, e),
          Param::TAG_BYTES => push_from_event_start!(self, Param, reader, reader_buff, e),
          Resource::<C>::TAG_BYTES => {
            self.push_resource_by_ref(from_event_start_by_ref!(Resource, reader, reader_buff, e))
          }
          Info::TAG_BYTES => {
            self.push_post_info_by_ref(from_event_start_by_ref!(Info, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Definitions::TAG_BYTES => set_from_event_empty!(self, Definitions, e),
          Info::TAG_BYTES => {
            let info = Info::from_event_empty(e)?;
            if self.resources.is_empty() {
              self.push_info_by_ref(info);
            } else {
              self.push_post_info_by_ref(info);
            }
          }
          CooSys::TAG_BYTES => push_from_event_empty!(self, CooSys, e),
          TimeSys::TAG_BYTES => push_from_event_empty!(self, TimeSys, e),
          Group::TAG_BYTES => push_from_event_empty!(self, Group, e),
          Param::TAG_BYTES => push_from_event_empty!(self, Param, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          return if self.resources.is_empty() {
            Err(VOTableError::Custom(String::from(
              "No resource found in the VOTable",
            )))
          } else {
            Ok(())
          }
        }
        Event::Text(e) if is_empty(e) => {}
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
    write_elem!(self, definitions, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, resources, writer, context);
    write_elem_vec!(self, post_infos, writer, context);
    Ok(())
  }
}

/*
impl<C: TableDataContent> QuickXmlReadWrite<HasSubElems> for VOTable<C> {
  type Context = ();

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    // We do not use the 'default' write impleemntation because of this first line.

    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    self.for_each_attribute(|k, v| tag.push_attribute((k, v)));
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elems
    self.write_sub_elements_by_ref(writer, context)?;
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}*/

#[cfg(test)]
mod tests {
  use crate::votable::VOTableWrapper;
  use crate::{
    impls::mem::{InMemTableDataRows, InMemTableDataStringRows},
    QuickXmlReadWrite,
  };
  use quick_xml::Writer;

  #[test]
  fn test_votable_read_from_str() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<VOTABLE version="1.4" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xmlns="http://www.ivoa.net/xml/VOTable/v1.3"
  xsi:schemaLocation="http://www.ivoa.net/xml/VOTable/v1.3 http://www.ivoa.net/xml/VOTable/v1.3">
    <DESCRIPTION>
   VizieR Astronomical Server vizier.u-strasbg.fr
    Date: 2022-04-19T13:38:24 [V1.99+ (14-Oct-2013)]
   Explanations and Statistics of UCDs:			See LINK below
   In case of problem, please report to:	cds-question@unistra.fr
   In this version, NULL integer columns are written as an empty string
   &lt;TD&gt;&lt;/TD&gt;, explicitely possible from VOTable-1.3
    </DESCRIPTION>
    <RESOURCE>
    </RESOURCE>
</VOTABLE>"#;
    let votable = VOTableWrapper::<InMemTableDataStringRows>::from_ivoa_xml_str(xml)
      .unwrap()
      .unwrap();
    assert!(votable.description.is_some())
  }

  #[test]
  fn test_votable_read_datatable_from_file() {
    // let votable =  VOTable::<InMemTableDataStringRows>::from_file("resources/sdss12.vot").unwrap();
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/sdss12.vot")
      .unwrap()
      .unwrap();
    match serde_json::ser::to_string_pretty(&votable) {
      Ok(_content) => println!("\nOK"), // println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false)
      }
    }
    match toml::ser::to_string_pretty(&votable) {
      Ok(_content) => println!("\nOK"), // println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false)
      }
    }

    /*println!("\n\n#### XML ####\n");

    match quick_xml::se::to_string(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => println!("{:?}", &error),
    }*/
  }

  #[test]
  fn test_votable_read_with_namespace_file() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file(
      "resources/IMCCE.with_namespace.vot",
    );
    assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_obscore_file() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/obscore.vot");
    assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_with_cdata() {
    let votable =
      VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/vot_with_cdata.vot");
    println!("{:?}", votable);
    assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_with_definitions_file() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file(
      "resources/vot_with_definitions.vot",
    );
    assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_with_empty_precision() {
    let votable =
      VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/vot_with_empty_prec.vot");
    assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_datalink_003_file() {
    match VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/dataLink_003.xml") {
      Ok(_) => {}
      Err(e) => {
        eprintln!("Error: {:?}", e);
        assert!(false);
      }
    }
  }

  #[test]
  fn test_votable_read_simbad_unicodechar_file() {
    match VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file(
      "resources/simbad_unicodechar.xml",
    ) {
      Ok(votable_wrapper) => match serde_json::ser::to_string_pretty(&votable_wrapper.unwrap()) {
        Ok(content) => println!("{}", &content),
        Err(error) => {
          println!("{:?}", &error);
          assert!(false)
        }
      },
      Err(e) => {
        eprintln!("Error: {:?}", e);
        assert!(false);
      }
    }
  }

  #[test]
  fn test_votable_read_iter_datatable_from_file() {
    use crate::iter::{TableIter, VOTableIterator};

    /*
    println!();
    println!("-- next_table_row_string_iter dss12.vot --");
    println!();

    let mut table_it = TableIterator::from_file("resources/sdss12.vot").unwrap();
    while let Some(row_it) = table_it.next_table_row_string_iter().unwrap() {
      for (i, row) in row_it.enumerate() {
        println!("Row {}: {:?}", i, row);
      }
    }*/

    println!();
    println!("-- next_table_row_value_iter dss12.vot --");
    println!();

    let mut votable_it = VOTableIterator::from_file("resources/sdss12.vot").unwrap();
    while let Some(mut row_it) = votable_it.next_table_row_value_iter().unwrap() {
      let table_ref_mut = row_it.table();
      println!("Fields: {:?}", table_ref_mut.elems);
      for (i, row) in row_it.enumerate() {
        println!("Row {}: {:?}", i, row);
      }
    }
    let votable = votable_it.end_of_it();
    println!("VOTable: {:?}", votable);

    println!();
    println!("-- next_table_row_value_iter binary.b64 --");
    println!();

    let mut table_it = VOTableIterator::from_file("resources/binary.b64").unwrap();
    while let Some(row_it) = table_it.next_table_row_value_iter().unwrap() {
      for (i, row) in row_it.enumerate() {
        println!("Row {}: {:?}", i, row);
      }
    }

    println!();
    println!("-- next_table_row_value_iter gaia_dr3.b264 --");
    println!();

    let mut table_it = VOTableIterator::from_file("resources/gaia_dr3.b264").unwrap();
    while let Some(row_it) = table_it.next_table_row_value_iter().unwrap() {
      for (i, row) in row_it.enumerate() {
        println!("Row {}: {:?}", i, row);
      }
    }

    assert!(true)
  }

  #[test]
  fn test_votable_read_binary_from_file() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/binary.b64")
      .unwrap()
      .unwrap();
    match toml::ser::to_string_pretty(&votable) {
      Ok(_content) => println!("\nOK"), // println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }
  }

  #[test]
  fn test_votable_read_binary2_from_file() {
    let votable =
      VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/gaia_dr3.b264")
        .unwrap()
        .unwrap();
    let mut votable = votable.wrap();
    match serde_json::ser::to_string_pretty(&votable) {
      Ok(_content) => println!("\nOK"), //println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }
    // let mut votable = votable.unwrap();
    // if true { return; }

    match toml::ser::to_string_pretty(&votable) {
      Ok(_content) => println!("\nOK"), // println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }

    println!("\n\n#### VOTABLE ####\n");
    let mut votable2: Vec<u8> = Vec::new();
    let mut write = Writer::new_with_indent(/*stdout()*/ &mut votable2, b' ', 4);
    match votable.votable.write(&mut write, &()) {
      Ok(_content) => {
        println!("\nOK")
      }
      Err(error) => println!("Error: {:?}", &error),
    }

    let votable2 = String::from_utf8(votable2).unwrap();
    println!("{}", &votable2);

    let votable3 =
      VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_str(votable2.as_str()).unwrap();
    match toml::ser::to_string_pretty(&votable3) {
      Ok(_content) => println!("\nOK"), // println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }
  }

  #[cfg(feature = "mivot")]
  #[test]
  fn test_votable_read_mivot_from_file() {
    let votable_res_res =
      VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/mivot_appendix_A.xml");
    match votable_res_res {
      Ok(votable_res) => {
        let mut votable = votable_res.unwrap();

        println!("\n\n#### JSON ####\n");

        match serde_json::ser::to_string_pretty(&votable) {
          Ok(content) => {
            println!("{}", &content);
            println!("\nOK");
          } //,
          Err(error) => {
            println!("{:?}", &error);
            assert!(false);
          }
        }

        println!("\n\n#### TOML ####\n");

        match toml::ser::to_string_pretty(&votable) {
          Ok(content) => {
            println!("{}", &content);
            println!("\nOK")
          }
          Err(error) => {
            println!("{:?}", &error);
            assert!(false);
          }
        }

        println!("\n\n#### XML ####\n");

        let mut votable2: Vec<u8> = Vec::new();
        let mut write = Writer::new_with_indent(&mut votable2, b' ', 4);
        match votable.write(&mut write, &()) {
          Ok(()) => {
            let votable2 = String::from_utf8(votable2).unwrap();
            println!("{}", votable2);
            println!("\nOK")
          }
          Err(error) => {
            println!("{:?}", &error);
            assert!(false);
          }
        }
      }
      Err(e) => {
        eprintln!("Error: {}", e.to_string());
        assert!(false);
      }
    }
  }
}
