use std::{
  fmt,
  io::{BufRead, Write},
  str::{self, FromStr},
};
use std::fmt::Debug;

use quick_xml::{Reader, Writer, events::{attributes::Attributes}};

use paste::paste;

use serde;

use super::{
  QuickXmlReadWrite,
  error::VOTableError,
};


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TimeSys {
  #[serde(rename = "ID")]
  pub id: String,
  /// Julian Date in MJD.
  /// * `MJD-origin` = 2400000.5
  /// *  `JD-origin` = 0.0
  /// Not clear to me so far: "The timeorigin attribute MUST be given unless the time's 
  /// representation contains a year of a calendar era, in which case it MUST NOT be present"
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timeorigin: Option<f64>,
  pub timescale: TimeScale,
  pub refposition: RefPosition,
}

impl TimeSys {
  pub fn new<S: Into<String>>(
    id: S,
    timescale: TimeScale,
    refposition: RefPosition,
  ) -> Self {
    Self {
      id: id.into(),
      timeorigin: None,
      timescale,
      refposition,
    }
  }

  impl_builder_opt_attr!(timeorigin, f64);
}

impl QuickXmlReadWrite for TimeSys {
  const TAG: &'static str = "TIMESYS";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut id: Option<String> = None;
    let mut timeorigin: Option<f64> = None;
    let mut timescale: Option<TimeScale> = None;
    let mut refposition: Option<RefPosition> = None;
    // Look for attributes
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let value = String::from_utf8(attr.value.as_ref().to_vec()).map_err(VOTableError::FromUtf8)?;
      match attr.key {
        b"ID" => id = Some(value),
        b"timeorigin" => timeorigin = Some(value.parse().map_err(VOTableError::ParseFloat)?),
        b"timescale" => timescale = Some(value.parse().map_err(VOTableError::Variant)?),
        b"refposition" => refposition = Some(value.parse().map_err(VOTableError::Variant)?),
        _ => { eprintln!("WARNING: attribute {:?} in {} is ignored", std::str::from_utf8(attr.key), Self::TAG); }
      }
    }
    // Set from found attributes
    if let (Some(id), Some(timescale), Some(refposition)) = (id, timescale, refposition) {
      let mut timesys = TimeSys::new(id, timescale, refposition);
      if let Some(timeorigin) = timeorigin {
        timesys = timesys.set_timeorigin(timeorigin);
      }
      Ok(timesys)
    } else {
      Err(VOTableError::Custom(format!("Attributes 'ID', 'timescale' and 'refposition' are mandatory in tag '{}'", Self::TAG)))
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
    todo!()
  }
  
  fn write<W: Write>(
    &mut self, 
    writer: &mut Writer<W>, 
    _context: &Self::Context
  ) -> Result<(), VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("ID", self.id.as_str()));
    if let Some(timeorigin) = self.timeorigin {
      elem_writer = elem_writer.with_attribute(("timeorigin", timeorigin.to_string().as_str()));
    }
    elem_writer = elem_writer.with_attribute(("timescale", self.timescale.to_string().as_str()));
    elem_writer = elem_writer.with_attribute(("refposition", self.refposition.to_string().as_str()));
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}


pub struct Info {
  pub label: &'static str,
  pub description: &'static str,
}

impl Info {
  const fn new(label: &'static str, description: &'static str) -> Info {
    Info { label, description }
  }
}


