
use std::{
  io::{BufRead, Write},
  str::{self, FromStr},
  num::ParseFloatError,
};

use quick_xml::{
  ElementWriter, Reader, Writer,
  events::attributes::Attributes
};

use serde;

use super::{
  QuickXmlReadWrite,
  error::VOTableError, 
};

// https://ned.ipac.caltech.edu/Documents/Guides/Calculations/calcdoc

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CooSys {
  #[serde(rename = "ID")]
  pub id: String,
  #[serde(flatten)]
  pub coosys: System
}

impl CooSys {
  pub fn new<S: Into<String>>(id: S, coosys: System) -> Self {
    Self { 
      id: id.into(), 
      coosys
    }
  }
}

impl QuickXmlReadWrite for CooSys {
  const TAG: &'static str = "COOSYS";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut id: Option<String> = None;
    let mut system: Option<String> = None;
    let mut equinox: Option<String> = None;
    let mut epoch: Option<String> = None;
    // Look for attributes
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = String::from_utf8(attr.value.as_ref().to_vec()).map_err(VOTableError::FromUtf8)?;
      match attr.key {
        b"ID" => id = Some(value),
        b"system" => system = Some(value),
        b"equinox" => equinox = Some(value),
        b"epoch" => epoch = Some(value),
        _ => { eprintln!("WARNING: attribute {:?} in {} is ignored", std::str::from_utf8(attr.key), Self::TAG); }
      }
    }
    // Set from found attributes
    if let (Some(id), Some(system)) = (id, system) {
     let mut system = match system.as_str() {
       "eq_FK4" => if let Some(equinox) = equinox {
         System::new_eq_fk4(equinox.parse::<BesselianYear>().map_err(VOTableError::ParseYear)?.0)
       } else {
         System::new_default_eq_fk4()
       },
       "eq_FK5" => if let Some(equinox) = equinox {
         System::new_eq_fk5(equinox.parse::<JulianYear>().map_err(VOTableError::ParseYear)?.0)
       } else {
         System::new_default_eq_fk5()
       },
       "ICRS" => System::new_icrs(),
       "ecl_FK4" => if let Some(equinox) = equinox {
         System::new_ecl_fk4(equinox.parse::<BesselianYear>().map_err(VOTableError::ParseYear)?.0)
       } else {
         System::new_default_ecl_fk4()
       },
       "ecl_FK5" => if let Some(equinox) = equinox {
         System::new_ecl_fk5(equinox.parse::<JulianYear>().map_err(VOTableError::ParseYear)?.0)
       } else {
         System::new_default_ecl_fk5()
       },
       "galactic" => System::new_galactic(),
       "supergalactic" => System::new_supergalactic(),
       _ => { return Err(VOTableError::Custom(format!("System '{}' not recognized in tag '{}'", system, Self::TAG))); }
     };
      if let Some(epoch) = epoch {
        system = system.set_epoch_from_str(epoch.as_str()).map_err(VOTableError::ParseYear)?;
      }
      Ok(CooSys::new(id, system))
    } else {
      Err(VOTableError::Custom(format!("Attributes 'ID' and 'system' are mandatory in tag '{}'", Self::TAG)))
    }
  }

  fn read_sub_elements<R: BufRead>(
    &mut self,
    _reader: Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    unreachable!()
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    unreachable!()
  }

  fn write<W: Write>(&mut self, writer: &mut Writer<W>,  _context: &Self::Context) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("ID", self.id.as_str()));
    elem_writer = self.coosys.with_attributes(elem_writer);
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
  
}

/*
/// Either Julian years or Besselian years.
/// E.g: J2000, B1950, J2015.5, ...
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "astro_year")]
enum JulianOrBesselianYear{
  Julian(f64),
  Besselian(f64) 
}*/

/// Besselian (= tropical) year, e.g. B1950
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BesselianYear(pub f64);
impl FromStr for BesselianYear {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let decimal_year_str= if let Some(stripped) = s.strip_prefix('B') {
      stripped
    } else {
      s
    };
    decimal_year_str.parse::<f64>().map(BesselianYear)
  }
}

/// Julian year, e.g. J2000, J2015.5, ...
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct JulianYear(pub f64);
impl FromStr for JulianYear {
  type Err = ParseFloatError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let decimal_year_str= if let Some(stripped) = s.strip_prefix('J') {
      stripped
    } else {
      s
    };
    decimal_year_str.parse::<f64>().map(JulianYear)
  }
}

/*#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "epoch_type")]
pub enum Epoch {
  #[serde(rename = "unknown")]
  Unknown,
  #[serde(rename = "unknown")]
  Val{
    #[serde(rename = "epoch_val")]
    value: f64
  }, // in years
  Var {
    #[serde(rename = "epoch_ref")]
    ref_: String 
  },
}*/

/*
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BesselianEpoch(pub f64);
// pub struct BesselianEpoch(pub Epoch);

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct JulianEpoch(pub f64);
// pub struct JulianEpoch(pub Epoch);
*/

