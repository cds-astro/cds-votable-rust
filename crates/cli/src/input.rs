use std::{
  fs::File,
  io::{stdin, BufRead, BufReader},
  path::{Path, PathBuf},
  str::FromStr,
};

use clap::Args;

use votable::{error::VOTableError, impls::mem::InMemTableDataRows, votable::VOTableWrapper};

#[derive(Debug, Copy, Clone)]
pub enum InputFormat {
  Xml,
  Json,
  Yaml,
  Toml,
}
impl FromStr for InputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "xml" => Ok(Self::Xml),
      "json" => Ok(Self::Json),
      "yaml" => Ok(Self::Yaml),
      "toml" => Ok(Self::Toml),
      _ => Err(format!(
        "Unrecognized format. Actual: '{}'. Expected: 'xml', 'json', 'yaml' or 'toml'",
        s
      )),
    }
  }
}
impl InputFormat {
  pub fn is_xml(&self) -> bool {
    matches!(self, Self::Xml)
  }

  /// Guess the file format from the given path extension.
  pub fn from_extension(path: &Path) -> Result<Self, String> {
    match path.extension().and_then(|e| e.to_str()) {
      Some("vot") | Some("xml") => Ok(Self::Xml),
      Some("json") => Ok(Self::Json),
      Some("yaml") | Some("yml") => Ok(Self::Yaml),
      Some("toml") => Ok(Self::Toml),
      _ => Err(String::from(
        "Unable to guess the format from the file extension, see options.",
      )),
    }
  }
  fn get<R: BufRead>(self, reader: R) -> Result<VOTableWrapper<InMemTableDataRows>, VOTableError> {
    match self {
      InputFormat::Xml => VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_reader(reader),
      InputFormat::Json => VOTableWrapper::<InMemTableDataRows>::from_json_reader(reader),
      InputFormat::Yaml => VOTableWrapper::<InMemTableDataRows>::from_yaml_reader(reader),
      InputFormat::Toml => VOTableWrapper::<InMemTableDataRows>::from_toml_reader(reader),
    }
  }
}

/// General VOTable input arguments.
#[derive(Debug, Args)]
pub struct Input {
  /// Path of the input VOTable [default: read from stdin]
  #[clap(short = 'i', long = "in", value_name = "FILE")]
  pub input: Option<PathBuf>,
  /// Format of the input VOTable ('xml', 'json', 'yaml' or 'toml') [default: guess from file extension]
  #[clap(short = 't', long = "in-fmt", value_enum)]
  pub input_fmt: Option<InputFormat>,
}

impl Input {
  pub fn is_stdin(&self) -> bool {
    self.input.is_none()
  }

  pub fn is_streamable(&self) -> Result<bool, VOTableError> {
    self.get_fmt().map(|f| f.is_xml())
  }

  pub fn get_fmt(&self) -> Result<InputFormat, VOTableError> {
    match &self.input_fmt {
      Some(input_fmt) => Ok(*input_fmt),
      None => match &self.input {
        Some(path) => InputFormat::from_extension(path).map_err(VOTableError::Custom),
        None => Err(VOTableError::Custom(String::from(
          "Input format **must** be provided when reading from stdin.",
        ))),
      },
    }
  }

  pub fn load(&self) -> Result<VOTableWrapper<InMemTableDataRows>, VOTableError> {
    match &self.input {
      Some(path) => self.load_from_path(path),
      None => self.load_from_stdin(),
    }
  }

  fn load_from_path(
    &self,
    path: &PathBuf,
  ) -> Result<VOTableWrapper<InMemTableDataRows>, VOTableError> {
    let file = File::open(path).map_err(VOTableError::Io)?;
    let reader = BufReader::new(file);
    match self.input_fmt {
      Some(input_fmt) => input_fmt.get(reader),
      None => InputFormat::from_extension(path)
        .map_err(VOTableError::Custom)
        .and_then(|input_fmt| input_fmt.get(reader)),
    }
  }

  fn load_from_stdin(&self) -> Result<VOTableWrapper<InMemTableDataRows>, VOTableError> {
    let stdin = stdin();
    let handle = stdin.lock();
    match self.input_fmt {
      Some(input_fmt) => input_fmt.get(handle),
      None => Err(VOTableError::Custom(String::from(
        "Input format **must** be provided when reading from stdin.",
      ))),
    }
  }
}
