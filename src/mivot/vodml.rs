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

use super::{globals::Globals, model::Model, report::Report, templates::Templates};
use crate::{error::VOTableError, is_empty, QuickXmlReadWrite};
use paste::paste;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde_json::Value;
use std::collections::HashMap;
use std::str;

/*
    struct Vodml
    @elem report: Tells the client whether the annotation process succeeded or not.
    @elem models: Declares which models are represented in the file.
    @elem globals: Holds model instances or collections of model instances that are not connected with any table column.
    @elem templates: Defines a template for deriving multiple data model instances, one for each row of the associated VOTable TABLE.
    @elem extra: extra attributes that may be present while reading like "xlmns".
*/
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
        Ok(vodml)
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

///////////////////////
// UTILITY FUNCTIONS //

/*
    function read_vodml_sub_elem
    Description:
    *   reads the children of Vodml
    @generic R: BufRead; a struct that implements the std::io::BufRead trait.
    @param vodml &mut Vodml: an instance of Vodml
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
                    if vodml.globals.is_empty() {
                        vodml
                            .globals
                            .push(from_event_start!(Globals, reader, reader_buff, e))
                    } else {
                        return Err(VOTableError::Custom(
                            "Only one <GLOBALS> tag should be present".to_owned(),
                        ));
                    }
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
                Report::TAG_BYTES => {
                    if vodml.report.is_none() {
                        vodml.report = Some(Report::from_event_empty(e)?)
                    }
                }
                Model::TAG_BYTES => vodml.models.push(Model::from_event_empty(e)?),
                Globals::TAG_BYTES => {
                    if vodml.globals.is_empty() {
                        vodml.globals.push(Globals::from_event_empty(e)?)
                    } else {
                        return Err(VOTableError::Custom(
                            "Only one <GLOBALS> tag should be present".to_owned(),
                        ));
                    }
                }
                Templates::TAG_BYTES => vodml.templates.push(Templates::from_event_empty(e)?),
                _ => {
                    return Err(VOTableError::UnexpectedEmptyTag(
                        e.local_name().to_vec(),
                        Vodml::TAG,
                    ))
                }
            },
            Event::Text(e) if is_empty(e) => {}
            Event::End(e) if e.local_name() == Vodml::TAG_BYTES => {
                if !vodml.models.is_empty() {
                    return Ok(reader);
                } else {
                    return Err(VOTableError::Custom(
                        "Expected a <MODEL> tag, none was found".to_owned(),
                    ));
                }
            }
            Event::Eof => return Err(VOTableError::PrematureEOF(Vodml::TAG)),
            _ => eprintln!("Discarded event in {}: {:?}", Vodml::TAG, event),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mivot::test::test_error,
        mivot::{vodml::Vodml, test::get_xml},
        tests::{test_read},
    };

    #[test]
    fn test_vodml_read() {
        // OK VODMLS
        let xml = get_xml("./resources/mivot/1/test_1_ok_1.1.xml");
        println!("testing 1.1");
        test_read::<Vodml>(&xml);
        let xml = get_xml("./resources/mivot/1/test_1_ok_1.2.xml");
        println!("testing 1.2");
        test_read::<Vodml>(&xml);
        let xml = get_xml("./resources/mivot/1/test_1_ok_1.3.xml");
        println!("testing 1.3");
        test_read::<Vodml>(&xml);
        let xml = get_xml("./resources/mivot/1/test_1_ok_1.4.xml");
        println!("testing 1.4");
        test_read::<Vodml>(&xml);
        let xml = get_xml("./resources/mivot/1/test_1_ok_1.8.xml");
        println!("testing 1.8");
        test_read::<Vodml>(&xml);
        let xml = get_xml("./resources/mivot/1/test_1_ok_1.9.xml");
        println!("testing 1.9");
        test_read::<Vodml>(&xml);
        
        // KO VODMLS
        let xml = get_xml("./resources/mivot/1/test_1_ko_1.5.xml");
        println!("testing 1.5"); // MODEL required
        test_error::<Vodml>(&xml, false);
        let xml = get_xml("./resources/mivot/1/test_1_ko_1.6.xml");
        println!("testing 1.6"); // MODEL subnode must be first (parser can overlook this and write it correctly later)
        test_read::<Vodml>(&xml); // Should read correctly
        let xml = get_xml("./resources/mivot/1/test_1_ko_1.7.xml");
        println!("testing 1.7"); // GLOBALS must be after MODEL and before TEMPLATES (parser can overlook this and write it correctly later)
        test_read::<Vodml>(&xml); // Should read correctly
        let xml = get_xml("./resources/mivot/1/test_1_ko_1.10.xml");
        println!("testing 1.10"); // Only 1 GLOBALS subnode allowed.
        test_error::<Vodml>(&xml, false);
        let xml = get_xml("./resources/mivot/1/test_1_ko_1.11.xml");
        println!("testing 1.11"); // Includes invalid subnode.
        test_error::<Vodml>(&xml, false);
    }
}
