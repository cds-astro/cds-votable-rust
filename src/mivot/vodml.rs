/*
DOC FOR VODML
Spec MIVOT
https://github.com/ivoa-std/ModelInstanceInVot

parser
https://github.com/ivoa/modelinstanceinvot-code

Groupe de travail sur l'implémentation d'une API astropy
https://github.com/ivoa/modelinstanceinvot-code/wiki
les deux derniers items de Hack-a-thon

wiki API
https://github.com/ivoa/modelinstanceinvot-code/wiki/guideline

service:
https://xcatdb.unistra.fr/xtapdb

RFC:
https://wiki.ivoa.net/twiki/bin/view/IVOA/DataAnnotation <= dead link

Meas
https://ivoa.net/documents/Meas/20211019/index.html
*/

// pos {
//   id: String (ex: pos.eq.main)
//   sys: Option<eq>,
//   ra:  FIELDRef
//   dec: FIELDRef
// }

// pos.err {
//   pos: Option<MODELRef>
//   type:
//   params (depends on type)
// }
// Get system associated to error: error.pos.sys

use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use paste::paste;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde_json::Value;
use std::collections::HashMap;
use std::str;

use super::{globals::Globals, model::Model, report::Report, templates::Templates};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Vodml {
  #[serde(skip_serializing_if = "Option::is_none")]
  report: Option<Report>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  models: Vec<Model>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  globals: Vec<Globals>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  templates: Vec<Templates>,
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
}
impl Vodml {
  impl_builder_opt_attr!(report, Report);
  impl_builder_insert_extra!();
}
impl QuickXmlReadWrite for Vodml {
  const TAG: &'static str = "VODML";
  type Context = ();

  fn from_attributes(
    attrs: quick_xml::events::attributes::Attributes,
  ) -> Result<Self, crate::error::VOTableError> {
    let mut vodml = Self::default();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      vodml = match attr.key {
        _ => vodml.insert_extra(
          str::from_utf8(attr.key).map_err(VOTableError::Utf8)?,
          Value::String(value.into()),
        ),
      }
    }

    non_empty_read_sub!(read_vodml_sub_elem);

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut quick_xml::Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
        writer
            .write_event(Event::Start(tag.to_borrowed()))
            .map_err(VOTableError::Write)?;
        write_elem_vec_empty_context!(self, report, writer);
        write_elem_vec_empty_context!(self, models, writer);
        write_elem_vec_empty_context!(self, globals, writer);
        write_elem_vec_empty_context!(self, templates, writer);
        writer
            .write_event(Event::End(tag.to_end()))
            .map_err(VOTableError::Write)
    }
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut quick_xml::Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    todo!()
  }

  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut quick_xml::Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    write_elem_vec_empty_context!(self, report, writer);
    write_elem_vec_empty_context!(self, models, writer);
    write_elem_vec_empty_context!(self, globals, writer);
    write_elem_vec_empty_context!(self, templates, writer);
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
  }
}

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_vodml_sub_elem
    Description:
    *   reads the children of Vodml
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @generic T: QuickXMLReadWrite + ElemImpl<InstanceElem>; a struct that implements the quickXMLReadWrite and ElemImpl for InstanceElem traits.
    @param vodml &mut T: an instance of T (here either GlobOrTempInstance or Vodml)
    @param reader &mut quick_xml::Reader<R>: the reader used to read the elements
    @param reader &mut &mut Vec<u8>: a buffer used to read events [see read_event function from quick_xml::Reader]
    #returns Result<quick_xml::Reader<R>, VOTableError>: returns the Reader once finished or an error if reading doesn't work
*/
fn read_vodml_sub_elem<R: std::io::BufRead>(
    vodml: &mut Vodml,
    _context: &(),
    mut reader: quick_xml::Reader<R>,
    mut reader_buff: &mut Vec<u8>,
) -> Result<quick_xml::Reader<R>, VOTableError> {
    loop {
        let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
        println!("{:?}", event);
        match &mut event {
            Event::Start(ref e) => match e.local_name() {
                Report::TAG_BYTES => {
                    if vodml.report.is_none() {
                        vodml.report = Some(from_event_start!(Report, reader, reader_buff, e))
                    }
                }
                Globals::TAG_BYTES => {
                    vodml
                        .globals
                        .push(from_event_start!(Globals, reader, reader_buff, e))
                }
                Templates::TAG_BYTES => {
                    vodml
                        .templates
                        .push(from_event_start!(Templates, reader, reader_buff, e))
                }
                _ => {
                    return Err(VOTableError::UnexpectedStartTag(
                        e.local_name().to_vec(),
                        Vodml::TAG,
                    ))
                }
            },
            Event::Empty(ref e) => match e.local_name() {
                Model::TAG_BYTES => vodml.models.push(Model::from_event_empty(e)?),
                _ => {
                    return Err(VOTableError::UnexpectedEmptyTag(
                        e.local_name().to_vec(),
                        Vodml::TAG,
                    ))
                }
            },
            Event::Text(e) if is_empty(e) => {}
            Event::End(e) if e.local_name() == Vodml::TAG_BYTES => return Ok(reader),
            Event::Eof => return Err(VOTableError::PrematureEOF(Vodml::TAG)),
            _ => eprintln!("Discarded event in {}: {:?}", Vodml::TAG, event),
        }
    }
}

#[cfg(test)]
mod tests {
  use crate::{mivot::vodml::Vodml, tests::test_read};

