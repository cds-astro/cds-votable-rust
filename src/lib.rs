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
use std::{
  error::Error,
  io::{BufRead, Write},
};

use quick_xml::{
  events::{attributes::Attributes, BytesStart},
  Reader, Writer,
};

#[macro_use]
mod macros;
mod utils;

pub mod coosys;
pub mod data;
pub mod datatype;
pub mod definitions;
pub mod desc;
pub mod error;
pub mod field;
pub mod fieldref;
pub mod group;
pub mod impls;
pub mod info;
pub mod iter;
pub mod link;
pub mod param;
pub mod paramref;
pub mod resource;
pub mod table;
pub mod timesys;
pub mod values;
pub mod votable;

#[cfg(feature = "mivot")]
pub mod mivot;

#[cfg(feature = "mivot")]
pub use self::mivot::VodmlVisitor;
pub use self::{
  coosys::CooSys,
  data::{
    binary::Binary, binary2::Binary2, fits::Fits, stream::Stream, tabledata::TableData, Data,
  },
  definitions::Definitions,
  desc::Description,
  error::VOTableError,
  field::Field,
  fieldref::FieldRef,
  group::{Group, TableGroup},
  impls::mem::VoidTableDataContent,
  info::Info,
  link::Link,
  param::Param,
  paramref::ParamRef,
  resource::Resource,
  table::Table,
  table::TableElem,
  timesys::TimeSys,
  values::{Max, Min, Opt, Values},
  votable::VOTable,
};

pub trait TableDataContent: Default + PartialEq + serde::Serialize {
  fn new() -> Self {
    Self::default()
  }

  /// When deserializing from JSON, TOML or YAML, we should implement a 'DeserializeSeed'
  /// based on the table Schema. But:
  /// * we have to implement `Deserialize` by hand on `Table`, `Data`, `DataElem`,
  /// `TableData`, `Binary`, `Binary2` and `Stream`, which is daunting task, even using
  /// `cargo expand`...
  /// * even so, the metadata may be parsed after the data
  /// (e.g. in JSON the key order is not guaranteed to be preserved)
  ///
  /// So, the result of the deserialization without knowing the table schema may result in
  /// no-homogeneous datatype in a same column.
  /// E.g `short` and `int`, or `char` and `string` may be mixed.
  ///
  /// So, we use this method to replace incorrect types by the porper ones as a post-parsing process.
  /// This is not ideal on a performance point-of-view, but Serde usage to convert from JSON, TOML
  /// and YAML **should be** limited to small tables (less than a few hundreds of megabytes).
  fn ensures_consistency(&mut self, context: &[TableElem]) -> Result<(), String>;

  /// Called when Event::Start("DATATABLE") as been detected and **MUST**
  /// return after event Event::End("DATATABLE")
  fn read_datatable_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError>;

  /// Called when Event::Start("STREAM") as been detected (in BINARY) and **MUST**
  /// return after event Event::End("STREAM")
  fn read_binary_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError>;

  /// Called when Event::Start("STREAM") as been detected (in BINARY2) and **MUST**
  /// return after event Event::End("STREAM")
  fn read_binary2_content<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    context: &[TableElem],
  ) -> Result<(), VOTableError>;

  fn write_in_datatable<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError>;

  fn write_in_binary<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError>;

  fn write_in_binary2<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &[TableElem],
  ) -> Result<(), VOTableError>;
}

trait QuickXmlReadWrite: Sized {
  const TAG: &'static str;
  const TAG_BYTES: &'static [u8] = Self::TAG.as_bytes();
  type Context;

  fn tag(&self) -> &str {
    Self::TAG
  }

  fn tag_bytes(&self) -> &[u8] {
    Self::TAG_BYTES
  }

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
  /// when finding the `End` event matching the last `Start` event before entering the method.
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
  /// when finding the `End` event matching the last `Start` event before entering the method.
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

// We visit all sub elements, bu we retrieve attributes from objects
// We kinf of added a part of the context by prefixing some visit methods with the name of the
// TAG it is called from.
pub trait VOTableVisitor<C: TableDataContent> {
  type E: Error;

  #[cfg(feature = "mivot")]
  type M: VodmlVisitor<E = Self::E>;

