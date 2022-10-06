
use std::{
  sync::{RwLock, Once},
  str::from_utf8_unchecked
};

extern crate console_error_panic_hook;
use unreachable::{UncheckedOptionExt, UncheckedResultExt};

use wasm_bindgen::{
  prelude::*,
  JsValue, JsCast
};


use web_sys::{Event, FileReader, HtmlInputElement};

use serde::ser::Serialize;

use votable::{
  votable::VOTableWrapper,
  impls::mem::InMemTableDataRows
};


type VOT = VOTableWrapper<InMemTableDataRows>;


////////////////////////
// IMPORT JS FONCTION //
// see https://rustwasm.github.io/docs/wasm-bindgen/examples/console-log.html
#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}


/// Activate debugging mode (Rust stacktrace)
#[wasm_bindgen(js_name = "debugOn")]
pub fn debug_on() {
  console_error_panic_hook::set_once();
}


// XML

#[wasm_bindgen(js_name = "fromXML", catch)]
pub fn from_xml(xml: &str) -> Result<JsValue, JsValue> {
  let vot = VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_str(xml)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(js_name = "toXML", catch)]
pub fn to_xml(js_val: JsValue) -> Result<String, JsValue> {
  let mut vot: VOT = serde_wasm_bindgen::from_value(js_val)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.to_ivoa_xml_string()
    .map_err(|e| JsValue::from_str(&e.to_string()))
}


// JSON

#[wasm_bindgen(js_name = "fromJSON", catch)]
pub fn from_json(json: &str) -> Result<JsValue, JsValue> {
  let vot = VOTableWrapper::<InMemTableDataRows>::from_json_str(json)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(js_name = "toJSON", catch)]
pub fn to_json(js_val: JsValue, pretty: bool) -> Result<String, JsValue> {
  let mut vot: VOT = serde_wasm_bindgen::from_value(js_val)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.to_json_string(pretty)
    .map_err(|e| JsValue::from_str(&e.to_string()))
}


// TOML

#[wasm_bindgen(js_name = "fromTOML", catch)]
pub fn from_toml(toml: &str) -> Result<JsValue, JsValue> {
  let vot = VOTableWrapper::<InMemTableDataRows>::from_toml_str(toml)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(js_name = "toTOML", catch)]
pub fn to_toml(js_val: JsValue, pretty: bool) -> Result<String, JsValue> {
  let mut vot: VOT = serde_wasm_bindgen::from_value(js_val)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.to_toml_string(pretty)
    .map_err(|e| JsValue::from_str(&e.to_string()))
}


// YAML

#[wasm_bindgen(js_name = "fromYAML", catch)]
pub fn from_yaml(yaml: &str) -> Result<JsValue, JsValue> {
  let vot = VOTableWrapper::<InMemTableDataRows>::from_yaml_str(yaml)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.serialize(&serde_wasm_bindgen::Serializer::json_compatible())
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(js_name = "toYAML", catch)]
pub fn to_yaml(js_val: JsValue) -> Result<String, JsValue> {
  let mut vot: VOT = serde_wasm_bindgen::from_value(js_val)
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
  vot.to_yaml_string()
    .map_err(|e| JsValue::from_str(&e.to_string()))
}


// Methods to load filess


/// Fonction used only once to init the last loaded
static LAST_LOADED_INIT: Once = Once::new();
/// The last loaded file, protected from concurrent access by a RwLock.
static mut LAST_LOADED: Option<RwLock<Result<JsValue, JsValue>>> = None;

/// Get (or create and get) the read/write protected last loaded file
/// All read/write  operations on the store have to call this method.
pub(crate) fn get_last_loaded() -> &'static RwLock<Result<JsValue, JsValue>> {
  unsafe {
    // Inspired from the Option get_or_insert_with method, modified to ensure thread safety with
    // https://doc.rust-lang.org/std/sync/struct.Once.html
    // This implements a double-checked lock.
    if let None = LAST_LOADED {
      LAST_LOADED_INIT.call_once(|| {
        LAST_LOADED = Some(RwLock::new(Ok(JsValue::NULL)));
      });
    }
    match &LAST_LOADED {
      Some(v) => v,
      None => unreachable!(),
    }
  }
}

/// Put the content of the last read file in the global variable
pub(crate) fn set_last_loaded(res: Result<JsValue, JsValue>) -> Result<(), JsValue> {
  let mut store = get_last_loaded().write().map_err(|_| JsValue::from_str("Write lock poisoned"))?;
  (*store) = res;
  Ok(())
}

