<meta charset="utf-8"/>

# `votable` or `VOTLibRust`

Library to build, read and write [VOTables](https://www.ivoa.net/documents/VOTable/)
in Rust and to convert them efficiently back and forth in JSON, YAML, TOML, XML-TABLEDATA, XML-BINARY and XML-BINARY2
while preserving all elements (except comments) and their order.

[![](https://img.shields.io/crates/v/votable.svg)](https://crates.io/crates/votable)
[![](https://img.shields.io/crates/d/votable.svg)](https://crates.io/crates/votable)
[![API Documentation on docs.rs](https://docs.rs/votable/badge.svg)](https://docs.rs/votable/)
[![BenchLib](https://github.com/cds-astro/cds-votable-rust/actions/workflows/libtest.yml/badge.svg)](https://github.com/cds-astro/cds-moc-rust/actions/workflows/libtest.yml)

VOT Lib Rust is used in:
* [VOTCli](https://github.com/cds-astro/cds-votable-rust/tree/main/crates/cli) 
  convert VOTables from the command line;
* [VOTWasm](https://github.com/cds-astro/cds-votable-rust/tree/main/crates/wasm) 
  read/write and convert VOTables in Web Browsers.
* [Aladin Lite V3](https://github.com/cds-astro/aladin-lite)

## Status

The code contains all main functionalities: 
* supports BINARY, BINARY2 formats
* supports CDATA in rows, StAX mode for streaming
* supports MIVOT block parsing

But it is far from been as clean and documented as I would like.
If you want to use this crate in a Rust project, you should probably have a look 
at the VOTCli code.

We are still (reasonably) open to changes, e.g.:
* we could flag attributes with a '@' prefix
* we could use upper case elements tag names
* we could remove the 's' suffix in elements arrays
* we could change the `post_infos` name for something else
* ...

More testing is required, especially the bit type and arrays.
Please, provide us with your VOTable examples!


## Why JSON, TOML, YAML in addition to XML

VOTable is an XML based format. Why other formats?
* JSON: to easily manipulate VOTable data in Web Browsers since JSON 
  represent JavaScript objects (and all browsers parse JSON into 
  JavaScript objects natively).
* TOML: to easily update manually VOTables (especially the metadata part of 
  VOTables). Moreover, it is quite compact.
* YAML: because some people like it, and it was almost free to implement 
  (thanks to serde).

## Motivations

* Support natively the VOTable format in the new CDS internal tool `qat2s`
  (tool to query and manipulate possibly large catalogues with multi-thread capabilities).
* Store VizieR (large) catalogues rich metadata in a user friendly format (TOML) while
  being able to return the same VOTable header as VizieR (without using a database connexion).
    + for `qat2s`, `ExXmatch`, `progressive catalogue`
* Add a Rust VOTable parsing and writing library for 
  [Aladin Lite V3](https://github.com/cds-astro/aladin-lite/tree/develop)
* Ensure statically the in-memory built structure is compliant with a VOTable 
* ...

## Design choices and problems

Although the default provided implementation converting from JSON/YAML/TOML **does not
focus on performances** since we do not use the VOTable FIELDs information but 
deserialize each table field in the first succeeding *VOTableValue* 
(see `votable::impls::Schema.serialize_seed`), performances seems to be good nevertheless.

VOT Lib resort heavily on [serde](https://serde.rs/).

This library has been design to preserve the order of VOTable TAGs when 
converting back on forth in XML, JSON, ...  
But, so far XML comments are ignored and lost.

In JSON/TOML/YAML, for the VOTABLE and RESOURCE elements, we make a difference
between INFO blocks located before and after the RESOURCE element(s).
We use *infos* (only for RESOURCE) and *post_infos* arrays.
Quoting the IVOA document: 
> The INFO element may occur before the closing tags /TABLE and /RESOURCE and 
> /VOTABLE (enables post-operational diagnostics)
> 
(we wonder if post-operational diagnostics should not have a name different 
from INFO in VOTables).


In JSON/TOML/YAML, for VOTABLE, RESOURCE, TABLE and GROUP elements, we group
together the *"open bullet"* (see 7.1 of the [VOTable standard](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html))
elements in an *elements* array containing objects having an *"elem\_type"* 
attribute set to one of: *Info*, *Field*, *Coosys*, *Timesys*, *Group*, *Param*, ... 

Internally we make a difference (different struct/class) between GROUP
in VOTABLE and RESOURCE from GROUP in TABLE since in the later case the
GROUP may contain FIELDRef.

In a *RESOURCE*, we pack together *LINK*s, a sub-*RESOURCE* or a *TABLE* and *INFO*
together in a *ResourceSubElem* structure.

In JSON/TOML/YAML, there is no difference between attribute and sub-elements 
names (all in camel case).

### WARNINGS

* TOML does not support `null` (we so far convert `null` values by an empty string).
* Conversions from/to TOML/JSON/YAML requires all data to be loaded in memory, it is not adapted for large files.

## Other way to convert from VOTable to JSON

The XML2JSON conversion has been exercised
by [Laurent Michel](https://github.com/lmichel)
in the context of the processing of model annotations in VOTables 
([the MIVOT](https://github.com/ivoa-std/ModelInstanceInVot)).
The use case is to convert model instances, serialized in XML, into JSON messages.  
The conversion is using standard Python tools ([xmltodic](https://pypi.org/project/xmltodict/) module). 
The code below is extracted from the [client code](https://github.com/ivoa/modelinstanceinvot-code) project. 
It is to be noted that the translation rules are not PYTHON (nor VOTable) specific, 
they are also implemented in e.g. [XSLT](https://github.com/bramstein/xsltjson).
```python
import os
import xmltodict
import json
import numpy
from lxml import etree

class MyEncoder(json.JSONEncoder):

  def default(self, obj):
    if isinstance(obj, numpy.integer):
      return int(obj)
    elif isinstance(obj, numpy.floating):
      return float(obj)
    elif isinstance(obj, numpy.ndarray):
      return obj.tolist()
    else:
      return super(MyEncoder, self).default(obj)

data_path = os.path.dirname(os.path.realpath(__file__))

xml_block = etree.parse(os.path.join(data_path, "votable_to_json.xml"))
raw_json = xmltodict.parse(etree.tostring(xml_block))
pretty_json = json.dumps(raw_json, indent=2, cls=MyEncoder)
print(pretty_json)

with open(os.path.join(data_path, "votable_to_json.json"), 'w') as file:
  file.write(json.dumps(raw_json, indent=2))
```


Advantages:
* standard
* few lines of python

Inconvenient:
* the order of elements (especially INFOs and post processing INFOs) is lost
* it is a one way conversion (not possible to then convert from JSON to VOTable)


## Example

Several outputs obtained from the same API made VOTable.

### Rust code (API created VOTable)

```rust
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
      .set_width(6)
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
  .push_sub_elem(
    ResourceSubElem::from_table(table)
      .push_info(Info::new("QUERY_STATUS", "OVERFLOW").set_content("truncated result (maxtup=2)"))
  )  
);

let mut votable = VOTable::new(resource)
  .set_id("my_votable")
  .set_version(Version::V1_4)
  .set_description(r#"VizieR Astronomical Server vizier.u-strasbg.fr"#.into())
  .push_info(Info::new("votable-version", "1.99+ (14-Oct-2013)").set_id("VERSION"));
```

Remark: the coherence between user input `VOTableValue` and declared `Fields` is checked
only when serializing in `BINARY` or `BINARY2`.

### VOTable

```xml
<?xml version="1.0" encoding="UTF-8"?>
<VOTABLE ID="my_votable" version="1.4">
  <DESCRIPTION>VizieR Astronomical Server vizier.u-strasbg.fr</DESCRIPTION>
  <INFO ID="VERSION" name="votable-version" value="1.99+ (14-Oct-2013)"/>
  <RESOURCE ID="yCat_17011219" name="J/ApJ/701/1219">
    <DESCRIPTION>Photometric and spectroscopic catalog of objects in the field around HE0226-4110</DESCRIPTION>
    <COOSYS ID="J2000" system="eq_FK4" equinox="B2000"/>
    <COOSYS ID="J2015.5" system="ICRS" epoch="J2015.5"/>
    <TABLE ID="V_147_sdss12" name="V/147/sdss12">
      <DESCRIPTION>SDSS photometric catalog</DESCRIPTION>
      <FIELD name="RA_ICRS" datatype="double" unit="deg" precision="6" width="10" ucd="pos.eq.ra;meta.main">
        <DESCRIPTION>Right Ascension of the object (ICRS) (ra)</DESCRIPTION>
      </FIELD>
      <FIELD name="m_SDSS12" datatype="char" precision="6" width="10" ucd="meta.code.multip" arraysize="1">
        <DESCRIPTION>[*] Multiple SDSS12 name</DESCRIPTION>
        <LINK href="http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-info=XML&amp;-out.add=.&amp;-source=V/147&amp;SDSS12=${SDSS12}"/>
      </FIELD>
      <FIELD name="umag" datatype="float" unit="mag" precision="3" width="2" ucd="phot.mag;em.opt.U">
        <DESCRIPTION>[4/38]? Model magnitude in u filter, AB scale (u) (5)</DESCRIPTION>
        <VALUES null="NaN"/>
      </FIELD>
      <DATA>
        <TABLEDATA>
          <TR>
            <TD>NaN</TD>
            <TD>*</TD>
            <TD>14.52</TD>
          </TR>
          <TR>
            <TD>1.25</TD>
            <TD></TD>
            <TD>-1.2</TD>
          </TR>
        </TABLEDATA>
      </DATA>
    </TABLE>
    <INFO name="QUERY_STATUS" value="OVERFLOW">truncated result (maxtup=2)</INFO>
  </RESOURCE>
</VOTABLE>
```


### JSON

```json
{
  "votable": {
    "ID": "my_votable",
    "version": "1.4",
    "description": "VizieR Astronomical Server vizier.u-strasbg.fr",
    "elems": [
      {
        "elem_type": "Info",
        "ID": "VERSION",
        "name": "votable-version",
        "value": "1.99+ (14-Oct-2013)"
      }
    ],
    "resources": [
      {
        "ID": "yCat_17011219",
        "name": "J/ApJ/701/1219",
        "description": "Photometric and spectroscopic catalog of objects in the field around HE0226-4110",
        "elems": [
          {
            "elem_type": "CooSys",
            "ID": "J2000",
            "system": "eq_FK4",
            "equinox": 2000.0
          },
          {
            "elem_type": "CooSys",
            "ID": "J2015.5",
            "system": "ICRS",
            "epoch": 2015.5
          }
        ],
        "sub_elems": [
          {
            "resource_or_table": {
              "elem_type": "Table",
              "id": "V_147_sdss12",
              "name": "V/147/sdss12",
              "description": "SDSS photometric catalog",
              "elems": [
                {
                  "elem_type": "Field",
                  "name": "RA_ICRS",
                  "datatype": "double",
                  "unit": "deg",
                  "precision": "6",
                  "width": 10,
                  "ucd": "pos.eq.ra;meta.main",
                  "description": "Right Ascension of the object (ICRS) (ra)"
                },
                {
                  "elem_type": "Field",
                  "name": "m_SDSS12",
                  "datatype": "char",
                  "precision": "6",
                  "width": 10,
                  "ucd": "meta.code.multip",
                  "arraysize": "1",
                  "description": "[*] Multiple SDSS12 name",
                  "links": [
                    {
                      "href": "http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-info=XML&-out.add=.&-source=V/147&SDSS12=${SDSS12}"
                    }
                  ]
                },
                {
                  "elem_type": "Field",
                  "name": "umag",
                  "datatype": "float",
                  "unit": "mag",
                  "precision": "3",
                  "width": 2,
                  "ucd": "phot.mag;em.opt.U",
                  "description": "[4/38]? Model magnitude in u filter, AB scale (u) (5)",
                  "values": {
                    "null": "NaN"
                  }
                }
              ],
              "data": {
                "data_type": "TableData",
                "rows": [
                  [
                    null,
                    "*",
                    14.52
                  ],
                  [
                    1.25,
                    null,
                    -1.2
                  ]
                ]
              }
            },
            "infos": [
              {
                "name": "QUERY_STATUS",
                "value": "OVERFLOW",
                "content": "truncated result (maxtup=2)"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

### TOML

```toml
[votable]
ID = 'my_votable'
version = '1.4'
description = 'VizieR Astronomical Server vizier.u-strasbg.fr'

[[votable.elems]]
elem_type = 'Info'
ID = 'VERSION'
name = 'votable-version'
value = '1.99+ (14-Oct-2013)'

[[votable.resources]]
ID = 'yCat_17011219'
name = 'J/ApJ/701/1219'
description = 'Photometric and spectroscopic catalog of objects in the field around HE0226-4110'

[[votable.resources.elems]]
elem_type = 'CooSys'
ID = 'J2000'
system = 'eq_FK4'
equinox = 2000.0

[[votable.resources.elems]]
elem_type = 'CooSys'
ID = 'J2015.5'
system = 'ICRS'
epoch = 2015.5

[[votable.resources.sub_elems]]
[votable.resources.sub_elems.resource_or_table]
elem_type = 'Table'
id = 'V_147_sdss12'
name = 'V/147/sdss12'
description = 'SDSS photometric catalog'

[[votable.resources.sub_elems.resource_or_table.elems]]
elem_type = 'Field'
name = 'RA_ICRS'
datatype = 'double'
unit = 'deg'
precision = '6'
width = 10
ucd = 'pos.eq.ra;meta.main'
description = 'Right Ascension of the object (ICRS) (ra)'

[[votable.resources.sub_elems.resource_or_table.elems]]
elem_type = 'Field'
name = 'm_SDSS12'
datatype = 'char'
precision = '6'
width = 10
ucd = 'meta.code.multip'
arraysize = '1'
description = '[*] Multiple SDSS12 name'

[[votable.resources.sub_elems.resource_or_table.elems.links]]
href = 'http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-info=XML&-out.add=.&-source=V/147&SDSS12=${SDSS12}'

[[votable.resources.sub_elems.resource_or_table.elems]]
elem_type = 'Field'
name = 'umag'
datatype = 'float'
unit = 'mag'
precision = '3'
width = 2
ucd = 'phot.mag;em.opt.U'
description = '[4/38]? Model magnitude in u filter, AB scale (u) (5)'

[votable.resources.sub_elems.resource_or_table.elems.values]
null = 'NaN'

[votable.resources.sub_elems.resource_or_table.data]
data_type = 'TableData'
rows = [
    [
    nan,
    '*',
    14.52,
],
    [
    1.25,
    '',
    -1.2,
],
]

[[votable.resources.sub_elems.infos]]
name = 'QUERY_STATUS'
value = 'OVERFLOW'
content = 'truncated result (maxtup=2)'
```

### YAML

```yaml
votable:
  ID: my_votable
  version: '1.4'
  description: VizieR Astronomical Server vizier.u-strasbg.fr
  elems:
    - elem_type: Info
      ID: VERSION
      name: votable-version
      value: 1.99+ (14-Oct-2013)
  resources:
    - ID: yCat_17011219
      name: J/ApJ/701/1219
      description: Photometric and spectroscopic catalog of objects in the field around HE0226-4110
      elems:
        - elem_type: CooSys
          ID: J2000
          system: eq_FK4
          equinox: 2000.0
        - elem_type: CooSys
          ID: J2015.5
          system: ICRS
          epoch: 2015.5
      sub_elems:
        - resource_or_table:
            elem_type: Table
            id: V_147_sdss12
            name: V/147/sdss12
            description: SDSS photometric catalog
            elems:
              - elem_type: Field
                name: RA_ICRS
                datatype: double
                unit: deg
                precision: '6'
                width: 10
                ucd: pos.eq.ra;meta.main
                description: Right Ascension of the object (ICRS) (ra)
              - elem_type: Field
                name: m_SDSS12
                datatype: char
                precision: '6'
                width: 10
                ucd: meta.code.multip
                arraysize: '1'
                description: '[*] Multiple SDSS12 name'
                links:
                  - href: http://vizier.u-strasbg.fr/viz-bin/VizieR-4?-info=XML&-out.add=.&-source=V/147&SDSS12=${SDSS12}
              - elem_type: Field
                name: umag
                datatype: float
                unit: mag
                precision: '3'
                width: 2
                ucd: phot.mag;em.opt.U
                description: '[4/38]? Model magnitude in u filter, AB scale (u) (5)'
                values:
                  'null': NaN
            data:
              data_type: TableData
              rows:
                - - .nan
                  - '*'
                  - 14.52
                - - 1.25
                  - null
                  - -1.2
          infos:
            - name: QUERY_STATUS
              value: OVERFLOW
              content: truncated result (maxtup=2)
```

## Example: Iterate on both Tables and Rows of a VOTable

```rust
    let mut votable_it = VOTableIterator::from_file("resources/sdss12.vot")?;
    while let Some(mut row_it) = votable_it.next_table_row_value_iter()? {
      let table_ref_mut = row_it.table();
      println!("Fields: {:?}", table_ref_mut.elems);
      for (i, row) in row_it.enumerate() {
        println!("Row {}: {:?}", i, row);
      }
    }
    let votable = votable_it.end_of_it();
    println!("VOTable: {:?}", votable);
```


## To-do list

* [X] Support `CDATA`
    + [X] Support in `Info`, `Description`, `Link`, `PARAMRef` and `FIELDRef`
    + [X] Support `CDATA` in `<TD></TD>` 
* [ ] Fill the doc for the Rust library (but I so far do not know people interested in such a lib since Rust is not very used in the astronomy community so far, so...)
* [X] Add a check method ensuring that user input VOTableValue (using the API to build a VOTable) 
      matches the table schema (or automatically converting in the right VOTableValue)
* [ ] Add much more tests!
* [X] Add possibility to convert to/from `TABLEDATA`, `BINARY`, `BINARY2`
    + but still to implement streaming mode in CLI
* [X] Implements `toCSV` (but not `fromCSV`) in CLI
* [ ] Replace `quick_error` by `anyhow` and `thiserror`.
* [ ] Bump `quick_xml` version and: 
    + [ ] Implement parsing without copy from `&[u8]` (when full data in men, e.g. in wasm, or for memmapped files) 
    + [ ] Implement async
* ...


## License

Like most projects in Rust, this project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.