  fn visit_votable_start(&mut self, votable: &mut VOTable<C>) -> Result<(), Self::E>;
  fn visit_votable_ended(&mut self, votable: &mut VOTable<C>) -> Result<(), Self::E>;

  fn visit_description(&mut self, description: &mut Description) -> Result<(), Self::E>; // No start/end
  fn visit_coosys_start(&mut self, coosys: &mut CooSys) -> Result<(), Self::E>;
  fn visit_coosys_ended(&mut self, coosys: &mut CooSys) -> Result<(), Self::E>;
  fn visit_timesys(&mut self, timesys: &mut TimeSys) -> Result<(), Self::E>; // No start/end
  fn visit_group_start(&mut self, group: &mut Group) -> Result<(), Self::E>;
  fn visit_group_ended(&mut self, group: &mut Group) -> Result<(), Self::E>;

  #[cfg(feature = "mivot")]
  fn get_mivot_visitor(&mut self) -> Self::M;

  fn visit_table_group_start(&mut self, group: &mut TableGroup) -> Result<(), Self::E>;
  fn visit_table_group_ended(&mut self, group: &mut TableGroup) -> Result<(), Self::E>;

  fn visit_paramref(&mut self, paramref: &mut ParamRef) -> Result<(), Self::E>; // No start/end
  fn visit_fieldref(&mut self, fieldref: &mut FieldRef) -> Result<(), Self::E>; // No start/end

  fn visit_param_start(&mut self, param: &mut Param) -> Result<(), Self::E>;
  fn visit_param_ended(&mut self, param: &mut Param) -> Result<(), Self::E>;

  fn visit_field_start(&mut self, field: &mut Field) -> Result<(), Self::E>;
  fn visit_field_ended(&mut self, field: &mut Field) -> Result<(), Self::E>;

  fn visit_info(&mut self, info: &mut Info) -> Result<(), Self::E>; // No start/end
  fn visit_definitions_start(&mut self, coosys: &mut Definitions) -> Result<(), Self::E>;
  fn visit_definitions_ended(&mut self, coosys: &mut Definitions) -> Result<(), Self::E>;

  fn visit_resource_start(&mut self, resource: &mut Resource<C>) -> Result<(), Self::E>;
  fn visit_resource_ended(&mut self, resource: &mut Resource<C>) -> Result<(), Self::E>;

  fn visit_post_info(&mut self, info: &mut Info) -> Result<(), Self::E>;

  /// Resource sub-elems are purely virtual elements.
  fn visit_resource_sub_elem_start(&mut self) -> Result<(), Self::E>;
  fn visit_resource_sub_elem_ended(&mut self) -> Result<(), Self::E>;

  fn visit_link(&mut self, link: &mut Link) -> Result<(), Self::E>; // No start/end

  fn visit_table_start(&mut self, table: &mut Table<C>) -> Result<(), Self::E>;
  fn visit_table_ended(&mut self, table: &mut Table<C>) -> Result<(), Self::E>;

  fn visit_data_start(&mut self, data: &mut Data<C>) -> Result<(), Self::E>;
  fn visit_data_ended(&mut self, data: &mut Data<C>) -> Result<(), Self::E>;

  fn visit_tabledata(&mut self, table: &mut TableData<C>) -> Result<(), Self::E>;
  fn visit_binary_stream(&mut self, stream: &mut Stream<C>) -> Result<(), Self::E>;
  fn visit_binary2_stream(&mut self, stream: &mut Stream<C>) -> Result<(), Self::E>;
  fn visit_fits_start(&mut self, fits: &mut Fits) -> Result<(), Self::E>;
  fn visit_fits_stream(&mut self, stream: &mut Stream<VoidTableDataContent>)
    -> Result<(), Self::E>;
  fn visit_fits_ended(&mut self, fits: &mut Fits) -> Result<(), Self::E>;

  fn visit_values_start(&mut self, values: &mut Values) -> Result<(), Self::E>;
  fn visit_values_min(&mut self, min: &mut Min) -> Result<(), Self::E>; // No start/end
  fn visit_values_max(&mut self, max: &mut Max) -> Result<(), Self::E>; // No start/end
  fn visit_values_opt_start(&mut self, opt: &mut Opt) -> Result<(), Self::E>;
  fn visit_values_opt_ended(&mut self, opt: &mut Opt) -> Result<(), Self::E>;
  fn visit_values_ended(&mut self, values: &mut Values) -> Result<(), Self::E>;
}

// For Javascript, see https://rustwasm.github.io/wasm-bindgen/reference/arbitrary-data-with-serde.html

#[cfg(test)]
mod tests {
  use std::{i64, str::from_utf8};

