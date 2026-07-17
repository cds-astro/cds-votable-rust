//! Module dedicated to the `COOSYS` tag.

use std::{
  fmt::{self, Display, Formatter},
  io::{BufRead, Write},
  num::ParseFloatError,
  str::{self, FromStr},
};

use log::{trace, warn};
use paste::paste;
use quick_xml::{Reader, Writer, events::Event};

use crate::{
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
  error::VOTableError,
  fieldref::FieldRef,
  paramref::ParamRef,
  timesys::RefPosition,
  utils::{discard_comment, discard_event, unexpected_attr_warn},
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
        .map(|equinox| System::from_system_and_equinox(&system, &equinox))
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
        Event::Start(e) => match e.local_name() {
          FieldRef::TAG_BYTES => push_from_event_start!(self, FieldRef, reader, reader_buff, e),
          ParamRef::TAG_BYTES => push_from_event_start!(self, ParamRef, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ));
          }
        },
        Event::Empty(e) => match e.local_name() {
          FieldRef::TAG_BYTES => push_from_event_empty!(self, FieldRef, e),
          ParamRef::TAG_BYTES => push_from_event_empty!(self, ParamRef, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ));
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

/*
pub enum BesselianOrJulian {
  Besselian(BesselianYear),
  Julian(JulianYear),
}
*/

/// See [IVOA refframe vocabulary](https://www.ivoa.net/rdf/refframe/2022-02-22/refframe.html)
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "system")]
pub enum System {
  #[serde(rename = "AZ_EL")]
  /// "Local azimuth and elevation. (Ground-based observations; Azimuth from North through East.)"
  AzEl,
  #[serde(rename = "BODY")]
  /// "Generic bodycentric coordinates. Data annotated in this way cannot be automatically combined with any other data."
  Body,
  #[serde(rename = "ECLIPTIC")]
  /// "Ecliptic coordinates; the ecliptic of J2000.0 is assumed."
  Ecliptic {
    /// Equinox in Julian years (ex: 1950.0)
    equinox: JulianYear,
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "EQUATORIAL")]
  /// "Umbrella term of all equatorial frames. Only use for old, pre-FK4 equatorial coordinates."
  Equatorial {
    /// Equinox in Besselian years (ex: 1950.0)
    equinox: BesselianYear,
    /// Epoch value in Besselian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "FK4")]
  FK4 {
    /// Equinox in Besselian years (ex: 1950.0)
    equinox: BesselianYear,
    /// Epoch value in Besselian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "FK5")]
  FK5 {
    /// Equinox in Julian years (ex: 2000.0)
    equinox: JulianYear,
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "GALACTIC")]
  /// "Galactic coordinates, modern definition: Pole at precisely FK4 B1950 192.25, 27.4,
  /// origin at approximately FK4 B1950 265.55, -28.92. See 1960MNRAS.121..123B for details."
  Galactic {
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "GALACTIC_I")]
  /// "Old, pre-1958, Galactic coordinates. See 1960MNRAS.121..123B for details."
  GalacticI {
    /// Epoch value in Besselian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "GENERIC_GALACTIC")]
  /// "Umbrella term for Galactic coordinates. If at all possible, use a more specific term,
  /// as historically, many different conventions have been in use."
  GenericGalactic {
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>, // or Julian ?!!
  },
  #[serde(rename = "ICRS")]
  /// "International Celestial Reference System as defined by 1998AJ....116..516M."
  ICRS {
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "SUPER_GALACTIC")]
  /// "Supergalactic coordinates (pole at GALACTIC 47.37, +6.32, origin at GALACTIC 137.37, 0."
  SuperGalactic {
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "UNKNOWN")]
  /// "Unknown reference frame. Only to be used as a last resort or for simulations.
  /// Data annotated in this way cannot be automatically combined with any other data."
  Unknown,
  #[serde(rename = "barycentric")]
  /// "Old VOTable COOSYS term indicating ICRS at BARYCENTER reference position.
  /// In the modern VO, refpos and refframe are no longer represented together.
  /// Do not use this any more."
  BarycentricDeprec,
  #[serde(rename = "ecl_FK4")]
  /// "Old VOTable COOSYS term ecliptic coordinates for the FK4 ecliptic (of B1950.0)."
  EclipticFK4 {
    /// Equinox in Besselian years (ex: 1950.0)
    equinox: BesselianYear,
    /// Epoch value in Besselian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "ecl_FK5")]
  /// "Old VOTable COOSYS term ecliptic coordinates for the FK5 ecliptic (of J2000.0)."
  EclipticFK5Deprec {
    /// Equinox in Julian years (ex: 1950.0)
    equinox: JulianYear,
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "eq_FK4")]
  /// "Old VOTable COOSYS term for FK4."
  EquatorialFK4Deprec {
    /// Equinox in Besselian years (ex: 1950.0)
    equinox: BesselianYear,
    /// Epoch value in Besselian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<BesselianYear>,
  },
  #[serde(rename = "eq_FK5")]
  /// "Old VOTable COOSYS term for FK5."
  EquatorialFK5Deprec {
    /// Equinox in Julian years (ex: 2000.0)
    equinox: JulianYear,
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "galactic")]
  /// "Old VOTable COOSYS term for GALACTIC."
  GalacticDeprec {
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "geo_app")]
  /// "Positions given as observed by a fictitious observer at the Earth's centre for the equator of observation."
  GeoApp,
  #[serde(rename = "supergalactic")]
  /// "Old VOTable COOSYS term for SUPER_GALACTIC"
  SuperGalacticDeprec {
    /// Epoch value in Julian years (ex: 2010.5506), or colname
    #[serde(skip_serializing_if = "Option::is_none")]
    epoch: Option<JulianYear>,
  },
  #[serde(rename = "xy")]
  /// "Old VOTable COOSYS term for UNKNOWN"
  XyDeprec,
}

