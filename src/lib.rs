extern crate core;

/// Metadata, in addition to the `schema` information, possibly used in the header part of
/// a file.
/// The `schema` information are mandatory to be able to read/write data in the various file format.
/// The `metadata` here contains additional optional information (such as units, UCDs, descriptions,
/// ...).
/// The `metadata` may contains redundancies with `schema`. If it is the case, the metadata
/// *MUST* be overwritten by the `schema` (the schema contains minimal, yet very important, information).
///
/// `schema` + `metadata` => any format meta
///
/// `metadata` my be complex, we would like to build it from a simpler (e.x. TOML) files. 
///
/// We used first the `VOTable` format to defined a `metadata` set, add other elements, 
/// plus a mechanism to allow for custom metadata.

use std::io::{BufRead, Write};

use quick_xml::{
    Reader, Writer,
    events::{BytesStart, BytesText, attributes::Attributes}
};

#[macro_use]
pub mod macros;
pub mod error;
pub mod impls;
pub mod coosys;
pub mod data;
pub mod datatype;
pub mod desc;
pub mod definitions;
pub mod field;
pub mod fieldref;
pub mod group;
pub mod info;
pub mod link;
pub mod param;
pub mod paramref;
pub mod resource;
pub mod table;
pub mod timesys;
pub mod values;
pub mod votable;

pub mod iter;

use error::VOTableError;
use table::TableElem;

pub trait TableDataContent: Default + serde::Serialize { //+ serde::Deserialize<'de> {

    fn new() -> Self {
        Self::default()
    }

    /// Called when Event::Start("DATATABLE") as been detected and **MUST**
    /// return after event Event::End("DATATABLE")
    fn read_datatable_content<R: BufRead>(
        &mut self, reader: Reader<R>, reader_buff: &mut Vec<u8>, context: &[TableElem]
    ) -> Result<Reader<R>, VOTableError>;

    /// Called when Event::Start("STREAM") as been detected (in BINARY) and **MUST**
    /// return after event Event::End("STREAM")
    fn read_binary_content<R: BufRead>(
        &mut self, reader: Reader<R>, reader_buff: &mut Vec<u8>, context: &[TableElem]
    ) -> Result<Reader<R>, VOTableError>;

    /// Called when Event::Start("STREAM") as been detected (in BINARY2) and **MUST**
    /// return after event Event::End("STREAM")
    fn read_binary2_content<R: BufRead>(
        &mut self, reader: Reader<R>, reader_buff: &mut Vec<u8>, context: &[TableElem]
    ) -> Result<Reader<R>, VOTableError>;

    fn write_in_datatable<W: Write>(
        &mut self, writer: &mut Writer<W>, context: &[TableElem]
    ) -> Result<(), VOTableError>;

    fn write_in_binary<W: Write>(
        &mut self, writer: &mut Writer<W>, context: &[TableElem]
    ) -> Result<(), VOTableError>;

    fn write_in_binary2<W: Write>(
        &mut self, writer: &mut Writer<W>, context: &[TableElem]
    ) -> Result<(), VOTableError>;

}


trait QuickXmlReadWrite: Sized {
    const TAG: &'static str;
    const TAG_BYTES: &'static [u8] = Self::TAG.as_bytes();
    type Context;

    fn from_event_empty(e: &BytesStart) -> Result<Self, VOTableError> {
        Self::from_attributes(e.attributes())
    }

    /// We assume that the previous event was either `Start` or `Empty`.
    fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError>;

    /// Same as `read_sub_elements`, cleaning the `reader_buf` before returning.
    fn read_sub_elements_and_clean<R: BufRead>(
        &mut self,
        reader: Reader<R>,
        reader_buff: &mut Vec<u8>,
        context: &Self::Context,
    ) -> Result<Reader<R>, VOTableError> {
        let res = self.read_sub_elements(reader, reader_buff, context);
        reader_buff.clear();
        res
    }

    /// We assume that the previous event was `Start`, and that the method returns
    /// when encountering the `End` event matching the last `Start` event before entering the method.
    fn read_sub_elements<R: BufRead>(
        &mut self,
        reader: Reader<R>,
        reader_buff: &mut Vec<u8>,
        context: &Self::Context,
    ) -> Result<Reader<R>, VOTableError>;
    
