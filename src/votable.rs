use std::{
  fs::File,
  path::Path,
  str::{self, FromStr},
  io::{BufRead, Write, BufReader, BufWriter},
  collections::HashMap,
};

use quick_xml::{
  Reader, Writer,
  events::{
    BytesStart, Event,
    attributes::Attributes,
  },
};

use paste::paste;
use quick_xml::events::BytesDecl;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::is_empty;

use super::{
  QuickXmlReadWrite, TableDataContent,
  error::VOTableError,
  coosys::CooSys,
  desc::Description,
  definitions::Definitions,
  group::Group,
  info::Info,
  param::Param,
  resource::Resource,
  timesys::TimeSys,
};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
      _ => Err(format!("Unrecognized version. Actual: '{}'. Expected: '1.1', '1.2', '1.3' or '1.4'", s)),
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
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum VOTableElem {
  CooSys(CooSys),
  TimeSys(TimeSys),
  Group(Group),
  Param(Param),
  Info(Info),
  Definitions(Definitions), // Deprecated since v1.1
}

impl VOTableElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      VOTableElem::CooSys(elem) => elem.write(writer, &()),
      VOTableElem::TimeSys(elem) => elem.write(writer, &()),
      VOTableElem::Group(elem) => elem.write(writer, &()),
      VOTableElem::Param(elem) => elem.write(writer, &()),
      VOTableElem::Info(elem) => elem.write(writer, &()),
      VOTableElem::Definitions(elem) => elem.write(writer, &()),
    }
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VOTableWrapper<C: TableDataContent> {
  votable: VOTable<C>,
}

impl<C: TableDataContent> VOTableWrapper<C> {
  /// Returns the inner `VOTable` element
  pub fn unwrap(self) -> VOTable<C> {
    self.votable
  }

  // Manual parser

  pub fn manual_from_ivoa_xml_file<P: AsRef<Path>>(path: P, reader_buff: &mut Vec<u8>)
    -> Result<(VOTable<C>, Resource<C>, Reader<BufReader<File>>), VOTableError> {
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
    self.votable.write(&mut write, &())
  }
}