impl System {
  pub fn new_az_el() -> System {
    Self::AzEl
  }

  pub fn new_body() -> System {
    Self::Body
  }

  pub fn new_default_ecliptic() -> System {
    Self::new_ecliptic(2000.0)
  }

  pub fn new_ecliptic(equinox_in_julian_year: f64) -> System {
    Self::Ecliptic {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None,
    }
  }

  pub fn new_default_equatorial() -> System {
    Self::new_equatorial(1950.0)
  }

  pub fn new_equatorial(equinox_in_besselian_year: f64) -> System {
    Self::Equatorial {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None,
    }
  }

  pub fn new_default_fk4() -> System {
    Self::new_fk4(1950.0)
  }

  pub fn new_fk4(equinox_in_besselian_year: f64) -> System {
    Self::FK4 {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None,
    }
  }

  pub fn new_default_fk5() -> System {
    Self::new_fk4(1950.0)
  }

  pub fn new_fk5(equinox_in_julian_year: f64) -> System {
    Self::FK5 {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None,
    }
  }

  pub fn new_galactic() -> System {
    Self::Galactic { epoch: None }
  }

  pub fn new_galactic_i() -> System {
    Self::GalacticI { epoch: None }
  }

  pub fn new_generic_galactic() -> System {
    Self::GenericGalactic { epoch: None }
  }

  pub fn new_icrs() -> System {
    Self::ICRS { epoch: None }
  }

  pub fn new_super_galactic() -> System {
    Self::SuperGalactic { epoch: None }
  }

  pub fn new_unknown() -> System {
    Self::Unknown
  }

  /// Warning: deprecated
  pub fn new_barycentric_deprec() -> System {
    Self::BarycentricDeprec
  }

  /// WARNING: deprecated
  pub fn new_default_eq_fk4_deprec() -> System {
    Self::new_eq_fk4_deprec(1950.0)
  }

  /// WARNING: deprecated
  pub fn new_eq_fk4_deprec(equinox_in_besselian_year: f64) -> System {
    Self::EquatorialFK4Deprec {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None,
    }
  }

  pub fn new_default_ecl_fk4() -> System {
    Self::new_ecl_fk4(1950.0)
  }

  pub fn new_ecl_fk4(equinox_in_besselian_year: f64) -> System {
    Self::EclipticFK4 {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None,
    }
  }

  /// WARNING: deprecated
  pub fn new_default_eq_fk5_deprec() -> System {
    Self::new_eq_fk5_deprec(2000.0)
  }

  /// WARNING: deprecated
  pub fn new_eq_fk5_deprec(equinox_in_julian_year: f64) -> System {
    Self::EquatorialFK5Deprec {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None,
    }
  }

  /// WARNING: deprecated
  pub fn new_default_ecl_fk5_deprec() -> System {
    Self::new_ecl_fk5_deprec(1950.0)
  }

