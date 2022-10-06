<meta charset="utf-8"/>

# `votable-wasm` or `VOTWasm`

WebAssembly Library to convert IVOA [VOTables](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html)
in Json Objects, XML, JSON, YAML and TOML.

## About

TBW


## Build

Install wasm-pack
> cargo install wasm-pack

To build, c.f. wasm-bindgen [doc](https://rustwasm.github.io/docs/wasm-bindgen/reference/deployment.html):
> wasm-pack build --out-name vot --target web --no-typescript --release


## Insert in your web page

See example in [index.html](index.html), simply add the following code snippet in your web page
(plus the `pkg` ou build or retrieve from the github release section):
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
vot.toSML(JsObject) -> String
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

