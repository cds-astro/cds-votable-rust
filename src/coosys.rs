//! Module dedicated to the `COOSYS` tag.

use std::{
  fmt::{self, Display, Formatter},
  io::{BufRead, Write},
  num::ParseFloatError,
  str::{self, FromStr},
};

use log::warn;
use paste::paste;
use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  ElementWriter, Reader, Writer,
};
// use serde;

use crate::{
  error::VOTableError,
  fieldref::FieldRef,
  paramref::ParamRef,
  timesys::RefPosition,
  utils::{discard_comment, discard_event},
  QuickXmlReadWrite, TableDataContent, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "elem_type")]
pub enum CooSysElem {
  FieldRef(FieldRef),
  ParamRef(ParamRef),
}

impl CooSysElem {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError> {
    match self {
      CooSysElem::FieldRef(elem) => elem.write(writer, &()),
      CooSysElem::ParamRef(elem) => elem.write(writer, &()),
    }
  }
  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    match self {
      CooSysElem::FieldRef(e) => e.visit(visitor),
      CooSysElem::ParamRef(e) => e.visit(visitor),
    }
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CooSys {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(flatten)]
  pub coosys: System,
  /// We so far put `refposition` as Optional to stay compatible with VOTable 1.4
  /// (and since it is not yet clear if `refposition` is mandatory or not).
  /// See [the IVOA doc](https://www.ivoa.net/documents/VOTable/20230913/WD-VOTable-1.5-20230913.html#elem:COOSYS).
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refposition: Option<RefPosition>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub elems: Vec<CooSysElem>,
}

impl CooSys {
  pub fn new<S: Into<String>>(id: S, coosys: System) -> Self {
    Self {
      id: id.into(),
      coosys,
      refposition: None,
      elems: Default::default(),
    }
  }

  /// Calls a closure on each (key, value) attribute pairs.
  pub fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("ID", self.id.as_str());
    self.coosys.for_each_attribute(&mut f);
    if let Some(refposition) = &self.refposition {
      f("refposition", refposition.to_string().as_str());
    }
  }

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_coosys_start(self)?;
    for elem in &mut self.elems {
      elem.visit(visitor)?;
    }
    visitor.visit_coosys_ended(self)
  }

  impl_builder_opt_attr!(refposition, RefPosition);
}