impl<C> VOTableWrapper<C>
  where
    C: TableDataContent + Serialize + for<'a> Deserialize<'a>
{
  // JSON

  pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_json_reader(reader)
  }

  pub fn from_json_str(s: &str) -> Result<Self, VOTableError> {
    serde_json::from_str(s).map_err(VOTableError::Json)
  }

  pub fn from_json_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    serde_json::from_slice(s).map_err(VOTableError::Json)
  }

  pub fn from_json_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    serde_json::from_reader(reader).map_err(VOTableError::Json)
  }

  pub fn to_json_file<P: AsRef<Path>>(&mut self, path: P, pretty: bool) -> Result<(), VOTableError> {
    let file = File::create(path).map_err(VOTableError::Io)?;
    let write = BufWriter::new(file);
    self.to_json_writer(write, pretty)
  }

  pub fn to_json_string(&mut self, pretty: bool) -> Result<String, VOTableError> {
    if pretty {
      serde_json::ser::to_string_pretty(&self)
    } else {
      serde_json::ser::to_string(&self)
    }.map_err(VOTableError::Json)
  }

  pub fn to_json_bytes(&mut self, pretty: bool) -> Result<Vec<u8>, VOTableError> {
    if pretty {
      serde_json::ser::to_vec_pretty(&self)
    } else {
      serde_json::ser::to_vec(&self)
    }.map_err(VOTableError::Json)
  }

  pub fn to_json_writer<W: Write>(&mut self, write: W, pretty: bool) -> Result<(), VOTableError> {
    if pretty {
      serde_json::ser::to_writer_pretty(write, &self)
    } else {
      serde_json::ser::to_writer(write, &self)
    }.map_err(VOTableError::Json)
  }

  // YAML

  pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_yaml_reader(reader)
  }

  pub fn from_yaml_str(s: &str) -> Result<Self, VOTableError> {
    serde_yaml::from_str(s).map_err(VOTableError::Yaml)
  }

  pub fn from_yaml_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    serde_yaml::from_slice(s).map_err(VOTableError::Yaml)
  }

  pub fn from_yaml_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    serde_yaml::from_reader(reader).map_err(VOTableError::Yaml)
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
    serde_yaml::to_string(&self).map(|s| s.into()).map_err(VOTableError::Yaml)
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
    toml::from_str(s).map_err(VOTableError::TomlDe)
  }

  pub fn from_toml_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    toml::from_slice(s).map_err(VOTableError::TomlDe)
  }

  pub fn from_toml_reader<R: BufRead>(mut reader: R) -> Result<Self, VOTableError> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data).map_err(VOTableError::Io)?;
    Self::from_toml_bytes(data.as_slice())
  }

  pub fn to_toml_file<P: AsRef<Path>>(&mut self, path: P, pretty: bool) -> Result<(), VOTableError> {
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
    }.map_err(VOTableError::TomlSer)
  }

  pub fn to_toml_bytes(&mut self, pretty: bool) -> Result<Vec<u8>, VOTableError> {
    // toml::ser::to_vec(&self).map_err(VOTableError::TomlSer)
    self.to_toml_string(pretty).map(|s| s.into_bytes())
  }

  pub fn to_toml_writer<W: Write>(&mut self, mut write: W, pretty: bool) -> Result<(), VOTableError> {
    let bytes = self.to_toml_bytes(pretty)?;
    write.write_all(bytes.as_slice()).map_err(VOTableError::Io)
  }
}


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct VOTable<C: TableDataContent> {
  // attributes
  #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub version: Option<Version>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<VOTableElem>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub resources: Vec<Resource<C>>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub post_infos: Vec<Info>,
}
/* A ajouter?!
xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
xmlns="http://www.ivoa.net/xml/VOTable/v1.3"
xsi:schemaLocation="http://www.ivoa.net/xml/VOTable/v1.3 http://www.ivoa.net/xml/VOTable/v1.3"
*/

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
  fn new_empty() -> Self {
    Self {
      id: None,
      version: None,
      extra: Default::default(),
      description: None,
      elems: Default::default(),
      resources: Default::default(),
      post_infos: Default::default(),
    }
  }

  pub fn new(resource: Resource<C>) -> Self {
    let votable = Self::new_empty();
    votable.push_resource(resource)
  }

  /*pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError>  {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_reader(reader)
  }

  pub fn from_str(s: &str) -> Result<Self, VOTableError> {
    Self::from_reader(s.as_bytes())
  }

  pub fn from_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    Self::from_reader(s)
  }*/

  pub(crate) fn from_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    let mut reader = Reader::from_reader(reader);
    let mut buff: Vec<u8> = Vec::with_capacity(1024);
    loop {
      let mut event = reader.read_event(&mut buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Decl(ref e) => check_declaration(e),
        Event::Start(ref mut e) if e.local_name() == VOTable::<C>::TAG_BYTES => {
          let mut votable = VOTable::<C>::from_attributes(e.attributes())?;
          votable.read_sub_elements_and_clean(reader, &mut buff, &())?;
          // ignore the remaining of the reader !
          return Ok(votable);
        }
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  pub(crate) fn from_reader_till_next_resource<R: BufRead>(reader: R, mut reader_buff: &mut Vec<u8>)
    -> Result<(VOTable<C>, Resource<C>, Reader<R>), VOTableError> {
    let mut reader = Reader::from_reader(reader);
    loop {
      let mut event = reader.read_event(&mut reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Decl(ref e) => check_declaration(e),
        Event::Start(ref mut e) if e.local_name() == VOTable::<C>::TAG_BYTES => {
          let mut votable = VOTable::<C>::from_attributes(e.attributes())?;
          let (resource, reader) = votable.read_till_next_resource(reader, &mut reader_buff)?;
          reader_buff.clear();
          return Ok((votable, resource, reader));
        }
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  impl_builder_opt_string_attr!(id);

  impl_builder_opt_attr!(version, Version);

  impl_builder_insert_extra!();

  impl_builder_opt_attr!(description, Description);

  impl_builder_push_elem!(CooSys, VOTableElem);
  impl_builder_push_elem!(TimeSys, VOTableElem);
  impl_builder_push_elem!(Group, VOTableElem);
  impl_builder_push_elem!(Param, VOTableElem);
  impl_builder_push_elem!(Info, VOTableElem);
  impl_builder_push_elem!(Definitions, VOTableElem);

  impl_builder_push!(Resource, C);

  impl_builder_push_post_info!();

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
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES =>
              from_event_start_desc_by_ref!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.resources.is_empty() =>
              self.elems.push(VOTableElem::Info(from_event_start_by_ref!(Info, reader, reader_buff, e))),
            Group::TAG_BYTES =>
              self.elems.push(VOTableElem::Group(from_event_start_by_ref!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES =>
              self.elems.push(VOTableElem::Param(from_event_start_by_ref!(Param, reader, reader_buff, e))),
            Resource::<C>::TAG_BYTES => {
              let resource = Resource::<C>::from_attributes(e.attributes())?;
              return Ok(Some(resource));
            }
            Info::TAG_BYTES =>
              self.post_infos.push(from_event_start_by_ref!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES if self.resources.is_empty() => self.elems.push(VOTableElem::Info(Info::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(VOTableElem::CooSys(CooSys::from_event_empty(e)?)),
            TimeSys::TAG_BYTES => self.elems.push(VOTableElem::TimeSys(TimeSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(VOTableElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(VOTableElem::Param(Param::from_event_empty(e)?)),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.local_name() == Self::TAG_BYTES =>
          return Ok(None),
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }
  
  // * The Resource returned has only its attribute sets, not the sub-elements
  // * The VOTable.push_resource it to be done externally
  fn read_till_next_resource<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    mut reader_buff: &mut Vec<u8>,
  ) -> Result<(Resource<C>, Reader<R>), VOTableError> {
    // If the full document is in memory, we could have use a Reader<'a [u8]> and then the method
    // `read_event_unbuffered` to avoid a copy.
    // But are more generic that this to be able to read in streaming mode
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => {
          match e.local_name() {
            Description::TAG_BYTES =>
              from_event_start_desc!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.resources.is_empty() =>
              self.elems.push(VOTableElem::Info(from_event_start!(Info, reader, reader_buff, e))),
            Group::TAG_BYTES =>
              self.elems.push(VOTableElem::Group(from_event_start!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES =>
              self.elems.push(VOTableElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            Definitions::TAG_BYTES =>
              self.elems.push(VOTableElem::Definitions(from_event_start!(Definitions, reader, reader_buff, e))),
            Resource::<C>::TAG_BYTES => {
              let resource = Resource::<C>::from_attributes(e.attributes())?;
              return Ok((resource, reader));
            }
            Info::TAG_BYTES =>
              self.post_infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES if self.resources.is_empty() => self.elems.push(VOTableElem::Info(Info::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(VOTableElem::CooSys(CooSys::from_event_empty(e)?)),
            TimeSys::TAG_BYTES => self.elems.push(VOTableElem::TimeSys(TimeSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(VOTableElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(VOTableElem::Param(Param::from_event_empty(e)?)),
            Definitions::TAG_BYTES => self.elems.push(VOTableElem::Definitions(Definitions::from_event_empty(e)?)),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.local_name() == Self::TAG_BYTES =>
          return Err(VOTableError::Custom(String::from("No resource found in the VOTable"))),
        Event::Text(e) if is_empty(e) => {}
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }
}

fn check_declaration(decl: &BytesDecl) {
  let version = decl.version().map(|v|
    unsafe { String::from_utf8_unchecked(v.as_ref().to_vec()) }
  ).unwrap_or_else(|e| format!("Error: {:?}", e));
  let encoding = decl.encoding().map(|r|
    r.map(|r| unsafe { String::from_utf8_unchecked(r.as_ref().to_vec()) }).unwrap_or_else(|e| format!("Error: {:?}", e))
  ).unwrap_or_else(|| String::from("error"));
  let standalone = decl.standalone().map(|r|
    r.map(|r| unsafe { String::from_utf8_unchecked(r.as_ref().to_vec()) }).unwrap_or_else(|e| format!("Error: {:?}", e))
  ).unwrap_or_else(|| String::from("error"));
  eprintln!("XML declaration. Version: {}; Encoding: {}; Standalone: {}.", version, encoding, standalone);
}


impl<C: TableDataContent> QuickXmlReadWrite for VOTable<C> {
  const TAG: &'static str = "VOTABLE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut votable = Self::new_empty();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      votable = match attr.key {
        b"ID" => votable.set_id(value),
        b"version" => votable.set_version(Version::from_str(value).map_err(VOTableError::Custom)?),
        _ => votable.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }
    Ok(votable)
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
            Description::TAG_BYTES =>
              from_event_start_desc!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.resources.is_empty() =>
              self.elems.push(VOTableElem::Info(from_event_start!(Info, reader, reader_buff, e))),
            Group::TAG_BYTES =>
              self.elems.push(VOTableElem::Group(from_event_start!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES =>
              self.elems.push(VOTableElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            Definitions::TAG_BYTES =>
              self.elems.push(VOTableElem::Definitions(from_event_start!(Definitions, reader, reader_buff, e))),
            Resource::<C>::TAG_BYTES =>
              self.resources.push(from_event_start!(Resource, reader, reader_buff, e)),
            Info::TAG_BYTES =>
              self.post_infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.local_name() {
            Info::TAG_BYTES if self.resources.is_empty() => self.elems.push(VOTableElem::Info(Info::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(VOTableElem::CooSys(CooSys::from_event_empty(e)?)),
            TimeSys::TAG_BYTES => self.elems.push(VOTableElem::TimeSys(TimeSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(VOTableElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(VOTableElem::Param(Param::from_event_empty(e)?)),
            Definitions::TAG_BYTES => self.elems.push(VOTableElem::Definitions(Definitions::from_event_empty(e)?)),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.local_name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.local_name() == Self::TAG_BYTES =>
          return if self.resources.is_empty() {
            Err(VOTableError::Custom(String::from("No resource found in the VOTable")))
          } else {
            Ok(reader)
          },
        /*
        Event::Text(_) => {}
        Event::Comment(_) => {}
        Event::CData(_) => {}
        Event::Decl(_) => {}
        Event::PI(_) => {}
        Event::DocType(_) => {}
        */
        Event::Text(e) if is_empty(e) => {}
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
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    writer.write(r#"<?xml version="1.0" encoding="UTF-8"?>
"#.as_bytes()).map_err(VOTableError::Write)?;
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, ID);
    push2write_opt_into_attr!(self, tag, version);
    push2write_extra!(self, tag);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-elems
    write_elem!(self, description, writer, context);
    write_elem_vec_no_context!(self, elems, writer);
    write_elem_vec!(self, resources, writer, context);
    write_elem_vec!(self, post_infos, writer, context);
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}


#[cfg(test)]
mod tests {
  use quick_xml::Writer;
  use crate::{
    QuickXmlReadWrite,
    impls::mem::{InMemTableDataStringRows, InMemTableDataRows},
  };
  use crate::votable::VOTableWrapper;

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
    let votable = VOTableWrapper::<InMemTableDataStringRows>::from_ivoa_xml_str(xml).unwrap().unwrap();
    assert!(votable.description.is_some())
  }

  #[test]
  fn test_votable_read_datatable_from_file() {
    // let votable =  VOTable::<InMemTableDataStringRows>::from_file("resources/sdss12.vot").unwrap();
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/sdss12.vot").unwrap().unwrap();
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
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/IMCCE.with_namespace.vot");
    assert!(votable.is_ok())
  }
  
  #[test]
  fn test_votable_read_obscore_file() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/obscore.vot");
    assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_with_cdata() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/vot_with_cdata.vot");
    assert!(votable.is_ok())
  }
  
  #[test]
  fn test_votable_read_with_definitions_file() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/vot_with_definitions.vot");
    assert!(votable.is_ok())
  }


  #[test]
  fn test_votable_read_with_empty_precision() {
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/vot_with_empty_prec.vot");
    assert!(votable.is_ok())
  }
  
  #[test]
  fn test_votable_read_dataLink_003_file() {
    match VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/dataLink_003.xml") {
      Ok(_) => { },
      Err(e) => {
        eprintln!("Error: {:?}", e);
        assert!(false);
      },
    }
    // assert!(votable.is_ok())
  }

  #[test]
  fn test_votable_read_iter_datatable_from_file() {
    use crate::iter::VOTableIterator;
    
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
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/binary.b64").unwrap().unwrap();
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
    let votable = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_file("resources/gaia_dr3.b264").unwrap().unwrap();
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

    let votable3 = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_str(votable2.as_str()).unwrap();
    match toml::ser::to_string_pretty(&votable3) {
      Ok(_content) => println!("\nOK"), // println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }
  }
}