  #[test]
  fn test_vodml_read_write() {
    let xml = r#"<VODML xmlns:dm-mapping="http://www.ivoa.net/xml/merged-syntax">
        <REPORT status="OK">hand-made mapping</REPORT>
        <MODEL name="meas" url="https://www.ivoa.net/xml/Meas/20200908/Meas-v1.0.vo-dml.xml" />
        <MODEL name="coords" url="https://www.ivoa.net/xml/STC/20200908/Coords-v1.0.vo-dml.xml" />
        <MODEL name="ivoa" url="https://www.ivoa.net/xml/VODML/IVOA-v1.vo-dml.xml" />
        <GLOBALS>
          <INSTANCE dmid="SpaceFrame_ICRS" dmtype="coords:SpaceSys">
            <INSTANCE dmrole="coords:PhysicalCoordSys.frame" dmtype="coords:SpaceFrame">
              <ATTRIBUTE dmrole="coords:SpaceFrame.spaceRefFrame" dmtype="ivoa:string" value="ICRS" />
              <INSTANCE dmrole="coords:SpaceFrame.refPosition" dmtype="coords:CustomRefLocation">
                <ATTRIBUTE dmrole="coords:CustomRefLocation.epoch" dmtype="coords:Epoch" value="2015.0"/>
                <REFERENCE dmrole="coords:Coordinate.coordSys" dmref="\_timesys"/>
              </INSTANCE>
            </INSTANCE>
          </INSTANCE>
        </GLOBALS>
        <TEMPLATES tableref="Results">
          <INSTANCE dmtype="meas:Position">
            <ATTRIBUTE dmrole="meas:Measure.ucd" dmtype="ivoa:string" value="pos" />
            <INSTANCE dmrole="meas:Measure.coord" dmtype="coords:LonLatPoint">
              <ATTRIBUTE dmtype="ivoa:RealQuantity" dmrole="coords:LonLatPoint.lon" ref="ra" unit="deg"/>
              <ATTRIBUTE dmtype="ivoa:RealQuantity" dmrole="coords:LonLatPoint.lat" ref="dec" unit="deg"/>
              <ATTRIBUTE dmtype="ivoa:RealQuantity" dmrole="coords:LonLatPoint.dist" ref="parallax" unit="parsec"/>
              <REFERENCE dmrole="coords:Coordinate.coordSys" dmref="SpaceFrame_ICRS" />
            </INSTANCE>
            <INSTANCE dmrole="meas:Measure.error" dmtype="meas:Ellipse">
              <ATTRIBUTE dmrole="meas:Ellipse.posAngle" dmtype="meas:Ellipse" value="0"/>
              <COLLECTION dmrole="meas:Ellipse.semiAxis">
                <ATTRIBUTE dmtype="ivoa:RealQuantity" ref="ra_error" unit="mas"/>
                <ATTRIBUTE dmtype="ivoa:RealQuantity" ref="dec_error" unit="mas"/>
              </COLLECTION>
            </INSTANCE>
          </INSTANCE>
          <INSTANCE dmtype="meas:Velocity">
            <ATTRIBUTE dmrole="meas:Measure.ucd" dmtype="ivoa:string" value="spect.dopplerVeloc.opt" />
            <INSTANCE dmrole="meas:Measure.coord" dmtype="coords:LonLatPoint">
              <ATTRIBUTE dmtype="ivoa:RealQuantity" dmrole="coords:LonLatPoint.dist"
                         ref="radial_velocity" unit="km/s"/>
            </INSTANCE>
            <ATTRIBUTE dmrole="meas:Measure.error" dmtype="meas:Symmetrical"
                       ref="radial_velocity_error" unit="km/s"/>
           </INSTANCE>
          <INSTANCE dmtype="meas:GenericMeasure">
            <ATTRIBUTE dmrole="meas:Measure.ucd" dmtype="ivoa:string" value="pos.parallax" />
            <INSTANCE dmrole="meas:Measure.coord" dmtype="coords:PhysicalCoordinate">
              <ATTRIBUTE dmrole="coords:PhysicalCoordinate.cval" dmtype="ivoa:RealQuantity" ref="parallax" unit="mas"/>
            </INSTANCE>
            <ATTRIBUTE dmrole="meas:Measure.error" dmtype="meas:Symmetrical" ref="parallax_error" unit="mas"/>
          </INSTANCE>
          <INSTANCE dmtype="meas:ProperMotion">
            <ATTRIBUTE dmrole="meas:Measure.ucd" dmtype="ivoa:string" value="pos.pm" />
            <INSTANCE dmrole="meas:Measure.coord" dmtype="coords:LonLatPoint">
              <ATTRIBUTE dmrole="coords:LonLatPoint.lon" dmtype="ivoa:RealQuantity" ref="pmra" unit="mas/year"/>
              <ATTRIBUTE dmrole="coords:LonLatPoint.lat" dmtype="ivoa:RealQuantity" ref="pmdec" unit="mas/year"/>
              <ATTRIBUTE dmrole="meas:ProperMotion.cosLat_applied" dmtype="ivoa:bool" value="true" />
            </INSTANCE>
            <INSTANCE dmrole="meas:Measure.error" dmtype="meas:Ellipse">
              <ATTRIBUTE dmrole="meas:Ellipse.posAngle" dmtype="meas:Ellipse" value="0"/>
              <COLLECTION dmrole="meas:Ellipse.semiAxis">
                <ATTRIBUTE dmtype="ivoa:RealQuantity" ref="pmra_error" unit="mas/year"/>
                <ATTRIBUTE dmtype="ivoa:RealQuantity" ref="pmdec_error" unit="mas/year"/>
              </COLLECTION>
            </INSTANCE>
          </INSTANCE>
        </TEMPLATES>
      </VODML>"#; // Test read
    let _vodml = test_read::<Vodml>(xml);
  }
}