const TAI_INFO: Info = Info::new(
  "International Atomic Time TAI",
  "Atomic time standard, TT-TAI = 32.184 s.",
);
const TT_INFO: Info = Info::new(
  "Terrestrial Time TT",
  "Time measured by a continuous clock on the surface of an ideal Earth. Defined via TCG as having been identical on 1977-01-01 and since running slower than it by an empirically determined factor L_C.  It is continuous with the ephemeris time ET widely used before 1984-01-01. The term TT should therefore be used for times in ET, too.  (IAU standard)",
);
const UT_INFO: Info = Info::new(
  "Earth rotation time UT",
  "We do not distinguish between UT0, UT1, and UT2. Applications requiring this level of precision need additional metadata.  This should also be used to label GMT times in datasets covering dates between 1925-01-01 and 1972-01-01. GMT in astronomical use before 1925 had a 12 hour offset and would require a new term.",
);
const UTC_INFO: Info = Info::new(
  "Universal Time, Coordinated UTC",
  "This is TAI, with leap seconds inserted occasionally in order to keep UTC within 0.9 s of UT1 (a different convention was in use before 1972-01-01).",
);
const GPS_INFO: Info = Info::new(
  "Global Positioning System time",
  "Runs (approximately) synchronously with TAI",
);
const TCG_INFO: Info = Info::new(
  "Geocentric Coordinate Time TCG",
  "Time measured by a clock moving with the Earth's center but not subject to the gravitational potential of the Earth",
);
const TCB_INFO: Info = Info::new(
  "Barycentric Coordinate Time TCB",
  " Derived from TCG, but taking into account the relativistic effects of the gravitational potential at the barycenter as well as velocity time dilation variations due to the eccentricity of the Earth's orbit.  See 1999A&A...348..642I for details.",
);
const TDB_INFO: Info = Info::new("Barycentric Dynamical Time TDB",
                                 "Runs slower than TCB at a constant rate so as to remain approximately in step with TT. Therefore runs quasi-synchronously with TT, except for the relativistic effects introduced by variations in the Earth's velocity relative to the barycenter.",
);
const UNKNOWN_TIMESCALE_INFO: Info = Info::new("Unknown or unavailable timescale",
                                               "This value indicates clients cannot transform the times reliably. This is to be used for simulated data, free-running clocks, or data for which information on the time scale has been lost.",
);

/// See the [IVOA timescale vocabulary](https://www.ivoa.net/rdf/timescale/2019-03-15/timescale.html)
// #[serde(tag = "timescale")]
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum TimeScale {
  TAI,
  TT,
  UT,
  UTC,
  GPS,
  TCG,
  TCB,
  TDB,
  UNKNOWN,
}

impl TimeScale {
  pub const fn info(&self) -> Info {
    match self {
      TimeScale::TAI => TAI_INFO,
      TimeScale::TT => TT_INFO,
      TimeScale::UT => UT_INFO,
      TimeScale::UTC => UTC_INFO,
      TimeScale::GPS => GPS_INFO,
      TimeScale::TCG => TCG_INFO,
      TimeScale::TCB => TCB_INFO,
      TimeScale::TDB => TDB_INFO,
      TimeScale::UNKNOWN => UNKNOWN_TIMESCALE_INFO
    }
  }
}

impl FromStr for TimeScale {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "tai" | "TAI" => Ok(TimeScale::TAI),
      "tt" | "TT"  => Ok(TimeScale::TT),
      "ut" | "UT"  => Ok(TimeScale::UT),
      "utc" | "UTC"  => Ok(TimeScale::UTC),
      "gps" | "GPS"  => Ok(TimeScale::GPS),
      "tcg" | "TCG"  => Ok(TimeScale::TCG),
      "tcb" | "TCB"  => Ok(TimeScale::TCB),
      "tdb" | "TDB"  => Ok(TimeScale::TDB),
      "unknown" | "UNKNOWN"  => Ok(TimeScale::UNKNOWN),
      _ => Err(format!("Unknown timescale variant. Actual: '{}'.", s))
    }
  }
}

impl fmt::Display for TimeScale {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    /*write!(f, "{}",
           match self {
             TimeScale::TAI => "tai",
             TimeScale::TT => "tt",
             TimeScale::UT => "ut",
             TimeScale::UTC => "utc",
             TimeScale::GPS => "gps",
             TimeScale::TCG => "tcg",
             TimeScale::TCB => "tcb",
             TimeScale::TDB => "tdb",
             TimeScale::UNKNOWN => "unknown"
           }
    )*/
    Debug::fmt(self, f)
  }
}


