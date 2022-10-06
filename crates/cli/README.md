<meta charset="utf-8"/>

# `votable-cli` or `VOTCli`

A command-line to read and convert [VOTables](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html)
from/to XML, JSON, TOML and YAML.

```bash
> time vot xml xml -i VII.vot > xml.1.vot
real	0m0,009s
user	0m0,001s
sys 0m0,009s

> time vot xml toml --pretty -:i VII.vot | vot toml json | vot json xml > xml.2.vot
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



