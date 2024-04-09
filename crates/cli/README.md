<meta charset="utf-8"/>

# `votable-cli` or `VOTCli`

Command-line to extract/edit information from [IVOA VOTables](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html)
and to convert efficiently VOTables back and forth in XML, JSON, YAML, TOML (and to CSV) while preserving all elements (except in CSV).

## Status

The CLI is in active development.

More testing is required, especially the bit type and arrays.
Please, provide us with VOTable examples and/or usecases!

 
## Install

### From pypi for python users

VOTable cli is available in [pypi](https://pypi.org/project/votable-cli/),
you can thus install the `vot` executable using `pip`:
```bash
pip install votable-cli
vot --help
```

### Debian package

Download the last `votable-cli_vxx_yyy.deb` corresponding to your architecture
(`x86_64_musl` has the most chances to fit your needs)
from the [github release page](https://github.com/cds-astro/cds-votable-rust/releases).

Install the `.deb` by clicking on it or using the command line:
```bash
sudo dpkg -i votable-cli_vxx_yyy.deb
sudo apt-get install -f
```

Then you can use the tool:
```bash
vot
man vot
```

You can uninstall using, e.g.:
```bash
sudo dpkg -r $(dpkg -f votable-cli_vxx_yyy.deb Package)
```

### Pre-compile binaries for MacOS, Linux and Windows

Download the last `vot-vxx_yyy.tar.gz` corresponding to your architecture
from the [github release page](https://github.com/cds-astro/cds-votable-rust/releases).
You probably want ot use:
* Linux: `vot-vxx-x86_64-unknown-linux-musl.tar.gz`
* MacOS: `vot-vxx-x86_64-apple-darwin.tar.gz`
* Windows: `vot-vxx-.zip`

WARNING: for linux, use [`musl`](https://en.wikipedia.org/wiki/Musl) instead of `gnu` (high chances of uncompatibility in the latter case)

The tar contains a single executable binary file.
```bash
tar xzvf vot-vxx-yyy.tar.gz
./vot
```


### Compile from source code

[Install rust](https://www.rust-lang.org/tools/install)
(and check that `~/.cargo/bin/` is in your path),
or update the Rust compiler with:
```bash
rustup update
``` 

Clone the [votable lib rust](https://github.com/cds-astro/cds-votable-rust) project:
```bash
git clone https://github.com/cds-astro/cds-votable-rust
```
Install from using `cargo`:
```bash
cargo install --path crates/cli
```


## Help message

```bash
> vot --help
Command-line to extract information from IVOA VOTables and to convert VOTable  back and forth in XML, JSON, YAML, TOML (and to CSV) while preserving all elements (except in CSV).

Usage: vot <COMMAND>

Commands:
  convert   Convert a VOTable from one format to another (full table loaded in memory)
  sconvert  Convert a single table XML VOTable in streaming mode. Tags after `</TABLE>` are preserved
  edit      Edit metadata adding/removing/updating attributes and/or elements
  get       Get information from a VOTable, like it structure or fields
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

```bash
> vot convert --help
Convert a VOTable from one format to another (full table loaded in memory)

Usage: vot convert [OPTIONS] --out-fmt <OUTPUT_FMT>

Options:
  -i, --in <FILE>             Path of the input VOTable [default: read from stdin]
  -t, --in-fmt <INPUT_FMT>    Format of the input VOTable ('xml', 'json', 'yaml' or 'toml') [default: guess from file extension]
  -o, --out <FILE>            Path of the output VOTable [default: write to stdout]
  -f, --out-fmt <OUTPUT_FMT>  Format of the output VOTable ('xml', 'xml-td', 'xml-bin', 'xml-bin2', 'json', 'yaml' or 'toml')
  -p, --pretty                Pretty print (for JSON and TOML)
  -h, --help                  Print help
```

```bash
> vot sconvert --help
Convert a single table XML VOTable in streaming mode. Tags after `</TABLE>` are preserved

Usage: vot sconvert [OPTIONS] --out-fmt <OUTPUT_FMT>

Options:
  -i, --in <FILE>                Path of the input XML VOTable [default: read from stdin]
  -o, --out <FILE>               Path of the output file [default: write to stdout]
  -f, --out-fmt <OUTPUT_FMT>     Format of the output file ('xml-td', 'xml-bin', 'xml-bin2' or 'csv')
  -s, --separator <SEPARATOR>    Separator used for the 'csv' format [default: ,]
      --parallel <N>             Exec concurrently using N threads (row order not preserved!)
      --chunk-size <CHUNK_SIZE>  Number of rows process by a same thread in `parallel` mode [default: 10000]
  -h, --help                     Print help
```

```bash
> vot get --help
Get information from a VOTable, like it structure or fields

Usage: vot get [OPTIONS] <COMMAND>

Commands:
  struct        Print the VOTable structure (useful to get Virtual IDs)
  colnames      Print column names, one separated values line per table.
  fields-array  Print selected field information as an array
  help          Print this message or the help of the given subcommand(s)

Options:
  -i, --in <FILE>           Path of the input VOTable [default: read from stdin]
  -t, --in-fmt <INPUT_FMT>  Format of the input VOTable ('xml', 'json', 'yaml' or 'toml') [default: guess from file extension]
  -s, --early-stop          Stop parsing before reading first data ('xml' input only)
  -h, --help                Print help
```

```bash
> vot edit --help
Edit metadata adding/removing/updating attributes and/or elements

Usage: vot edit [OPTIONS] --out-fmt <OUTPUT_FMT>

Options:
  -i, --in <FILE>             Path of the input VOTable [default: read from stdin]
  -t, --in-fmt <INPUT_FMT>    Format of the input VOTable ('xml', 'json', 'yaml' or 'toml') [default: guess from file extension]
  -o, --out <FILE>            Path of the output VOTable [default: write to stdout]
  -f, --out-fmt <OUTPUT_FMT>  Format of the output VOTable ('xml', 'xml-td', 'xml-bin', 'xml-bin2', 'json', 'yaml' or 'toml')
  -p, --pretty                Pretty print (for JSON and TOML)
  -e, --edit <ELEMS>          List of "TAG CONDITION ACTION ARGS", e.g.:
                              -e 'INFO name=Target rm' -e 'FIELD ID=RA set_attrs ucd=pos.eq.ra;meta.main unit=deg'
                              CONDITIONS:
                                name=VAL  name (if any) equals a given value
                                  id=VAL  id (if any) equals a given value
                                 vid=VAL  virtual id equals a given value
                              ACTIONS ARGS:
                                rm                                                 Remove the TAG
                                set_attrs        KEY=VAL (KEY=VAL) ...               Set TAG attributes
                                set_content      CONTENT                             Set the content for `DESCRIPTION`, `INFO`, `LINK`, `PARAMRef` or `FIELDRef`
                                set_desc         DESC                                Set the `DESCRIPTION` for `VOTABLE`, `RESOURCE`, `TABLE`, `FIELD`, `PARAM` or `GROUP`
                                push_timesys     KEY=VAL (KEY=VAL) ...               Push a new `TIMESYS` in `VOTABLE` or `RESOURCE`
                                set_min          KEY=VAL (KEY=VAL) ...               Set a new `MIN` for `VALUES`.
                                set_max          KEY=VAL (KEY=VAL) ...               Set a new `MAX` for `VALUES`.
                                push_option      KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `OPTION` in `VALUES` or `OPTION`
                                set_values       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Set the new `VALUES` for `FIELD` or `PARAM`
                                push_info        KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `INFO` in `VOTABLE`, `RESOURCE` or `TABLE`.
                                push_post_info   KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new post-`INFO` in `VOTABLE`.
                                push_link        KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `LINK` in  `RESOURCE`, `TABLE`, `FIELD` or `PARAM`.
                                push_fieldref    KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `FIELDRef` in `COOSYS` or table-`GROUP`.
                                push_paramref    KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `PARAMRef` in `COOSYS` or `GROUP`.
                                push_coosys      KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `COOSYS` in `VOTABLE` or `RESOURCE`.
                                push_group       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `GROUP` in `VOTABLE` or `RESOURCE`.
                                push_tablegroup  KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `GROUP` in `TABLE`.
                                push_param       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `PARAM` in `VOTABLE`, `RESOURCE`, `TABLE` or `GROUP`.
                                push_field       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `FIELD` in `TABLE`.
                                prepend_resource KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Preprend the new `RESOURCE` in `VOTABLE` or `RESOURCE`.
                                push_resource    KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `RESOURCE` in `VOTABLE` or `RESOURCE`.
                              SUB ACTIONS:
                                Sub-actions are the same as the ACTIONS (except `rm` which is not allowed).
                                A sub-action stars with a `@`. Actually one can see `@` as an action separator with the
                                main action at the left of the first `@` and all other actions being sub-actions applying on
                                the element created by the main action (the parent element).
                                E.g, in:
                                  `push_param KEY=VAL ... @set_description ... @push_link ...`
                                Both `set_description` an `push_link` are executed on the new `PARAM` built by `push_param`.
                                For sub-actions to be executed on the last created element, you can double once the `@`:
                                  `push_param KEY=VAL ... @set_description ... @push_link ... @@set_content CONTENT`
                                Here `set_content` will be applied on the new `LINK` before pushing it in the new `PARAM`.
                                After a `@@`, all sub-commands are executed on the last created element.
                                To go up from one level in the element hierarchy, use `@<`:
                                  `push_param KEY=VAL ... @set_description ... @push_link ... @@set_content CONTENT @< @push_link ...`
                                You can use arbitrary deeply nested elements using `@@` and `@<`.
                                Those three commands do not lead to the same hierarchy:
                                  `push_group ... @push_group ... @push_group @@push_group @push_group (@<)`
                                  `push_group ... @push_group ... @push_group @@push_group @@push_group (@<@<)`
                                  `push_group ... @push_group ... @push_group @@push_group @< @push_group`
                                Remark: `@@xxx` is a short version of `@> @xxx`.
  -s, --streaming             Use streaming mode: only for large XML files with a single table, and if the input format is the same as the output format
  -h, --help                  Print help
```

## Example

### XML/JSON/TOML/YAML convertion 

```bash
# In memory conversion of a VOTable from XML-TABLEDATA to JSON
vot convert --in my_votable.xml --out my_votable.json --out-fmt json
```

### Streaming conversion XML-TD, XML-BIN, CML-BIN2 and CSV

```bash
# Streaming conversion of a VOTable from XML-TABLEDATA to XML-BINARY2
vot sconvert --in my_votable.xml --out my_votable.xml.b64  --out-fmt xml-bin
# Streaming conversion from XML to CSV, in parallel, of a single large table
vot sconvert --in my_votable.xml --out my_votable.csv --out-fmt csv --parallel 6
```

### Get metadata

```bash
# Get the structure of a VOTable with virtual identifier for each element
vot get --in my_votable.xml struct --line-width 120
# Get the structure of alarge VOTable till DATA is reached
vot get --in my_votable.xml --early-stop struct --line-width 120
# Get only the colum names of a large table, with a non-ascii separator
vot get --in my_votable.xml --early-stop colnames --separator '▮'
# Get a field metadata array with selected info
vot get -in my_votable.xml fields-array index,name,datatype,arraysize,width,precision,unit,ucd,description --separator ,
```

Example: see chosen column metadata of [this votable](resource/test_edit.td.xml)
```
> vot get --in test_edit.td.xml fields-array name,datatype,arraysize,width,precision,unit,ucd
    name    dt  a w p   unit    ucd                         
   recno   int    8             meta.record                 
  f_GCTP  char  1               meta.code                   
    GCTP float    7 2           meta.id;meta.main           
    comp  char  1               meta.code.multip            
  RA1900  char 10      "h:m:s"  pos.eq.ra;meta.main         
  DE1900  char  9      "d:m:s"  pos.eq.dec;meta.main        
u_RA1900  char  1               meta.code.error;pos.eq.ra   
    Vmag float    5 2    mag    phot.mag;em.opt.V           
  n_Vmag  char  1               meta.note                   
     B-V float    5 2    mag    phot.color;em.opt.B;em.opt.V
     U-B float    5 2    mag    phot.color;em.opt.U;em.opt.B
  r_Vmag  char  1               meta.ref;pos.frame          
      MK  char  *               src.spType                  
    r_MK  char  1               meta.ref;pos.frame          
     var  char  9               meta.id                     
      HR short    4             meta.id                     
    supp  char  1               meta.note                   
      HD  char  7               meta.id                     
      DM  char 10               meta.id                     
    name  char  *               meta.id                     
      pm float    6 3 arcsec/yr pos.pm                      
    pmPA short    3      deg    pos.posAng;pos.pm           
      pi float    7 4  arcsec   pos.parallax.trig           
    e_pi float    4 1    mas    stat.error                  
    q_pi  char  1               meta.code.qual              
    o_pi short    2             meta.number                 
  Simbad  char  *               meta.ref.url                
_RA.icrs  char 10      "h:m:s"  pos.eq.ra                   
_DE.icrs  char  9      "d:m:s"  pos.eq.dec           
```


### Edit

When editing a VOTable, you probably need Virtual IDs (vid) to select tags you want to remove or to modify.
One Virtual ID is attributed by votable-cli to each tag of the VOTable.
To know the vid attributed to each tag, you can use the `vot get ... struct` subcommand. 

In the following example, we modify [this votable](resource/test_edit.td.xml) to:
* remove the first sub-RESOURCE
* add attributes to the main RESOURCE
* add DESCRIPTION to the main RESOURCE
* add COOSYS with FIELDref to the main resource
* remove PARAMs
* add a PARAM
* rename the column `recno` into `RecordNumber` 
* remove a LINK
* add an INFO to the TABLE
* add a post-INFO to the VOTABLE


```bash
# Command used to get the vid (Virtual ID) of each tag in the VOTable
vot get --in test_edit.td.xml struct 

# Command used to rename column 'recno' into 'RecordNumber'
# (the 'streaming' option on such a small table is useless)
vot edit --in test_edit.td.xml --out-fmt xml-td --streaming \
  -e 'FIELD name=recno set_attrs name=RecordNumber'

# Command used to edit the VOTable
# (the 'streaming' option on such a small table is useless)
vot edit --in test_edit.td.xml --out-fmt xml-td --streaming \
  -e 'RESOURCE vid=DR1R1 rm' \
  -e 'RESOURCE vid=DR1 set_attrs ID=R1 name=main_resource' \
  -e 'RESOURCE vid=DR1 set_description The main resource containing my super table' \
  -e 'RESOURCE vid=DR1 push_coosys ID=t4-coosys-1 system=eq_FK4 equinox=B1900 
        @push_fieldref ref=RA1900 
	  @@set_content Ref to the RA column @<
        @push_fieldref ref=DE1900 
	  @@set_content Ref to the Declination column @<' \
  -e 'PARAM name=votable-version rm' \
  -e 'PARAM name=-ref rm' \
  -e 'PARAM name=-out.max rm' \
  -e 'PARAM name=queryParameters rm' \
  -e 'TABLE name=I/238A/picat push_param name=hpx_order datatype=unsignedByte value=8 
       @set_description HEALPix order
       @set_values 
         @@set_min value=0  inclusive=true 
	  @set_max value=29 inclusive=true
	 @<
       @push_link href=https://en.wikipedia.org/wiki/HEALPix
         @@set_content General HEALPix info on Wikipedia @<' \
  -e 'FIELD name=recno set_attrs name=RecordNumber' \
  -e 'LINK vid=DR1T1F27l1 rm' \
  -e 'TABLE name=I/238A/picat push_info name=ps value=my post-scriptum @set_content My super post-scriptum' \
  -e 'VOTABLE vid=D push_post_info name=warning value=This table has been modified by using vot-cli'
```


## Log messages

You can adapt the `level` of log messages using
the environement variable `RUST_LOG` with one of the following value:
`error`, `warn`, `info`, `debug`, `trace` and `off`.

E.g.:
```bash
RUST_LOG="trace" vot get --in my_votable.xml struct
```

See [env_logger](https://docs.rs/env_logger/latest/env_logger/) for more details.

## To-Do list

* [X] Support `CDATA` in `TD` tags
* [X] Use the iterator to implement streaming transformations between DATATABLE/BINARY/BINARY2.
* [X] Also implement streaming conversion to CSV.
* [X] Add commands to modify a VOTable metadata.
* [ ] Implement streaming mode for multiple tables (if it is really useful, please tell me).
* [ ] Add commands to select/compute columns and filter rows?


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