const TOPOCENTER_INFO: Info = Info::new(
  "Topocenter",
  "The location of the instrument that made the observation",
);
const GEOCENTER_INFO: Info = Info::new(
  "Geocenter",
  "The center of the earth",
);
const BARYCENTER_INFO: Info = Info::new(
  "Solar System Barycenter",
  "The barycenter of the solar system",
);
const HELIOCENTER_INFO: Info = Info::new(
  "Heliocenter",
  "The center of the sun",
);
const EMBARYCENTER_INFO: Info = Info::new(
  "Earth-Moon Barycenter",
  "The barycenter of the Earth-Moon system",
);
const UNKNOWN_REFPOS_INFO: Info = Info::new(
  "Unknown",
  "The times cannot be transformed to a different reference position reliably.  This is to be used for simulated data or for data for which the reference position has been lost.",
);

/// See the [IVOA refposition vocabulary](https://www.ivoa.net/rdf/refposition/2019-03-15/refposition.html)
// #[serde(tag = "refposition")]
#[derive(Clone,  PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum RefPosition {
  TOPOCENTER,
  GEOCENTER,
  BARYCENTER,
  HELIOCENTER,
  EMBARYCENTER,
  UNKNOWN,
}

impl RefPosition {
  pub const fn info(&self) -> Info {
    match self {
      RefPosition::TOPOCENTER => TOPOCENTER_INFO,
      RefPosition::GEOCENTER => GEOCENTER_INFO,
      RefPosition::BARYCENTER => BARYCENTER_INFO,
      RefPosition::HELIOCENTER => HELIOCENTER_INFO,
      RefPosition::EMBARYCENTER => EMBARYCENTER_INFO,
      RefPosition::UNKNOWN => UNKNOWN_REFPOS_INFO,
    }
  }
}

impl FromStr for RefPosition {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "topocenter" | "TOPOCENTER" => Ok(RefPosition::TOPOCENTER),
      "geocenter" | "GEOCENTER" => Ok(RefPosition::GEOCENTER),
      "barycenter" | "BARYCENTER" => Ok(RefPosition::BARYCENTER),
      "heliocenter" | "HELIOCENTER" => Ok(RefPosition::HELIOCENTER),
      "embarycenter" | "EMBARYCENTER" => Ok(RefPosition::EMBARYCENTER),
      "unknown" | "UNKNOWN" => Ok(RefPosition::UNKNOWN),
      _ => Err(format!("Unknown 'refposition' variant. Actual: '{}'.", s))
    }
  }
}

impl fmt::Display for RefPosition {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    /*write!(f, "{}",
           match self {
             RefPosition::TOPOCENTER => "topocenter",
             RefPosition::GEOCENTER => "geocenter",
             RefPosition::BARYCENTER => "barycenter",
             RefPosition::HELIOCENTER => "heliocenter",
             RefPosition::EMBARYCENTER => "embarycenter",
             RefPosition::UNKNOWN => "unknown"
           }
    )*/
    Debug::fmt(self, f)
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{Reader, events::Event, Writer};
  
  use crate::{
    QuickXmlReadWrite,
    timesys::{TimeSys, RefPosition, TimeScale}
  };

  #[test]
  fn test_timesys_readwrite() {
    let xml = r#"<TIMESYS ID="time_frame" timeorigin="2455197.5" timescale="TCB" refposition="BARYCENTER"/>"#;
    // Test read
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    let mut timesys = loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Empty(ref mut e) if e.local_name() == TimeSys::TAG_BYTES => {
          let timesys = TimeSys::from_attributes(e.attributes()).unwrap();
          assert_eq!(timesys.id, "time_frame");
          assert_eq!(timesys.timeorigin, Some(2455197.5));
          assert_eq!(timesys.timescale, TimeScale::TCB);
          assert_eq!(timesys.refposition, RefPosition::BARYCENTER);
          break timesys;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        _ => unreachable!(),
      }
    };
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    timesys.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }
}
