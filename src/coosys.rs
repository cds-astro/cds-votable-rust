//! Module dedicated to the `COOSYS` tag.

use std::{
  fmt::{self, Display, Formatter},
  io::{BufRead, Write},
  num::ParseFloatError,
  str::{self, FromStr},
};

use log::trace;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use crate::{
  error::VOTableError,
  fieldref::FieldRef,
  paramref::ParamRef,
  timesys::RefPosition,
  utils::{discard_comment, discard_event, unexpected_attr_warn},
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
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

/// Struct corresponding to the `COOSYS` XML tag.
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

  // attributes
  impl_builder_mandatory_string_attr!(id);
  impl_builder_mandatory_attr!(coosys, System);
  impl_builder_opt_attr!(refposition, RefPosition);
  /// Use for modification.
  /// No `set_equinox` for construction since we have `set_coosys`.
  pub fn set_equinox_by_ref<S: AsRef<str>>(&mut self, epoch: S) -> Result<(), VOTableError> {
    self.coosys.set_equinox_from_str_by_ref(epoch.as_ref())
  }
  /// Use for modification.
  /// No `set_epoch` for construction since we have `set_coosys`.
  pub fn set_epoch_by_ref<S: AsRef<str>>(&mut self, epoch: S) -> Result<(), VOTableError> {
    self.coosys.set_epoch_from_str_by_ref(epoch.as_ref())
  }
  // sub-elements
  impl_builder_push_elem!(FieldRef, CooSysElem);
  impl_builder_push_elem!(ParamRef, CooSysElem);

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
}

impl VOTableElement for CooSys {
  const TAG: &'static str = "COOSYS";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut id: Option<String> = None;
    let mut system: Option<String> = None;
    let mut equinox: Option<String> = None;
    let mut epoch: Option<String> = None;
    let mut refposition: Option<RefPosition> = None;
    // Look for attributes
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "ID" => id = Some(val.into()),
        "system" => system = Some(val.into()),
        "equinox" => equinox = Some(val.into()),
        "epoch" => epoch = Some(val.into()),
        "refposition" => refposition = Some(val.as_ref().parse().map_err(VOTableError::Variant)?),
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    // Set from found attributes
    if let (Some(id), Some(system)) = (id, system) {
      // Create and set system
      let mut system = equinox
        .map(|equinox| System::from_system_and_equinox(&system, equinox))
        .unwrap_or(System::from_system(system))?;
      if let Some(epoch) = epoch {
        system.set_epoch_from_str_by_ref(epoch)?;
      }
      // Create and set CooSys
      let mut coosys = CooSys::new(id, system);
      if let Some(refposition) = refposition {
        coosys.set_refposition_by_ref(refposition);
      }
      Ok(coosys)
    } else {
      Err(VOTableError::Custom(format!(
        "Attributes 'ID' and 'system' are mandatory in tag '{}'",
        Self::TAG
      )))
    }
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut system: Option<String> = None;
    let mut equinox: Option<String> = None;
    let mut epoch: Option<String> = None;
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "ID" => self.set_id_by_ref(val),
        "refposition" => {
          self.set_refposition_by_ref(val.as_ref().parse().map_err(VOTableError::Variant)?)
        }
        "system" => system = Some(val.into()),
        "equinox" => equinox = Some(val.into()),
        "epoch" => epoch = Some(val.into()),
        _ => unexpected_attr_warn(key, Self::TAG),
      }
    }
    match (system, equinox) {
      (Some(system), Some(equinox)) => {
        self.coosys = System::from_system_and_equinox(system, equinox)?
      }
      (Some(system), None) => self.coosys = System::from_system(system)?,
      (None, Some(equinox)) => self.coosys.set_equinox_from_str_by_ref(equinox)?,
      (None, None) => {}
    }
    if let Some(epoch) = epoch {
      self.coosys.set_epoch_from_str_by_ref(epoch.as_str())?;
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("ID", self.id.as_str());
    self.coosys.for_each_attribute(&mut f);
    if let Some(refposition) = &self.refposition {
      f("refposition", refposition.to_string().as_str());
    }
  }
}

