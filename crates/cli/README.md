<meta charset="utf-8"/>

# `votable-cli` or `VOTCli`

Command-line to extract information from IVOA VOTables](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html)
and to convert efficiently VOTables back and forth in XML, JSON, YAML, TOML (and to CSV) while preserving all elements (except in CSV).

## Status

The [library](https://github.com/cds-astro/cds-votable-rust) this CLI 
is based on is in an early stage of development.
We are (reasonably) open to changes in the various format, e.g.:
* we could flag attributes with a '@' prefix
* we could use upper case elements tag names
* we could remove the 's' suffix in elements arrays
* we could change the `pos_infos` name for something else
* ...

More testing is required, especially the bit type and arrays.
Please, provide us with VOTable examples!

## To-Do list

* [X] Support `CDATA` in `TD` tags
* [X] Use the iterator to implement streaming transformations between DATATABLE/BINARY/BINARY2.
* [X] Also implement streaming conversion to CSV.
* [ ] Implement streaming mode for multiple tables (if it is really useful, please tell me).
* [ ] Add commands to modify a VOTable metadata.
* [ ] Add commands to select/compute columns and filter rows?
 
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
Command-line to extract information from IVOA VOTables and to convert them in XML, JSON, YAML, TOML and CSV.

Usage: vot <COMMAND>

Commands:
  convert   Convert a VOTable from one format to another (full table loaded in memory)
  sconvert  Convert a (large) single table VOTable in streaming mode. All information is preserved, even after the `</TABLE>` tag. 
            The `TABLEDATA` to `CSV` conversion preserve fields formatting.
  update    Update metadata from another VOTable of from a CSV file
  get       Get information from a VOTable (sstreaming for XML files)
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
Convert a (large) single table VOTable in streaming mode. All information is preserved, even after the `</TABLE>` tag. 
The `TABLEDATA` to `CSV` conversion preserve fields formatting.

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
Get information from a VOTable (sstreaming for XML files)

Usage: vot get [OPTIONS] <COMMAND>

Commands:
  struct        Print the VOTable structure
  colnames      Print column names, one separated values line per table.
  fields-array  Print selected field information as an array
  help          Print this message or the help of the given subcommand(s)

Options:
  -i, --in <FILE>           Path of the input VOTable [default: read from stdin]
  -t, --in-fmt <INPUT_FMT>  Format of the input VOTable ('xml', 'json', 'yaml' or 'toml') [default: guess from file extension]
  -s, --early-stop          Stop parsing before reading first data ('xml' input only)
  -h, --help                Print help
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


## Log messages

You can adapt the `level` of log messages using
the environement variable `RUST_LOG` with one of the following value:
`error`, `warn`, `info`, `debug`, `trace` and `off`.

E.g.:
```bash
RUST_LOG="trace" vot get --in my_votable.xml struct
```

See [env_logger](https://docs.rs/env_logger/latest/env_logger/) for more details.


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



