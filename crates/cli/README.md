<meta charset="utf-8"/>

# `votable-cli` or `VOTCli`

A command-line to read and convert [VOTables](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html)
from/to XML, JSON, TOML and YAML.

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
Command-line to convert IVOA VOTables in XML, JSON, YAML and TOML.

Usage: vot [OPTIONS] <INPUT_FMT> <OUTPUT_FMT>

Arguments:
  <INPUT_FMT>   Format of the input document ('xml', 'json', 'yaml' or 'toml')
  <OUTPUT_FMT>  Format of the output document ('xml', 'json', 'yaml' or 'toml')

Options:
  -i, --input <FILE>   Input file (else read from stdin)
  -o, --output <FILE>  Output file (else write to stdout)
  -p, --pretty         Pretty print (for JSON and TOML)
  -h, --help           Print help information
  -V, --version        Print version information

```


## Example

```bash
> time vot xml xml -i VII.vot > xml.1.vot
real	0m0,009s
user	0m0,001s
sys 0m0,009s

> time vot xml toml --pretty -i VII.vot | vot toml json | vot json xml > xml.2.vot
real	0m0,022s
user	0m0,018s
sys	0m0,012s

> diff xml.1.vot xml.2.vot
```


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