impl HasSubElements for CooSys {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.elems.is_empty()
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
          FieldRef::TAG_BYTES => push_from_event_start!(self, FieldRef, reader, reader_buff, e),
          ParamRef::TAG_BYTES => push_from_event_start!(self, ParamRef, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          FieldRef::TAG_BYTES => push_from_event_empty!(self, FieldRef, e),
          ParamRef::TAG_BYTES => push_from_event_empty!(self, ParamRef, e),
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

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec_no_context!(self, elems, writer);
    Ok(())
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
    trace!("Parse Besselian year: '{}'", decimal_year_str);
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
    trace!("Parse Julian year: '{}'", decimal_year_str);
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

  pub fn from_system<S>(system: S) -> Result<Self, VOTableError>
  where
    S: AsRef<str>,
  {
    let system = system.as_ref();
    match system {
      "eq_FK4" => Ok(System::new_default_eq_fk4()),
      "eq_FK5" => Ok(System::new_default_eq_fk5()),
      "ICRS" => Ok(System::new_icrs()),
      "ecl_FK4" => Ok(System::new_default_ecl_fk4()),
      "ecl_FK5" => Ok(System::new_default_ecl_fk5()),
      "galactic" => Ok(System::new_galactic()),
      "supergalactic" => Ok(System::new_supergalactic()),
      _ => {
        Err(VOTableError::Custom(format!(
          "System not recognized in tag '{}'. Expected: one of [eq_FK4, eq_FK5, ICRS, ecl_FK4, ecl_FK5, galactic, supergalactic]. Actual: '{}'.",
          CooSys::TAG, system,
        )))
      }
    }
  }

  pub fn from_system_and_equinox<S, E>(system: S, equinox: E) -> Result<Self, VOTableError>
  where
    S: AsRef<str>,
    E: AsRef<str>,
  {
    let system = system.as_ref();
    let equinox = equinox.as_ref();
    match system {
      "eq_FK4" => equinox
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| System::new_eq_fk4(equinox.0)),
      "eq_FK5" => equinox
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| System::new_eq_fk5(equinox.0)),
      "ICRS" => Ok(System::new_icrs()),
      "ecl_FK4" => equinox
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| System::new_ecl_fk4(equinox.0)),
      "ecl_FK5" => equinox
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| System::new_ecl_fk5(equinox.0)),
      "galactic" => Ok(System::new_galactic()),
      "supergalactic" => Ok(System::new_supergalactic()),
      _ => Err(VOTableError::Custom(format!(
        "System not recognized in tag '{}'. Expected: one of [eq_FK4, eq_FK5, ICRS, ecl_FK4, ecl_FK5, galactic, supergalactic]. Actual: '{}'.",
        CooSys::TAG, system,
      ))),
    }
  }

  /// For FK4 systems, the epoch must be provided in Besselian years
  /// For FK5 systems, the epoch must be provided in Julian years
  pub fn set_equinox(mut self, equinox_in_years: f64) -> Self {
    self.set_equinox_by_ref(equinox_in_years);
    self
  }

  /// For FK4 systems, the epoch must be provided in Besselian years
  /// For FK5 systems, the epoch must be provided in Julian years
  pub fn set_equinox_by_ref(&mut self, equinox_in_years: f64) {
    match self {
      System::EquatorialFK4 { equinox, .. } => equinox.0 = equinox_in_years,
      System::EcliptiqueFK4 { equinox, .. } => equinox.0 = equinox_in_years,
      System::EquatorialFK5 { equinox, .. } => equinox.0 = equinox_in_years,
      System::EcliptiqueFK5 { equinox, .. } => equinox.0 = equinox_in_years,
      _ => {}
    }
  }

  pub fn set_equinox_from_str<S: AsRef<str>>(mut self, equinox: S) -> Result<Self, VOTableError> {
    self.set_equinox_from_str_by_ref(equinox).map(|()| self)
  }

  pub fn set_equinox_from_str_by_ref<S: AsRef<str>>(
    &mut self,
    equinox: S,
  ) -> Result<(), VOTableError> {
    let equinox_str = equinox.as_ref();
    match self {
      System::EquatorialFK4 { equinox, .. } => equinox_str
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox_str.to_string(), e))
        .map(|new_equinox| *equinox = new_equinox),
      System::EcliptiqueFK4 { equinox, .. } => equinox_str
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox_str.to_string(), e))
        .map(|new_equinox| *equinox = new_equinox),
      System::EquatorialFK5 { equinox, .. } => equinox_str
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox_str.to_string(), e))
        .map(|new_equinox| *equinox = new_equinox),
      System::EcliptiqueFK5 { equinox, .. } => equinox_str
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox_str.to_string(), e))
        .map(|new_equinox| *equinox = new_equinox),
      _ => Ok(()),
    }
  }

  /// For FK4 systems, the epoch must be provided in Besselian years
  /// For FK5 systems, the epoch must be provided in Julian years
  pub fn set_epoch(mut self, epoch_in_years: f64) -> Self {
    self.set_epoch_by_ref(epoch_in_years);
    self
  }

  pub fn set_epoch_by_ref(&mut self, epoch_in_years: f64) {
    let _ = match self {
      System::EquatorialFK4 { equinox: _, epoch } => epoch.insert(BesselianYear(epoch_in_years)).0,
      System::EcliptiqueFK4 { equinox: _, epoch } => epoch.insert(BesselianYear(epoch_in_years)).0,
      System::EquatorialFK5 { equinox: _, epoch } => epoch.insert(JulianYear(epoch_in_years)).0,
      System::EcliptiqueFK5 { equinox: _, epoch } => epoch.insert(JulianYear(epoch_in_years)).0,
      System::ICRS { epoch } => epoch.insert(JulianYear(epoch_in_years)).0,
      System::Galactic { epoch } => epoch.insert(JulianYear(epoch_in_years)).0,
      System::SuperGalactic { epoch } => epoch.insert(JulianYear(epoch_in_years)).0,
    };
  }

  pub fn set_epoch_from_str<S: AsRef<str>>(mut self, epoch: S) -> Result<Self, VOTableError> {
    self.set_epoch_from_str_by_ref(epoch).map(|()| self)
  }

  pub fn set_epoch_from_str_by_ref<S: AsRef<str>>(&mut self, epoch: S) -> Result<(), VOTableError> {
    let epoch_str = epoch.as_ref();
    match self {
      System::EquatorialFK4 { equinox: _, epoch } => epoch_str
        .parse::<BesselianYear>()
        .map(|y| epoch.insert(y).0),
      System::EcliptiqueFK4 { equinox: _, epoch } => epoch_str
        .parse::<BesselianYear>()
        .map(|y| epoch.insert(y).0),
      System::EquatorialFK5 { equinox: _, epoch } => {
        epoch_str.parse::<JulianYear>().map(|y| epoch.insert(y).0)
      }
      System::EcliptiqueFK5 { equinox: _, epoch } => {
        epoch_str.parse::<JulianYear>().map(|y| epoch.insert(y).0)
      }
      System::ICRS { epoch } => epoch_str.parse::<JulianYear>().map(|y| epoch.insert(y).0),
      System::Galactic { epoch } => epoch_str.parse::<JulianYear>().map(|y| epoch.insert(y).0),
      System::SuperGalactic { epoch } => epoch_str.parse::<JulianYear>().map(|y| epoch.insert(y).0),
    }
    .map_err(|e| VOTableError::ParseYear(epoch_str.to_string(), e))
    .map(|_| ())
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
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{events::Event, Reader, Writer};

  use crate::{
    coosys::{CooSys, System},
    QuickXmlReadWrite, VOTableElement,
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
          let coosys = CooSys::from_event_empty(e).unwrap();
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