/// Returns the String content of the last read file
#[wasm_bindgen(js_name = "getLastLoaded", catch)]
pub fn get_last_loaded_file() -> Result<JsValue, JsValue> {
  let mut store = get_last_loaded().write().map_err(|_| JsValue::from_str("Write lock poisoned"))?;
  std::mem::replace(&mut *store, Ok(JsValue::NULL))
}

/// Open the file selection dialog and load the VOTable contained in the selected file
/// (for security reasons, we cannot simply provide a path on the client machine).
/// # Info
/// * For Json and Ascii file, requires the type of MOC to be loaded (Space, Time or Space-Time)
/// # Warning
/// Because of security restriction, the call to this method
/// **"needs to be triggered within a code block that was the handler of a user-initiated event"**
#[wasm_bindgen(js_name = "fromLocalFile", catch)]
pub fn from_local_file() -> Result<(), JsValue> {
  // Create the file input action that will be fired by the event 'change'
  let file_input_action = Closure::wrap(Box::new(move |event: Event| {
    let element = unsafe { event.target().unchecked_unwrap().dyn_into::<HtmlInputElement>().unchecked_unwrap_ok() };
    let file_list = unsafe {  element.files().unchecked_unwrap() };
    if file_list.length() > 0 {
      let file = unsafe {  file_list.get(0).unchecked_unwrap() };
      let file_name = file.name();
      let file_reader = unsafe {  FileReader::new().unchecked_unwrap_ok() };
      // There is a stream method, but I am not sure how to use it. I am so far going the easy way.
      match file_reader.read_as_array_buffer(&file) {
        Err(_) => log("Error reading file content"),
        _ => { },
      };
      let file_onload = Closure::wrap(Box::new(move |event: Event| {
        let file_reader: FileReader = unsafe { event.target().unchecked_unwrap().dyn_into().unchecked_unwrap_ok() };
        let file_content = unsafe { file_reader.result().unchecked_unwrap_ok() };
        let file_content: Vec<u8> = js_sys::Uint8Array::new(&file_content).to_vec();
        let file_content_str = unsafe{ from_utf8_unchecked(&file_content) };
        // We accept only files with given extensions so splitting on "." should be safe.
        let (_name, ext) = unsafe { file_name.rsplit_once('.').unchecked_unwrap() };
        let res = match ext {
          "xml" | "vot" | "b64" => from_xml(&file_content_str),
          "json" => from_json(&file_content_str),
          "toml" => from_toml(&file_content_str),
          "yaml" => from_yaml(&file_content_str),
          _ => unreachable!(), // since file_input.set_attribute("accept", ".vot, .xml, .json, .toml, .yaml");
        };
        // put in the globla variable
        match set_last_loaded(res) {
          Err(e) => log(format!("Error acquiring lock: {:?}", e).as_str()),
          _ => { },
        }
      }) as Box<dyn FnMut(_)>);
      file_reader.set_onload(Some(file_onload.as_ref().unchecked_ref()));
      file_onload.forget();
    }
  }) as Box<dyn FnMut(_)>);

  // Create a temporary input file and click on it
  // - get the body
  let window = web_sys::window().expect("no global `window` exists");
  // This could be used but not yet in web_sys: https://developer.mozilla.org/en-US/docs/Web/API/Window/showOpenFilePicker
  let document = window.document().expect("should have a document on window");
  let body = document.body().expect("document should have a body");
  // - create the input
  let file_input: HtmlInputElement = unsafe { document.create_element("input").unchecked_unwrap_ok().dyn_into()? };
  file_input.set_type("file");
  unsafe {
    file_input.set_attribute("hidden", "").unchecked_unwrap_ok();
    file_input.set_attribute("accept", ".vot, .b64, .xml, .json, .toml, .yaml").unchecked_unwrap_ok();
  }
  file_input.add_event_listener_with_callback("change", file_input_action.as_ref().unchecked_ref())?;
  file_input_action.forget();
  // - attach the input
  body.append_child(&file_input)?;
  // - simulate a click
  file_input.click();
  // - remove the input
  body.remove_child(&file_input)?;
  Ok(())
}


// v.votable.resources[0].tables[0].data.rows[0]
// v.votable.resources[0].tables[0].data.stream.rows[0]

/*
vot.fromLocalFile();
var v = vot.getLastLoaded();
var x = vot.toXML(v);
*/

