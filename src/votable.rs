use std::{
  str::{self, FromStr},
  io::{BufRead, Write},
  collections::HashMap,
};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

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
  error::VOTableError,
  coosys::CooSys,
  desc::Description,
  group::Group,
  info::Info,
  param::Param,
  resource::Resource,
  timesys::TimeSys,
};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Version {
  V1_3,
  V1_4,
}

impl FromStr for Version {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "1.3" => Ok(Version::V1_3),
      "1.4" => Ok(Version::V1_4),
      _ => Err(format!("Unrecognized version. Actual: '{}'. Expected: '1.3' or '1.4'", s)),
    }
  }
}

impl From<&Version> for &'static str {
  fn from(version: &Version) -> Self {
    match version {
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
}

impl VOTableElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      VOTableElem::CooSys(elem) => elem.write(writer),
      VOTableElem::TimeSys(elem) => elem.write(writer),
      VOTableElem::Group(elem) => elem.write(writer),
      VOTableElem::Param(elem) => elem.write(writer),
      VOTableElem::Info(elem) => elem.write(writer),
    }
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

  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, VOTableError>  {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    Self::from_reader(reader)
  }

  pub fn from_str(s: &str) -> Result<Self, VOTableError> {
    Self::from_reader(s.as_bytes())
  }

  pub fn from_bytes(s: &[u8]) -> Result<Self, VOTableError> {
    Self::from_reader(s)
  }
  
  pub fn from_reader<R: BufRead>(reader: R) -> Result<Self, VOTableError> {
    let mut reader = Reader::from_reader(reader);
    let mut buff: Vec<u8> = Vec::with_capacity(1024);
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Decl(ref e) => 
          eprintln!("XML declaration. Version: {}; Encoding: {}; Standalone: {}.", 
            e.version().map(|v| 
              unsafe { String::from_utf8_unchecked(v.as_ref().to_vec()) }
            ).unwrap_or_else(|e| format!("Error: {:?}", e)),
            e.encoding().map(|r| 
              r.map(|r| unsafe { String::from_utf8_unchecked(r.as_ref().to_vec()) }).unwrap_or_else(|e| format!("Error: {:?}", e))
            ).unwrap_or_else(|| String::from("error")),
            e.standalone().map(|r| 
              r.map(|r| unsafe { String::from_utf8_unchecked(r.as_ref().to_vec()) }).unwrap_or_else(|e| format!("Error: {:?}", e))
            ).unwrap_or_else(|| String::from("error")),
          ),
        Event::Start(ref mut e) if e.name() == VOTable::<C>::TAG_BYTES => {
          let mut votable = VOTable::<C>::from_attributes(e.attributes()).unwrap();
          votable.read_sub_elements_and_clean(reader, &mut buff, &())?;
          // ignore the remaining of the reader !
          return Ok(votable);
        }
        Event::Text(e) if is_empty(e) => { },
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

  impl_builder_push!(Resource, C);

  impl_builder_push_post_info!();
}

impl<C: TableDataContent> QuickXmlReadWrite for VOTable<C> {

  const TAG: &'static str = "VOTABLE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut votable = Self::new_empty();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
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
          match e.name() {
            Description::TAG_BYTES => 
              from_event_start_desc!(self, Description, reader, reader_buff, e),
            Info::TAG_BYTES if self.resources.is_empty() => 
              self.elems.push(VOTableElem::Info(from_event_start!(Info, reader, reader_buff, e))),
            Group::TAG_BYTES => 
              self.elems.push(VOTableElem::Group(from_event_start!(Group, reader, reader_buff, e))),
            Param::TAG_BYTES =>
              self.elems.push(VOTableElem::Param(from_event_start!(Param, reader, reader_buff, e))),
            Resource::<C>::TAG_BYTES => 
              self.resources.push(from_event_start!(Resource, reader, reader_buff, e)),
            Info::TAG_BYTES => 
              self.post_infos.push(from_event_start!(Info, reader, reader_buff, e)),
            _ => return Err(VOTableError::UnexpectedStartTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::Empty(ref e) => {
          match e.name() {
            Info::TAG_BYTES if self.resources.is_empty() => self.elems.push(VOTableElem::Info(Info::from_event_empty(e)?)),
            CooSys::TAG_BYTES => self.elems.push(VOTableElem::CooSys(CooSys::from_event_empty(e)?)),
            TimeSys::TAG_BYTES => self.elems.push(VOTableElem::TimeSys(TimeSys::from_event_empty(e)?)),
            Group::TAG_BYTES => self.elems.push(VOTableElem::Group(Group::from_event_empty(e)?)),
            Param::TAG_BYTES => self.elems.push(VOTableElem::Param(Param::from_event_empty(e)?)),
            Info::TAG_BYTES => self.post_infos.push(Info::from_event_empty(e)?),
            _ => return Err(VOTableError::UnexpectedEmptyTag(e.name().to_vec(), Self::TAG)),
          }
        }
        Event::End(e) if e.name() == Self::TAG_BYTES => 
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
        Event::Text(e) if is_empty(e) => { },
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
      }
    }
  }

  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    writer.write(r#"<?xml version="1.0" encoding="UTF-8"?>
"#.as_bytes()).map_err(VOTableError::Write)?;
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    push2write_opt_string_attr!(self, tag, ID);
    push2write_opt_into_attr!(self, tag, version);
    push2write_extra!(self, tag);
    writer.write_event(Event::Start(tag.to_borrowed())).map_err(VOTableError::Write)?;
    // Write sub-elems
    write_elem!(self, description, writer);
    write_elem_vec!(self, elems, writer);
    write_elem_vec!(self, resources, writer);
    write_elem_vec!(self, post_infos, writer);
    // Close tag
    writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
  }
}


#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{
    Reader, Writer,
    events::Event
  };
  use crate::{
    QuickXmlReadWrite,
    votable::VOTable,
    impls::mem::{InMemTableDataStringRows, InMemTableDataRows}
  };

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
    let votable =  VOTable::<InMemTableDataStringRows>::from_str(xml).unwrap();
    assert!(votable.description.is_some())
  }

  #[test]
  fn test_votable_read_datatable_from_file() {
    // let votable =  VOTable::<InMemTableDataStringRows>::from_file("resources/sdss12.vot").unwrap();
    let votable =  VOTable::<InMemTableDataRows>::from_file("resources/sdss12.vot").unwrap();
    match serde_json::ser::to_string_pretty(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false)
      },
    }
    match toml::ser::to_string_pretty(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false)
      },
    }

    /*println!("\n\n#### XML ####\n");

    match quick_xml::se::to_string(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => println!("{:?}", &error),
    }*/
  }

  #[test]
  fn test_votable_read_binary_from_file() {
    let votable =  VOTable::<InMemTableDataRows>::from_file("resources/binary.b64").unwrap();
    match toml::ser::to_string_pretty(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      },
    }
  }

  #[test]
  fn test_votable_read_binary2_from_file() {
    let votable =  VOTable::<InMemTableDataRows>::from_file("resources/gaia_dr3.b264").unwrap();
    match toml::ser::to_string_pretty(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      },
    }
  }
}