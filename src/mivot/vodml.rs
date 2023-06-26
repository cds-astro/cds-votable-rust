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
        let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
        push2write_extra!(self, tag);
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
    use std::str::from_utf8;

    use crate::{
        mivot::{vodml::Vodml},
        tests::{test_read, test_writer},
    };

    #[test]
    fn test_vodml_read_write() {
        let xml = r#"<VODML xmlns="http://www.ivoa.net/xml/mivot" ><REPORT status="OK">Mapping compiled by hand</REPORT><MODEL name="ivoa" url="https://www.ivoa.net/xml/VODML/IVOA-v1.vo-dml.xml" /><MODEL name="mango" url="https://github.com/ivoa-std/MANGO/blob/master/vo-dml/mango.vo-dml.xml" /><MODEL name="cube" url="https://github.com/ivoa-std/Cube/vo-dml/Cube-1.0.vo-dml.xml" /><MODEL name="ds" url="https://github.com/ivoa-std/DatasetMetadata/vo-dml/DatasetMetadata-1.0.vo-dml.xml" /><MODEL name="coords" url="https://www.ivoa.net/xml/VODML/Coords-v1.vo-dml.xml" /><MODEL name="meas" url="https://www.ivoa.net/xml/VODML/Meas-v1.vo-dml.xml" /><GLOBALS><COLLECTION dmid="\_CoordinateSystems" dmrole="" ><INSTANCE dmid="\_timesys" dmrole="" dmtype="coords:TimeSys"><PRIMARY\_KEY dmtype="ivoa:string" value="TCB"/><INSTANCE dmrole="coords:PhysicalCoordSys.frame" dmtype="coords:TimeFrame"><ATTRIBUTE dmrole="coords:TimeFrame.timescale" dmtype="ivoa:string" value="TCB" /><INSTANCE dmrole="coords:TimeFrame.refPosition" dmtype="coords:StdRefLocation"><ATTRIBUTE dmrole="coords:StdRefLocation.position" dmtype="ivoa:string" value="BARYCENTER"/></INSTANCE></INSTANCE></INSTANCE><INSTANCE dmid="\_spacesys1" dmrole="" dmtype="coords:SpaceSys"><PRIMARY\_KEY dmtype="ivoa:string" value="ICRS"/><INSTANCE dmrole="coords:PhysicalCoordSys.frame" dmtype="coords:SpaceFrame"><ATTRIBUTE dmrole="coords:SpaceFrame.spaceRefFrame" dmtype="ivoa:string" value="ICRS"/><ATTRIBUTE dmrole="coords:SpaceFrame.equinox" dmtype="coords:Epoch" value="J2015.5"/></INSTANCE></INSTANCE><INSTANCE dmid="\_photsys\_G" dmtype="mango:coordinates.PhotometryCoordSys"><PRIMARY\_KEY dmtype="ivoa:string" value="G"/><INSTANCE dmrole="coords:PhysicalCoordSys.frame" dmtype="mango:coordinates.PhotFilter"><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.name" dmtype="ivoa:string" value="GAIA/GAIA2r.G"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.zeroPointFlux" dmtype="ivoa:RealQuantity" value="2.49524e-9"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.magnitudeSystem" dmtype="ivoa:string" value="Vega"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.effectiveWavelength" dmtype="ivoa:RealQuantity" value="6246.77"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.unit" dmtype="ivoa:Unit" value="Angstrom" /><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.bandWidth" dmtype="ivoa:real" value="4578.32"/></INSTANCE></INSTANCE><INSTANCE dmid="\_photsys\_RP" dmrole="" dmtype="mango:coordinates.PhotometryCoordSys"><PRIMARY\_KEY dmtype="ivoa:string" value="RP"/><INSTANCE dmrole="coords:PhysicalCoordSys.frame" dmtype="mango:coordinates.PhotFilter"><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.name" dmtype="ivoa:string" value="GAIA/GAIA2r.Grp"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.zeroPointFlux" dmtype="ivoa:RealQuantity" value="1.29363e-9"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.magnitudeSystem" dmtype="ivoa:string" value="Vega"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.effectiveWavelength" dmtype="ivoa:RealQuantity" value="7740.87"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.unit" dmtype="ivoa:Unit" value="Angstrom"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.bandWidth" dmtype="ivoa:real" value="2943.72"/></INSTANCE></INSTANCE><INSTANCE dmid="\_photsys\_BP" dmrole="" dmtype="mango:coordinates.PhotometryCoordSys"><PRIMARY\_KEY dmtype="ivoa:string" value="BP"/><INSTANCE dmrole="coords:PhysicalCoordSys.frame" dmtype="mango:coordinates.PhotFilter"><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.name" dmtype="ivoa:string" value="GAIA/GAIA2r.Gbp"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.zeroPointFlux" dmtype="ivoa:RealQuantity" value="4.03528e-9"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.magnitudeSystem" dmtype="ivoa:string" value="Vega"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.effectiveWavelength" dmtype="ivoa:RealQuantity" value="5278.58"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.unit" dmtype="ivoa:Unit" value="Angstrom"/><ATTRIBUTE dmrole="mango:coordinates.PhotFilter.bandWidth" dmtype="ivoa:real" value="2279.45"/></INSTANCE></INSTANCE></COLLECTION><COLLECTION dmid="\_Datasets" dmrole=""><INSTANCE dmid="\_ds1" dmrole="" dmtype="ds:experiment.ObsDataset"><PRIMARY\_KEY dmtype="ivoa:string" value="5813181197970338560"/><ATTRIBUTE dmrole="ds:dataset.Dataset.dataProductType" dmtype="ds:dataset.DataProductType" value="TIMESERIES"/><ATTRIBUTE dmrole="ds:dataset.Dataset.dataProductSubtype" dmtype="ivoa:string" value="GAIA Time Series"/><ATTRIBUTE dmrole="ds:experiment.ObsDataset.calibLevel" dmtype="ivoa:integer" value="1"/><REFERENCE dmrole="ds:experiment.ObsDataset.target" dmref="\_tg1"/></INSTANCE></COLLECTION><INSTANCE dmid="\_tg1" dmrole="" dmtype="ds:experiment.Target"><ATTRIBUTE dmrole="ds:experiment.BaseTarget.name" dmtype="ivoa:string" value="5813181197970338560"/></INSTANCE></GLOBALS><TEMPLATES tableref="\_PKTable"><INSTANCE dmid="\_TimeSeries" dmrole="" dmtype="cube:SparseCube"><REFERENCE dmrole="cube:DataProduct.dataset" sourceref="\_Datasets"><FOREIGN\_KEY ref="\_pksrcid"/></REFERENCE><COLLECTION dmrole="cube:SparseCube.data"><JOIN dmref="\_ts\_data"><WHERE foreignkey="\_srcid" primarykey="\_pksrcid" /><WHERE foreignkey="\_band" primarykey="\_pkband" /></JOIN></COLLECTION></INSTANCE></TEMPLATES><TEMPLATES tableref="Results"><INSTANCE dmid="\_ts\_data" dmrole="" dmtype="cube:NDPoint"><COLLECTION dmrole="cube:NDPoint.observable"><INSTANCE dmtype="cube:Observable"><ATTRIBUTE dmrole="cube:DataAxis.dependent" dmtype="ivoa:boolean" value="False"/><INSTANCE dmrole="cube:MeasurementAxis.measure" dmtype="meas:Time"><INSTANCE dmrole="meas:Measure.coord" dmtype="coords:MJD"><ATTRIBUTE dmrole="coords:MJD.date" dmtype="ivoa:real" ref="\_obstime"/><REFERENCE dmrole="coords:Coordinate.coordSys" dmref="\_timesys"/></INSTANCE></INSTANCE></INSTANCE><INSTANCE dmtype="cube:Observable"><ATTRIBUTE dmrole="cube:DataAxis.dependent" dmtype="ivoa:boolean" value="True"/><INSTANCE dmrole="cube:MeasurementAxis.measure" dmtype="meas:GenericMeasure"><INSTANCE dmrole="meas:Measure.coord" dmtype="coords:PhysicalCoordinate"><ATTRIBUTE dmrole="coords:PhysicalCoordinate.cval" dmtype="ivoa:RealQuantity" ref="\_mag" unit="mag"/><REFERENCE dmrole="coords:Coordinate.coordSys" sourceref="\_CoordinateSystems"><FOREIGN\_KEY ref="\_band"/></REFERENCE></INSTANCE></INSTANCE></INSTANCE><INSTANCE dmtype="cube:Observable"><ATTRIBUTE dmrole="cube:DataAxis.dependent" dmtype="ivoa:boolean" value="True"/><INSTANCE dmrole="cube:MeasurementAxis.measure" dmtype="meas:GenericMeasure"><INSTANCE dmrole="meas:Measure.coord" dmtype="coords:PhysicalCoordinate"><ATTRIBUTE dmrole="coords:PhysicalCoordinate.cval" dmtype="ivoa:RealQuantity" ref="\_flux" unit="e-/s" /><REFERENCE dmrole="coords:Coordinate.coordSys" sourceref="\_CoordinateSystems"><FOREIGN\_KEY ref="\_band"/></REFERENCE></INSTANCE><INSTANCE dmrole="meas:Measure.error" dmtype="meas:Error"><INSTANCE dmrole="meas:Error.statError" dmtype="meas:Symmetrical"><ATTRIBUTE dmrole="meas:Symmetrical.radius" dmtype="ivoa:RealQuantity" ref="\_fluxerr" unit="e-/s" /></INSTANCE></INSTANCE></INSTANCE></INSTANCE></COLLECTION></INSTANCE></TEMPLATES></VODML>"#; // Test read
        let vodml = test_read::<Vodml>(xml);
        test_writer(vodml, xml);
    }
}