    /// Same as `read_sub_elements`, cleaning the `reader_buf` before returning.
    fn read_sub_elements_and_clean_by_ref<R: BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        reader_buff: &mut Vec<u8>,
        context: &Self::Context,
    ) -> Result<(), VOTableError> {
        let res = self.read_sub_elements_by_ref(reader, reader_buff, context);
        reader_buff.clear();
        res
    }

    /// We assume that the previous event was `Start`, and that the method returns
    /// when encountering the `End` event matching the last `Start` event before entering the method.
    fn read_sub_elements_by_ref<R: BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        reader_buff: &mut Vec<u8>,
        context: &Self::Context,
    ) -> Result<(), VOTableError>;
    
    /// `&mut self` in case internals are modified while writing (e.g. if we iterate on rows
    /// and discard them as we iterate).
    /// We could add a context, e.g. to modify the parent (adding infos for example).
    fn write<W: Write>(
        &mut self, 
        writer: &mut Writer<W>,
        context: &Self::Context,
    ) -> Result<(), VOTableError>;
}


pub(crate) fn is_empty(text: &BytesText) -> bool {
  for byte in text.escaped() {
      if *byte != b' ' && *byte != b'\n' && *byte != b'\t' {
          return false;
      }
  }
  true
}

// For Javascript, see https://rustwasm.github.io/wasm-bindgen/reference/arbitrary-data-with-serde.html


#[cfg(test)]
mod tests {
    use std::str::from_utf8;
    use quick_xml::Writer;
    use serde_json::{Number, Value};
    use crate::data::Data;
    use crate::impls::VOTableValue;
    use super::{
        QuickXmlReadWrite,
        datatype::Datatype,
        info::Info,
        link::Link,
        table::Table,
        values::Values,
        resource::Resource,
        field::{Field, Precision},
        coosys::{System, CooSys},
        votable::{Version, VOTable},
        impls::mem::InMemTableDataRows,
    };
    