  /// WARNING: deprecated
  pub fn new_ecl_fk5_deprec(equinox_in_julian_year: f64) -> System {
    Self::EclipticFK5Deprec {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None,
    }
  }

  /// WARNING: deprecated
  pub fn new_galactic_deprec() -> System {
    Self::GalacticDeprec { epoch: None }
  }

  pub fn new_geo_app() -> System {
    Self::GeoApp
  }

  /// WARNING: deprecated
  pub fn new_supergalactic_deprec() -> System {
    Self::SuperGalacticDeprec { epoch: None }
  }

  /// WARNING: deprecated
  pub fn new_xy_deprec() -> System {
    Self::XyDeprec
  }

  pub fn from_system<S>(system: S) -> Result<Self, VOTableError>
  where
    S: AsRef<str>,
  {
    let system = system.as_ref();
    match system {
      "AZ_EL" => Ok(Self::new_az_el()),
      "BODY" => Ok(Self::new_body()),
      "ECLIPTIC" => Ok(Self::new_default_ecliptic()),
      "EQUATORIAL" => Ok(Self::new_default_equatorial()),
      "FK4" => Ok(Self::new_default_fk4()),
      "FK5" => Ok(Self::new_default_fk5()),
      "GALACTIC" => Ok(Self::new_galactic()),
      "GALACTIC_I" => Ok(Self::new_galactic_i()),
      "GENERIC_GALACTIC" => Ok(Self::new_generic_galactic()),
      "ICRS" => Ok(Self::new_icrs()),
      "SUPER_GALACTIC" => Ok(Self::new_super_galactic()),
      "UNKNOWN" => Ok(Self::new_unknown()),
      "barycentric" => Ok(Self::new_barycentric_deprec()),
      "ecl_FK4" => Ok(Self::new_default_ecl_fk4()),
      "ecl_FK5" => Ok(Self::new_default_ecl_fk5_deprec()),
      "eq_FK4" => Ok(Self::new_default_eq_fk4_deprec()),
      "eq_FK5" => Ok(Self::new_default_eq_fk5_deprec()),
      "galactic" => Ok(Self::new_galactic_deprec()),
      "geo_app" => Ok(Self::new_geo_app()),
      "supergalactic" => Ok(Self::new_supergalactic_deprec()),
      "xy" => Ok(Self::new_xy_deprec()),
      _ => Err(VOTableError::Custom(format!(
        "System not recognized in tag '{}'. Expected: one of [AZ_EL, BODY, ECLIPTIC, EQUATORIAL, FK4, FK5, GALACTIC, GALACTIC_I, GENERIC_GALACTIC, ICRS, SUPER_GALACTIC, UNKNOWN, barycentric, ecl_FK4, ecl_FK5, eq_FK4, eq_FK5, galactic, geo_app, supergalactic, xy]. Actual: '{}'.",
        CooSys::TAG,
        system,
      ))),
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
      "ECLIPTIC" => equinox
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_ecliptic(equinox.0)),
      "EQUATORIAL" => equinox
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_equatorial(equinox.0)),
      "FK4" => equinox
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_fk4(equinox.0)),
      "FK5" => equinox
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_fk5(equinox.0)),
      "eq_FK4" => equinox
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_eq_fk4_deprec(equinox.0)),
      "eq_FK5" => equinox
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_eq_fk5_deprec(equinox.0)),
      "ecl_FK4" => equinox
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_ecl_fk4(equinox.0)),
      "ecl_FK5" => equinox
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox.to_string(), e))
        .map(|equinox| Self::new_ecl_fk5_deprec(equinox.0)),
      _ => {
        warn!(
          "Equinox not expected for system {}. Value is ignored!",
          system
        );
        Self::from_system(system)
      }
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
      Self::Equatorial { equinox, .. }
      | Self::FK4 { equinox, .. }
      | Self::EquatorialFK4Deprec { equinox, .. }
      | Self::EclipticFK4 { equinox, .. } => equinox.0 = equinox_in_years,
      Self::Ecliptic { equinox, .. }
      | Self::FK5 { equinox, .. }
      | Self::EquatorialFK5Deprec { equinox, .. }
      | Self::EclipticFK5Deprec { equinox, .. } => equinox.0 = equinox_in_years,
      _ => warn!(
        "Equinox not expected for system {:?}. Value is ignored!",
        self
      ),
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
      Self::Equatorial { equinox, .. }
      | Self::FK4 { equinox, .. }
      | Self::EquatorialFK4Deprec { equinox, .. }
      | Self::EclipticFK4 { equinox, .. } => equinox_str
        .parse::<BesselianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox_str.to_string(), e))
        .map(|new_equinox| *equinox = new_equinox),
      Self::Ecliptic { equinox, .. }
      | Self::FK5 { equinox, .. }
      | Self::EquatorialFK5Deprec { equinox, .. }
      | Self::EclipticFK5Deprec { equinox, .. } => equinox_str
        .parse::<JulianYear>()
        .map_err(|e| VOTableError::ParseYear(equinox_str.to_string(), e))
        .map(|new_equinox| *equinox = new_equinox),
      _ => {
        warn!(
          "Equinox not expected for system {:?}. Value is ignored!",
          self
        );
        Ok(())
      }
    }
  }

  /// For FK4 systems, the epoch must be provided in Besselian years
  /// For FK5 systems, the epoch must be provided in Julian years
  pub fn set_epoch(mut self, epoch_in_years: f64) -> Self {
    self.set_epoch_by_ref(epoch_in_years);
    self
  }

  pub fn set_epoch_by_ref(&mut self, epoch_in_years: f64) {
    match self {
      Self::Equatorial { equinox: _, epoch }
      | Self::FK4 { equinox: _, epoch }
      | Self::GalacticI { epoch }
      | Self::GenericGalactic { epoch }
      | Self::EquatorialFK4Deprec { equinox: _, epoch }
      | Self::EclipticFK4 { equinox: _, epoch } => {
        let _ = epoch.insert(BesselianYear(epoch_in_years)).0;
      }
      Self::Ecliptic { equinox: _, epoch }
      | Self::FK5 { equinox: _, epoch }
      | Self::EquatorialFK5Deprec { equinox: _, epoch }
      | Self::EclipticFK5Deprec { equinox: _, epoch }
      | Self::ICRS { epoch }
      | Self::Galactic { epoch }
      | Self::GalacticDeprec { epoch }
      | Self::SuperGalactic { epoch }
      | Self::SuperGalacticDeprec { epoch } => {
        let _ = epoch.insert(JulianYear(epoch_in_years)).0;
      }
      _ => warn!(
        "Epoch not expected for system {:?}. Value is ignored!",
        self
      ),
    }
  }

  pub fn set_epoch_from_str<S: AsRef<str>>(mut self, epoch: S) -> Result<Self, VOTableError> {
    self.set_epoch_from_str_by_ref(epoch).map(|()| self)
  }

  pub fn set_epoch_from_str_by_ref<S: AsRef<str>>(&mut self, epoch: S) -> Result<(), VOTableError> {
    let epoch_str = epoch.as_ref();
    match self {
      Self::Equatorial { equinox: _, epoch }
      | Self::FK4 { equinox: _, epoch }
      | Self::GalacticI { epoch }
      | Self::GenericGalactic { epoch }
      | Self::EquatorialFK4Deprec { equinox: _, epoch }
      | Self::EclipticFK4 { equinox: _, epoch } => epoch_str.parse::<BesselianYear>().map(|y| {
        let _ = epoch.insert(y).0;
      }),
      Self::Ecliptic { equinox: _, epoch }
      | Self::FK5 { equinox: _, epoch }
      | Self::EquatorialFK5Deprec { equinox: _, epoch }
      | Self::EclipticFK5Deprec { equinox: _, epoch }
      | Self::ICRS { epoch }
      | Self::Galactic { epoch }
      | Self::GalacticDeprec { epoch }
      | Self::SuperGalactic { epoch }
      | Self::SuperGalacticDeprec { epoch } => epoch_str.parse::<JulianYear>().map(|y| {
        let _ = epoch.insert(y).0;
      }),
      _ => {
        warn!(
          "Epoch not expected for system {:?}. Value is ignored!",
          self
        );
        Ok(())
      }
    }
    .map_err(|e| VOTableError::ParseYear(epoch_str.to_string(), e))
  }

  pub fn for_each_attribute<F>(&self, f: &mut F)
  where
    F: FnMut(&str, &str),
  {
    match self {
      Self::AzEl => {
        f("system", "AZ_EL");
      }
      Self::Body => {
        f("system", "BODY");
      }
      Self::Ecliptic { equinox, epoch } => {
        f("system", "ECLIPTIC");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::Equatorial { equinox, epoch } => {
        f("system", "EQUATORIAL");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::FK4 { equinox, epoch } => {
        f("system", "FK4");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::FK5 { equinox, epoch } => {
        f("system", "FK5");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::Galactic { epoch } => {
        f("system", "GALACTIC");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::GalacticI { epoch } => {
        f("system", "GALACTIC_I");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::GenericGalactic { epoch } => {
        f("system", "GENERIC_GALACTIC");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::ICRS { epoch } => {
        f("system", "ICRS");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::SuperGalactic { epoch } => {
        f("system", "SUPER_GALACTIC");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::Unknown => {
        f("system", "UNKNOWN");
      }
      Self::BarycentricDeprec => {
        f("system", "barycentric");
      }
      Self::EclipticFK4 { equinox, epoch } => {
        f("system", "ecl_FK4");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::EclipticFK5Deprec { equinox, epoch } => {
        f("system", "ecl_FK5");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::EquatorialFK4Deprec { equinox, epoch } => {
        f("system", "eq_FK4");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::EquatorialFK5Deprec { equinox, epoch } => {
        f("system", "eq_FK5");
        f("equinox", equinox.to_string().as_str());
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::GalacticDeprec { epoch } => {
        f("system", "galactic");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::GeoApp => {
        f("system", "geo_app");
      }
      Self::SuperGalacticDeprec { epoch } => {
        f("system", "supergalactic");
        if let Some(epoch) = epoch {
          f("epoch", epoch.to_string().as_str());
        }
      }
      Self::XyDeprec => {
        f("system", "xy");
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{Reader, Writer, events::Event};

  use crate::{
    QuickXmlReadWrite, VOTableElement,
    coosys::{CooSys, System},
  };

  fn test_in_eq_out(xml: &str) {
    eprintln!("Test: {}", &xml);
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    let mut coosys = loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Empty(e) if e.local_name() == CooSys::TAG_BYTES => {
          let coosys = CooSys::from_event_empty(e).unwrap();
          break coosys;
        }
        Event::Text(e) if e.escaped().is_empty() => (), // First even read
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

  #[test]
  fn test_coosys_readwrite() {
    let xml = r#"<COOSYS ID="J2000" system="eq_FK5" equinox="J2000"/>"#;
    // Test read
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    let mut coosys = loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Empty(e) if e.local_name() == CooSys::TAG_BYTES => {
          let coosys = CooSys::from_event_empty(e).unwrap();
          assert_eq!(coosys.id, "J2000");
          match &coosys.coosys {
            System::EquatorialFK5Deprec { equinox, epoch } => {
              assert_eq!(equinox.0, 2000.0);
              assert!(epoch.is_none());
            }
            _ => unreachable!(),
          }
          break coosys;
        }
        Event::Text(e) if e.escaped().is_empty() => (), // First even read
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

  #[test]
  fn test_all_coosys() {
    // env_logger::init();
    // RUN TEST WITH:
    // > RUST_LOG=WARN cargo test test_all_coosys -- --nocapture
    test_in_eq_out(r#"<COOSYS ID="a" system="AZ_EL"/>"#);
    test_in_eq_out(r#"<COOSYS ID="b" system="BODY"/>"#);
    test_in_eq_out(r#"<COOSYS ID="c" system="ECLIPTIC" equinox="J2015" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="d" system="EQUATORIAL" equinox="B2015" epoch="B2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="e" system="FK4" equinox="B2015" epoch="B2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="f" system="FK5" equinox="J2015" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="g" system="GALACTIC" epoch="J2015.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="h" system="GALACTIC_I" epoch="B2015.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="i" system="GENERIC_GALACTIC" epoch="B2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="j" system="ICRS" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="k" system="SUPER_GALACTIC" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="l" system="UNKNOWN"/>"#);
    test_in_eq_out(r#"<COOSYS ID="m" system="barycentric"/>"#);
    test_in_eq_out(r#"<COOSYS ID="n" system="ecl_FK4" equinox="B2015" epoch="B2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="o" system="ecl_FK5" equinox="J2015" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="p" system="eq_FK4" equinox="B2015" epoch="B2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="q" system="eq_FK5" equinox="J2015" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="r" system="galactic" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="s" system="geo_app"/>"#);
    test_in_eq_out(r#"<COOSYS ID="t" system="supergalactic" epoch="J2010.1"/>"#);
    test_in_eq_out(r#"<COOSYS ID="u" system="xy"/>"#);
  }
}