impl QuickXmlReadWrite for CooSys {
  const TAG: &'static str = "COOSYS";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut id: Option<String> = None;
    let mut system: Option<String> = None;
    let mut equinox: Option<String> = None;
    let mut epoch: Option<String> = None;
    let mut refposition: Option<RefPosition> = None;
    // Look for attributes
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value =
        String::from_utf8(attr.value.as_ref().to_vec()).map_err(VOTableError::FromUtf8)?;
      match attr.key {
        b"ID" => id = Some(value),
        b"system" => system = Some(value),
        b"equinox" => equinox = Some(value),
        b"epoch" => epoch = Some(value),
        b"refposition" => refposition = Some(value.parse().map_err(VOTableError::Variant)?),
        _ => {
          warn!(
            "Attribute {:?} in {} is ignored",
            std::str::from_utf8(attr.key),
            Self::TAG
          );
        }
      }
    }
    // Set from found attributes
    if let (Some(id), Some(system)) = (id, system) {
      let mut system = match system.as_str() {
        "eq_FK4" => {
          if let Some(equinox) = equinox {
            System::new_eq_fk4(
              equinox
                .parse::<BesselianYear>()
                .map_err(VOTableError::ParseYear)?
                .0,
            )
          } else {
            System::new_default_eq_fk4()
          }
        }
        "eq_FK5" => {
          if let Some(equinox) = equinox {
            System::new_eq_fk5(
              equinox
                .parse::<JulianYear>()
                .map_err(VOTableError::ParseYear)?
                .0,
            )
          } else {
            System::new_default_eq_fk5()
          }
        }
        "ICRS" => System::new_icrs(),
        "ecl_FK4" => {
          if let Some(equinox) = equinox {
            System::new_ecl_fk4(
              equinox
                .parse::<BesselianYear>()
                .map_err(VOTableError::ParseYear)?
                .0,
            )
          } else {
            System::new_default_ecl_fk4()
          }
        }
        "ecl_FK5" => {
          if let Some(equinox) = equinox {
            System::new_ecl_fk5(
              equinox
                .parse::<JulianYear>()
                .map_err(VOTableError::ParseYear)?
                .0,
            )
          } else {
            System::new_default_ecl_fk5()
          }
        }
        "galactic" => System::new_galactic(),
        "supergalactic" => System::new_supergalactic(),
        _ => {
          return Err(VOTableError::Custom(format!(
            "System '{}' not recognized in tag '{}'",
            system,
            Self::TAG
          )));
        }
      };
      if let Some(epoch) = epoch {
        system = system
          .set_epoch_from_str(epoch.as_str())
          .map_err(VOTableError::ParseYear)?;
      }
      // Create CooSys
      let mut coosys = CooSys::new(id, system);
      // Add refposition if any
      if let Some(refposition) = refposition {
        coosys = coosys.set_refposition(refposition);
      }
      Ok(coosys)
    } else {
      Err(VOTableError::Custom(format!(
        "Attributes 'ID' and 'system' are mandatory in tag '{}'",
        Self::TAG
      )))
    }
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
          FieldRef::TAG_BYTES => self
            .elems
            .push(CooSysElem::FieldRef(from_event_start_by_ref!(
              FieldRef,
              reader,
              reader_buff,
              e
            ))),
          ParamRef::TAG_BYTES => self
            .elems
            .push(CooSysElem::ParamRef(from_event_start_by_ref!(
              ParamRef,
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
          FieldRef::TAG_BYTES => self
            .elems
            .push(CooSysElem::FieldRef(FieldRef::from_event_empty(e)?)),
          ParamRef::TAG_BYTES => self
            .elems
            .push(CooSysElem::ParamRef(ParamRef::from_event_empty(e)?)),
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
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    if self.elems.is_empty() {
      let mut elem_writer = writer.create_element(Self::TAG_BYTES);
      elem_writer = elem_writer.with_attribute(("ID", self.id.as_str()));
      elem_writer = self.coosys.with_attributes(elem_writer);
      write_opt_tostring_attr!(self, elem_writer, refposition);
      elem_writer.write_empty().map_err(VOTableError::Write)?;
      Ok(())
    } else {
      let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
      // Write tag + attributes
      tag.push_attribute(("ID", self.id.as_str()));
      self.coosys.push_attributes(&mut tag);
      push2write_opt_tostring_attr!(self, tag, refposition);
      writer
        .write_event(Event::Start(tag.to_borrowed()))
        .map_err(VOTableError::Write)?;
      // Write sub-elems
      write_elem_vec_no_context!(self, elems, writer);
      // Close tag
      writer
        .write_event(Event::End(tag.to_end()))
        .map_err(VOTableError::Write)
    }
  }
}

/// Besselian (= tropical) year, e.g. B1950
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BesselianYear(pub f64);
impl FromStr for BesselianYear {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let decimal_year_str = if let Some(stripped) = s.strip_prefix('B') {
      stripped
    } else {
      s
    };
    decimal_year_str.parse::<f64>().map(BesselianYear)
  }
}
impl Display for BesselianYear {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_fmt(format_args!("B{}", self.0))
  }
}

/// Julian year, e.g. J2000, J2015.5, ...
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JulianYear(pub f64);
impl FromStr for JulianYear {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let decimal_year_str = if let Some(stripped) = s.strip_prefix('J') {
      stripped
    } else {
      s
    };
    decimal_year_str.parse::<f64>().map(JulianYear)
  }
}
impl Display for JulianYear {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_fmt(format_args!("J{}", self.0))
  }
}