    #[test]
    fn test_create_in_mem() {
        let rows = vec![
            vec![
                VOTableValue::Double(f64::NAN), 
                VOTableValue::String("*".to_owned()),
                VOTableValue::Long(i64::MAX)
            ],
            vec![
                VOTableValue::Double(0.4581e+38),
                VOTableValue::String("".to_owned()),
                VOTableValue::Long(i64::MIN)
            ],
            vec![
                VOTableValue::Null,
                VOTableValue::String("*".to_owned()),
                VOTableValue::Long(0)
            ],
        ];
        let data_content = InMemTableDataRows::new(rows);
        
        let table = Table::new()
          .set_id("V_147_sdss12")
          .set_name("V/147/sdss12")
          .set_description("* output of the SDSS photometric catalog".into())
          .push_field(
              Field::new("RA_ICRS", Datatype::Double)
                .set_unit("deg")
                .set_ucd("pos.eq.ra;meta.main")
                .set_ref("H")
                .set_width(10)
                .set_precision(Precision::new_dec(6))
                .set_description("Right Ascension of the object (ICRS) (ra)".into())
                .insert_extra("toto", Number::from_f64(0.5).map(Value::Number).unwrap_or(Value::Null))
          ).push_field(
            Field::new("m_SDSS12", Datatype::CharASCII)
              .set_ucd("meta.code.multip")
              .set_width(1)
              .set_description("[*] The asterisk indicates that 2 different SDSS objects share the same SDSS12 name".into())
              .push_link(Link::new().set_href("http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-info=XML&amp;-out.add=.&amp;-source=V/147&amp;SDSS12=${SDSS12}"))
        ).push_field(
            Field::new("umag", Datatype::LongInt)
              .set_unit("mag")
              .set_ucd("phot.mag;em.opt.U")
              .set_description("[4/38]? Model magnitude in u filter, AB scale (u) (5)".into())
              .set_values(Values::new().set_null("NaN"))
        ).set_data(Data::new_empty().set_tabledata(data_content));

        let resource = Resource::default()
          .set_id("yCat_17011219")
          .set_name("J/ApJ/701/1219")
          .set_description(r#"Photometric and spectroscopic catalog of objects in the field around HE0226-4110"#.into())
          .push_coosys(CooSys::new("J2000", System::new_default_eq_fk5()))
          .push_coosys(CooSys::new("J2015.5", System::new_icrs().set_epoch(2015.5)))
          .insert_extra("toto", Number::from_f64(0.5).map(Value::Number).unwrap_or(Value::Null))
          .push_table(table)
          .push_post_info(Info::new("matches", "50").set_content("matching records"))
          .push_post_info(Info::new("Warning", "No center provided++++"))
          .push_post_info(Info::new("Warning", "truncated result (maxtup=50)"))
          .push_post_info(Info::new("QUERY_STATUS", "OVERFLOW").set_content("truncated result (maxtup=50)"));

        let mut votable = VOTable::new(resource)
          .set_id("my_votable")
          .set_version(Version::V1_4)
          .set_description(r#"
VizieR Astronomical Server vizier.u-strasbg.fr
Date: 2022-04-13T06:55:08 [V1.99+ (14-Oct-2013)]
Explanations and Statistics of UCDs:			See LINK below
In case of problem, please report to:	cds-question@unistra.fr
In this version, NULL integer columns are written as an empty string
&lt;TD&gt;&lt;/TD&gt;, explicitely possible from VOTable-1.3
"#.into()
          )
          .push_info(Info::new("votable-version", "1.99+ (14-Oct-2013)").set_id("VERSION"))
          .push_info(Info::new("queryParameters", "25")
            .set_content(r#"
-oc.form=dec
-out.max=50
-out.all=2
-nav=cat:J/ApJ/701/1219&amp;tab:{J/ApJ/701/1219/table4}&amp;key:source=J/ApJ/701/1219&amp;HTTPPRM:&amp;
-c.eq=J2000
-c.r=  2
-c.u=arcmin
-c.geom=r
-source=J/ApJ/701/1219/table4
-order=I
-out=ID
-out=RAJ2000
-out=DEJ2000
-out=Sep
-out=Dist
-out=Bmag
-out=e_Bmag
-out=Rmag
-out=e_Rmag
-out=Imag
-out=e_Imag
-out=z
-out=Type
-out=RMag
-out.all=2
    "#));
        
        println!("\n\n#### JSON ####\n");

        match serde_json::to_string_pretty(&votable) {
            Ok(content) => {
                // println!("{}", &content);
                let votable2 = serde_json::de::from_str::<VOTable<InMemTableDataRows>>(content.as_str()).unwrap();
                let content2 = serde_json::to_string_pretty(&votable2).unwrap();
                assert_eq!(content, content2);
            },
            Err(error) => {
                println!("{:?}", &error);
                assert!(false);
            },
        }

        println!("\n\n#### YAML ####\n");

        match serde_yaml::to_string(&votable) {
            Ok(content) => {
                // println!("{}", &content);
                let votable2 = serde_yaml::from_str::<VOTable<InMemTableDataRows>>(content.as_str()).unwrap();
                let content2 = serde_yaml::to_string(&votable2).unwrap();
                assert_eq!(content, content2);
            },
            Err(error) => {
                println!("{:?}", &error);
                assert!(false);
            },
        }

        println!("\n\n#### VOTABLE ####\n");

        let mut content = Vec::new();
        let mut write = Writer::new_with_indent(/*stdout()*/ &mut content, b' ', 4);
        match votable.write(&mut write, &()) {
            Ok(_) => {
                let mut votable2 = VOTable::<InMemTableDataRows>::from_reader(content.as_slice()).unwrap();
                let mut content2 =  Vec::new();
                let mut write2 = Writer::new_with_indent(&mut content2, b' ', 4);
                votable2.write(&mut write2, &()).unwrap();

                eprintln!("CONTENT1:\n{}", from_utf8(content.as_slice()).unwrap());
                eprintln!("CONTENT2:\n{}", from_utf8(content2.as_slice()).unwrap());

                assert_eq!(content, content2);
            },
            Err(error) => {
                println!("Error: {:?}", &error);
                assert!(false);
            },
        }

        println!("\n\n#### TOML ####\n");

        match toml::ser::to_string_pretty(&votable) {
            Ok(content) => {
                // println!("{}", &content);
                let votable2 = toml::de::from_str::<VOTable<InMemTableDataRows>>(content.as_str()).unwrap();
                let content2 = toml::ser::to_string_pretty(&votable2).unwrap();
                assert_eq!(content, content2);
            },
            Err(error) => {
                println!("{:?}", &error);
                assert!(false);
            },
        }
        
        /*println!("\n\n#### XML ####\n");
    
        match quick_xml::se::to_string(&votable) {
          Ok(content) => println!("{}", &content),
          Err(error) => println!("{:?}", &error),
        }*/

        // AVRO ?
    }

    /* not a test, used for the README.md
    #[test]
    fn test_create_in_mem_simple() {
        let rows = vec![
            vec![VOTableValue::Double(f64::NAN), VOTableValue::CharASCII('*'), VOTableValue::Float(14.52)],
            vec![VOTableValue::Double(1.25), VOTableValue::Null, VOTableValue::Float(-1.2)],
        ];
        let data_content = InMemTableDataRows::new(rows);
        let table = Table::new()
          .set_id("V_147_sdss12")
          .set_name("V/147/sdss12")
          .set_description("SDSS photometric catalog".into())
          .push_field(
              Field::new("RA_ICRS", Datatype::Double)
                .set_unit("deg")
                .set_ucd("pos.eq.ra;meta.main")
                .set_width(10)
                .set_precision(Precision::new_dec(6))
                .set_description("Right Ascension of the object (ICRS) (ra)".into())
          ).push_field(
            Field::new("m_SDSS12", Datatype::CharASsCII)
              .set_ucd("meta.code.multip")
              .set_arraysize("1")
              .set_width(10)
              .set_precision(Precision::new_dec(6))
              .set_description("[*] Multiple SDSS12 name".into())
              .push_link(Link::new().set_href("http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-info=XML&-out.add=.&-source=V/147&SDSS12=${SDSS12}"))
        ).push_field(
            Field::new("umag", Datatype::Float)
              .set_unit("mag")
              .set_ucd("phot.mag;em.opt.U")
              .set_width(2)
              .set_precision(Precision::new_dec(3))
              .set_description("[4/38]? Model magnitude in u filter, AB scale (u) (5)".into())
              .set_values(Values::new().set_null("NaN"))
        ).set_data(Data::new_empty().set_tabledata(data_content));

        let resource = Resource::default()
          .set_id("yCat_17011219")
          .set_name("J/ApJ/701/1219")
          .set_description(r#"Photometric and spectroscopic catalog of objects in the field around HE0226-4110"#.into())
          .push_coosys(CooSys::new("J2000", System::new_default_eq_fk5()))
          .push_coosys(CooSys::new("J2015.5", System::new_icrs().set_epoch(2015.5)))
          .push_table(table)
          .push_post_info(Info::new("QUERY_STATUS", "OVERFLOW").set_content("truncated result (maxtup=2)"));

        let mut votable = VOTable::new(resource)
          .set_id("my_votable")
          .set_version(Version::V1_4)
          .set_description(r#"VizieR Astronomical Server vizier.u-strasbg.fr"#.into())
          .push_info(Info::new("votable-version", "1.99+ (14-Oct-2013)").set_id("VERSION"))
          .wrap();

        println!("\n\n#### JSON ####\n");

        match serde_json::to_string_pretty(&votable) {
            Ok(content) => {
                println!("{}", &content);
            },
            Err(error) => println!("{:?}", &error),
        }

        println!("\n\n#### YAML ####\n");

        match serde_yaml::to_string(&votable) {
            Ok(content) => {
                println!("{}", &content);
            },
            Err(error) => println!("{:?}", &error),
        }

        println!("\n\n#### TOML ####\n");

        match toml::ser::to_string_pretty(&votable) {
            Ok(content) => {
                println!("{}", &content);
            },
            Err(error) => println!("{:?}", &error),
        }

        println!("\n\n#### VOTABLE ####\n");

        let mut write = Writer::new_with_indent(stdout(), b' ', 4);
        match votable.unwrap().write(&mut write, &()) {
            Ok(content) => println!("\nOK"),
            Err(error) => println!("Error: {:?}", &error),
        }

        /*println!("\n\n#### XML ####\n");

        match quick_xml::se::to_string(&votable) {
          Ok(content) => println!("{}", &content),
          Err(error) => println!("{:?}", &error),
        }*/

        // AVRO ?
    }*/
}