/// Missing coosys are:
/// * xy
/// * barycentric
/// * geo_app
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
      epoch: None
    }
  }

  pub fn new_default_ecl_fk4() -> System {
    System::new_ecl_fk4(1950.0)
  }

  pub fn new_ecl_fk4(equinox_in_besselian_year: f64) -> System {
    System::EcliptiqueFK4 {
      equinox: BesselianYear(equinox_in_besselian_year),
      epoch: None
    }
  }

  pub fn new_default_eq_fk5() -> System {
    System::new_eq_fk4(2000.0)
  }

  pub fn new_eq_fk5(equinox_in_julian_year: f64) -> System {
    System::EquatorialFK5 {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None
    }
  }

  pub fn new_default_ecl_fk5() -> System {
    System::new_ecl_fk4(2000.0)
  }

  pub fn new_ecl_fk5(equinox_in_julian_year: f64) -> System {
    System::EcliptiqueFK5 {
      equinox: JulianYear(equinox_in_julian_year),
      epoch: None
    }
  }

  pub fn new_icrs() -> System {
    System::ICRS {
      epoch: None
    }
  }

  pub fn new_galactic() -> System {
    System::Galactic {
      epoch: None
    }
  }

  pub fn new_supergalactic() -> System {
    System::SuperGalactic {
      epoch: None
    }
  }
  
  /// For FK4 systems, the epoch must be provided in Besselian years
  /// For FK5 systems, the epoch must be provided in Julian years
  pub fn set_epoch(mut self, epoch_in_years: f64) -> Self {
    match &mut self {
      System::EquatorialFK4 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(BesselianYear(epoch_in_years)); }
      System::EcliptiqueFK4 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(BesselianYear(epoch_in_years)); }
      System::EquatorialFK5 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(JulianYear(epoch_in_years)); }
      System::EcliptiqueFK5 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(JulianYear(epoch_in_years)); }
      System::ICRS {
        epoch
      } => { let _prev = epoch.insert(JulianYear(epoch_in_years)); }
      System::Galactic {
        epoch
      } => { let _prev = epoch.insert(JulianYear(epoch_in_years)); }
      System::SuperGalactic {
        epoch
      } => { let _prev = epoch.insert(JulianYear(epoch_in_years)); }
    };
    self
  }

  pub fn set_epoch_from_str(mut self, epoch_in_years: &str) -> Result<Self, ParseFloatError> {
    match &mut self {
      System::EquatorialFK4 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
      System::EcliptiqueFK4 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
      System::EquatorialFK5 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
      System::EcliptiqueFK5 {
        equinox: _,
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
      System::ICRS {
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
      System::Galactic {
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
      System::SuperGalactic {
        epoch
      } => { let _prev = epoch.insert(epoch_in_years.parse()?); }
    }
    Ok(self)
  }
  
  pub fn with_attributes<'a, W: Write>(&self, mut writer: ElementWriter<'a, W>) -> ElementWriter<'a, W> {
    // let mut writer = writer;
    match self {
      System::EquatorialFK4 {
        equinox,
        epoch } => {
        writer = writer.with_attribute(("system", "eq_FK4"));
        writer = writer.with_attribute(("equinox", format!("B{:}", equinox.0).as_str()));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("B{:}", epoch.0).as_str()));
        }
      }
      System::EcliptiqueFK4 {
        equinox,
        epoch
      } => {        
        writer = writer.with_attribute(("system", "ecl_FK4"));
        writer = writer.with_attribute(("equinox", format!("B{:}", equinox.0).as_str()));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("B{:}", epoch.0).as_str()));
        } 
      }
      System::EquatorialFK5 {
        equinox,
        epoch
      } => {
        writer = writer.with_attribute(("system", "eq_FK5"));
        writer = writer.with_attribute(("equinox", format!("J{:}", equinox.0).as_str()));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("J{:}", epoch.0).as_str()));
        }
      }
      System::EcliptiqueFK5 {
        equinox,
        epoch
      } => {
        writer = writer.with_attribute(("system", "ecl_FK5"));
        writer = writer.with_attribute(("equinox", format!("J{:}", equinox.0).as_str()));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("J{:}", epoch.0).as_str()));
        }
      }
      System::ICRS {
        epoch
      } => {
        writer = writer.with_attribute(("system", "ICRS"));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("J{:}", epoch.0).as_str()));
        }
      }
      System::Galactic {
        epoch
      } => {
        writer = writer.with_attribute(("system", "galactic"));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("J{:}", epoch.0).as_str()));
        }
      }
      System::SuperGalactic {
        epoch
      } => {
        writer = writer.with_attribute(("system", "supergalactic"));
        if let Some(epoch) = epoch {
          writer = writer.with_attribute(("epoch", format!("J{:}", epoch.0).as_str()));
        }
      }
    };
    writer
  }
}


#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{Reader, events::Event, Writer};
  
  use crate::{
    QuickXmlReadWrite,
    coosys::{CooSys, System}
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
            System::EquatorialFK5 { equinox, epoch} => {
              assert_eq!(equinox.0, 2000.0);
              assert!(epoch.is_none());
            }
            _ => unreachable!()
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