  use quick_xml::{events::Event, Reader, Writer};
  use serde_json::{Number, Value};

  use super::{
    coosys::{CooSys, System},
    data::Data,
    datatype::Datatype,
    field::{Field, Precision},
    impls::mem::InMemTableDataRows,
    impls::VOTableValue,
    info::Info,
    link::Link,
    resource::Resource,
    table::Table,
    values::Values,
    votable::{VOTable, Version},
    QuickXmlReadWrite,
  };

  #[test]
  fn test_create_in_mem_1() {
    let rows = vec![
      vec![
        VOTableValue::Null, //VOTableValue::Double(f64::NAN),
        VOTableValue::CharASCII('*'),
        VOTableValue::Long(i64::max_value()),
      ],
      vec![
        VOTableValue::Double(0.4581e+38),
        VOTableValue::Null,
        VOTableValue::Long(i64::min_value()),
      ],
      vec![
        VOTableValue::Null,
        VOTableValue::CharASCII('*'),
        VOTableValue::Long(0),
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
      .set_description(
        r#"Photometric and spectroscopic catalog of objects in the field around HE0226-4110"#
          .into(),
      )
      .push_coosys(CooSys::new("J2000", System::new_default_eq_fk5()))
      .push_coosys(CooSys::new("J2015.5", System::new_icrs().set_epoch(2015.5)))
      .insert_extra(
        "toto",
        Number::from_f64(0.5)
          .map(Value::Number)
          .unwrap_or(Value::Null),
      )
      .push_sub_elem(
        ResourceSubElem::from_table(table)
          .push_info(Info::new("matches", "50").set_content("matching records"))
          .push_info(Info::new("Warning", "No center provided++++"))
          .push_info(Info::new("Warning", "truncated result (maxtup=50)"))
          .push_info(
            Info::new("QUERY_STATUS", "OVERFLOW").set_content("truncated result (maxtup=50)"),
          ),
      );

    let mut votable = VOTable::new(Version::V1_4, resource)
              .set_id("my_votable")
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
        println!("{}", &content);
        let mut votable2 =
          serde_json::de::from_str::<VOTable<InMemTableDataRows>>(content.as_str()).unwrap();
        votable2.ensures_consistency().unwrap();
        let content2 = serde_json::to_string_pretty(&votable2).unwrap();
        assert_eq!(content, content2);
        // To solve this, we have to implement either:
        // * a Deserialiser with Seed on Table::data from the table schema
        // * a post processing replacing the FieldValue by the righ objects (given the table schema)
        assert_eq!(votable, votable2);
      }
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }

    println!("\n\n#### YAML ####\n");

    match serde_yaml::to_string(&votable) {
      Ok(content) => {
        println!("{}", &content);
        let votable2 =
          serde_yaml::from_str::<VOTable<InMemTableDataRows>>(content.as_str()).unwrap();
        let content2 = serde_yaml::to_string(&votable2).unwrap();
        assert_eq!(content, content2);
      }
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }

    println!("\n\n#### VOTABLE ####\n");

    let mut content = Vec::new();
    let mut write = Writer::new_with_indent(/*stdout()*/ &mut content, b' ', 4);
    match votable.write(&mut write, &()) {
      Ok(_) => {
        println!("{}", from_utf8(content.as_slice()).unwrap());

        let mut votable2 = VOTable::<InMemTableDataRows>::from_reader(content.as_slice()).unwrap();
        let mut content2 = Vec::new();
        let mut write2 = Writer::new_with_indent(&mut content2, b' ', 4);
        votable2.write(&mut write2, &()).unwrap();

        //eprintln!("CONTENT1:\n{}", from_utf8(content.as_slice()).unwrap());
        //eprintln!("CONTENT2:\n{}", from_utf8(content2.as_slice()).unwrap());

        assert_eq!(content, content2);
      }
      Err(error) => {
        println!("Error: {:?}", &error);
        assert!(false);
      }
    }

    println!("\n\n#### TOML ####\n");

    match toml::ser::to_string_pretty(&votable) {
      Ok(content) => {
        println!("{}", &content);
        let votable2 = toml::de::from_str::<VOTable<InMemTableDataRows>>(content.as_str()).unwrap();
        let content2 = toml::ser::to_string_pretty(&votable2).unwrap();
        assert_eq!(content, content2);
      }
      Err(error) => {
        println!("{:?}", &error);
        assert!(false);
      }
    }

    /*println!("\n\n#### XML ####\n");

    match quick_xml::se::to_string(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => println!("{:?}", &error),
    }*/

    // AVRO ?
  }