/// Missing coosys are:
/// * xy
/// * barycentric
/// * geo_app
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "system")]
pub enum System {
  #[serde(rename = "eq_FK4")]
  EquatorialFK4 {
    /// Equinox in julian years (ex: 1950.0)
    equinox: BesselianYear,
    /// Epoch value in julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "eq_FK5")]
  EquatorialFK5 {
    /// Equinox in Besselian years (ex: 1950.0)
    equinox: JulianYear,
    /// Epoch value in Besseilan years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "ICRS")]
  ICRS {
    /// Epoch value in julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "ecl_FK4")]
  EcliptiqueFK4 {
    /// Equinox in julian years (ex: 1950.0)
    equinox: BesselianYear,
    /// Epoch value in julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "ecl_FK5")]
  EcliptiqueFK5 {
    /// Equinox in julian years (ex: 1950.0)
    equinox: JulianYear,
    /// Epoch value in julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "galactic")]
  Galactic {
    /// Epoch value in julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "supergalactic")]
  SuperGalactic {
    /// Epoch value in julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
}

impl System {
  pub fn new_default_eq_fk4() -> System {
    System::new_eq_fk4(1950.0)
  }

  pub fn new_eq_fk4(equinox_in_besselian_year: f64) -> System {
    System::EquatorialFK4 {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None,
    }
  }

  pub fn new_default_ecl_fk4() -> System {
    System::new_ecl_fk4(1950.0)
  }

  pub fn new_ecl_fk4(equinox_in_besselian_year: f64) -> System {
    System::EcliptiqueFK4 {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None,
    }
  }

  pub fn new_default_eq_fk5() -> System {
    System::new_eq_fk4(2000.0)
  }

  pub fn new_eq_fk5(equinox_in_julian_year: f64) -> System {
    System::EquatorialFK5 {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None,
    }
  }

  pub fn new_default_ecl_fk5() -> System {
    System::new_ecl_fk4(2000.0)
  }

  pub fn new_ecl_fk5(equinox_in_julian_year: f64) -> System {
    System::EcliptiqueFK5 {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None,
    }
  }

  pub fn new_icrs() -> System {
    System::ICRS { epoch: None }
  }

  pub fn new_galactic() -> System {
    System::Galactic { epoch: None }
  }

  pub fn new_supergalactic() -> System {
    System::SuperGalactic { epoch: None }
  }

  /// For FK4 systems, the epoch must be provided in Besselian years
  /// For FK5 systems, the epoch must be provided in Julian years
  pub fn set_epoch(mut self, epoch_in_years: f64) -> Self {
    match &mut self {
      System::EquatorialFK4 { equinox: _, epoch } => {
        let _prev = epoch.insert(BesselianYear(epoch_in_years));
      }
      System::EcliptiqueFK4 { equinox: _, epoch } => {
        let _prev = epoch.insert(BesselianYear(epoch_in_years));
      }
      System::EquatorialFK5 { equinox: _, epoch } => {
        let _prev = epoch.insert(JulianYear(epoch_in_years));
      }
      System::EcliptiqueFK5 { equinox: _, epoch } => {
        let _prev = epoch.insert(JulianYear(epoch_in_years));
      }
      System::ICRS { epoch } => {
        let _prev = epoch.insert(JulianYear(epoch_in_years));
      }
      System::Galactic { epoch } => {
        let _prev = epoch.insert(JulianYear(epoch_in_years));
      }
      System::SuperGalactic { epoch } => {
        let _prev = epoch.insert(JulianYear(epoch_in_years));
      }
    };
    self
  }

  pub fn set_epoch_from_str(mut self, epoch_in_years: &str) -> Result<Self, ParseFloatError> {
    match &mut self {
      System::EquatorialFK4 { equinox: _, epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
      System::EcliptiqueFK4 { equinox: _, epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
      System::EquatorialFK5 { equinox: _, epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
      System::EcliptiqueFK5 { equinox: _, epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
      System::ICRS { epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
      System::Galactic { epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
      System::SuperGalactic { epoch } => {
        let _prev = epoch.insert(epoch_in_years.parse()?);
      }
    }
    Ok(self)
  }

  pub fn for_each_attribute<F>(&self, f: &mut F)
  where
    F: FnMut(&str, &str),
  {
    match self {
      System::EquatorialFK4 { equinox, epoch } => {
        f("system", "eq_FK4");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      System::EcliptiqueFK4 { equinox, epoch } => {
        f("system", "ecl_FK4");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      System::EquatorialFK5 { equinox, epoch } => {
        f("system", "eq_FK5");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      System::EcliptiqueFK5 { equinox, epoch } => {
        f("system", "ecl_FK5");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      System::ICRS { epoch } => {
        f("system", "ICRS");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      System::Galactic { epoch } => {
        f("system", "galactic");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      System::SuperGalactic { epoch } => {
        f("system", "supergalactic");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
    }
  }

  pub fn with_attributes<'a, W: Write>(
    &self,
    mut writer: ElementWriter<'a, W>,
  ) -> ElementWriter<'a, W> {
    // I have not (yet) found a way to avoid copies :o/
    // * I tried 'attrs: Vec<(&str, &str)>' plus  'writer.with_attributes'
    let mut attrs: Vec<(String, String)> = Vec::with_capacity(3);
    let mut f = |key: &str, val: &str| attrs.push((key.to_owned(), val.to_owned()));
    self.for_each_attribute(&mut f);
    for (k, v) in attrs {
      writer = writer.with_attribute((k.as_str(), v.as_str()))
    }
    writer
  }

  pub fn push_attributes(&self, tag: &mut BytesStart) {
    let mut f = |key: &str, val: &str| tag.push_attribute((key, val));
    self.for_each_attribute(&mut f);
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{events::Event, Reader, Writer};

  use crate::{
    coosys::{CooSys, System},
    QuickXmlReadWrite,
  };

  #[test]
  fn test_coosys_readwrite() {
    let xml = r#"<COOSYS ID="J2000" system="eq_FK5" equinox="J2000"/>"#;
    // Test read
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    let mut coosys = loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Empty(ref mut e) if e.local_name() == CooSys::TAG_BYTES => {
          let coosys = CooSys::from_attributes(e.attributes()).unwrap();
          assert_eq!(coosys.id, "J2000");
          match &coosys.coosys {
            System::EquatorialFK5 { equinox, epoch } => {
              assert_eq!(equinox.0, 2000.0);
              assert!(epoch.is_none());
            }
            _ => unreachable!(),
          }
          break coosys;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        _ => unreachable!(),
      }
    };
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    coosys.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }
}
