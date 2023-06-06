<meta charset="utf-8"/>

# `votable-wasm` or `VOTWasm`

WebAssembly Library to convert IVOA [VOTables](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html)
in Json Objects, XML, JSON, YAML and TOML.

## Status

The [library](https://github.com/cds-astro/cds-votable-rust) this WASM module
is based on is in an early stage of development.
We are (reasonably) open to changes in the various format, e.g.:
* we could flag attributes with a '@' prefix
* we could use upper case elements tag names
* we could remove the 's' suffix in elements arrays
* we could change the `pos_infos` name for something else
* ...

More testing is required, especially the bit type and arrays.
Please, provide us with VOTable examples!


## Build

Install wasm-pack
> cargo install wasm-pack

To build, c.f. wasm-bindgen [doc](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html):
> wasm-pack build --out-name vot --target web --no-typescript --release


## Insert in your own web page

Download the last `vot-wasm-vxx.tar.gz` from the[github release page](https://github.com/cds-astro/cds-votable-rust/releases).
Put it in the same directory of you web page and decompress it:
```bash
tar xvzf vot-wasm-vxx.tar.gz
```
And add this in your HTML body (see example in [index.html](index.html)):
```html
    <script type="module">
      import init, * as vot from './pkg/vot.js';
      async function run() {
        const wasm = await init().catch(console.error);
	    window.vot = vot;
      }
      run();
    </script>
```

You should then be able to call the following methods:
```javascript
vot.fromXML(string) -> JsObject
vot.toXML(JsObject) -> String
vot.fromJSON(string) -> JsObject
vot.toJSON(JsObject) -> String
vot.fromTOML(string) -> JsObject
vot.toTOML(JsObject) -> String
vot.fromYAML(string) -> JsObject
vot.toYAML(JsObject) -> String
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