  // not a test, used for the README.md
  #[test]
  fn test_create_in_mem_simple() {
    let rows = vec![
      vec![
        VOTableValue::Double(f64::NAN),
        VOTableValue::CharASCII('*'),
        VOTableValue::Float(14.52),
      ],
      vec![
        VOTableValue::Double(1.25),
        VOTableValue::Null,
        VOTableValue::Float(-1.2),
      ],
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
          Field::new("m_SDSS12", Datatype::CharASCII)
            .set_ucd("meta.code.multip")
            .set_arraysize(ArraySize::new_fixed_1d(1))
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
      .set_description(
        r#"Photometric and spectroscopic catalog of objects in the field around HE0226-4110"#
          .into(),
      )
      .push_coosys(CooSys::new("J2000", System::new_default_eq_fk5()))
      .push_coosys(CooSys::new("J2015.5", System::new_icrs().set_epoch(2015.5)))
      .push_sub_elem(ResourceSubElem::from_table(table).push_info(
        Info::new("QUERY_STATUS", "OVERFLOW").set_content("truncated result (maxtup=2)"),
      ));

    let votable = VOTable::new(Version::V1_4, resource)
      .set_id("my_votable")
      .set_description(r#"VizieR Astronomical Server vizier.u-strasbg.fr"#.into())
      .push_info(Info::new("votable-version", "1.99+ (14-Oct-2013)").set_id("VERSION"))
      .wrap();

    println!("\n\n#### JSON ####\n");

    match serde_json::to_string_pretty(&votable) {
      Ok(content) => {
        println!("{}", &content);
      }
      Err(error) => println!("{:?}", &error),
    }

    println!("\n\n#### YAML ####\n");

    match serde_yaml::to_string(&votable) {
      Ok(content) => {
        println!("{}", &content);
      }
      Err(error) => println!("{:?}", &error),
    }

    println!("\n\n#### TOML ####\n");

    match toml::ser::to_string_pretty(&votable) {
      Ok(content) => {
        println!("{}", &content);
      }
      Err(error) => println!("{:?}", &error),
    }

    println!("\n\n#### VOTABLE ####\n");
    let mut write = Writer::new_with_indent(std::io::stdout(), b' ', 4);
    match votable.unwrap().write(&mut write, &()) {
      Ok(_) => println!("\nOK"),
      Err(error) => println!("Error: {:?}", &error),
    }

    /*println!("\n\n#### XML ####\n");
    match quick_xml::se::to_string(&votable) {
      Ok(content) => println!("{}", &content),
      Err(error) => println!("{:?}", &error),
    }*/
    // AVRO ?
  }

  use crate::field::ArraySize;
  use crate::resource::ResourceSubElem;
  use std::io::Cursor;

  pub(crate) fn test_read<X: QuickXmlReadWrite<Context = ()>>(xml: &str) -> X {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == X::TAG_BYTES => {
          let mut info = X::from_attributes(e.attributes()).unwrap();
          let res = info.read_sub_elements_and_clean(reader, &mut buff, &());
          if let Err(e) = res {
            eprintln!("Error: {}", e.to_string());
            assert!(false);
          }
          return info;
        }
        Event::Empty(ref mut e) if e.local_name() == X::TAG_BYTES => {
          let info = X::from_attributes(e.attributes()).unwrap();
          return info;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        Event::Comment(_) => (),
        Event::DocType(_) => (),
        Event::Decl(_) => (),
        _ => {
          println!("{:?}", event);
          assert!(false);
        }
      }
    }
  }

  pub(crate) fn test_writer<X: QuickXmlReadWrite<Context = ()>>(mut writable: X, xml: &str) {
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writable.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }
}
