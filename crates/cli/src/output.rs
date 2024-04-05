use std::{
  fs::File,
  io::{stdout, BufWriter, Write},
  path::PathBuf,
  str::FromStr,
};

use clap::Args;

use votable::{error::VOTableError, impls::mem::InMemTableDataRows, votable::VOTableWrapper};

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
  Xml,
  XmlTabledata,
  XmlBinary,
  XmlBinary2,
  Json,
  Yaml,
  Toml,
}
impl FromStr for OutputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "xml" => Ok(OutputFormat::Xml),
      "xml-td" => Ok(OutputFormat::XmlTabledata),
      "xml-bin" => Ok(OutputFormat::XmlBinary),
      "xml-bin2" => Ok(OutputFormat::XmlBinary2),
      "json" => Ok(OutputFormat::Json),
      "yaml" => Ok(OutputFormat::Yaml),
      "toml" => Ok(OutputFormat::Toml),
      _ => Err(format!(
        "Unrecognized format. Actual: '{}'. Expected: 'xml', 'xml-td', 'xml-bin', 'xml-bin2', 'json', 'yaml' or 'toml'",
        s
      )),
    }
  }
}
impl OutputFormat {
  pub fn is_streamable(&self) -> bool {
    match self {
      Self::Xml | Self::XmlTabledata | Self::XmlBinary | Self::XmlBinary2 => true,
      _ => false,
    }
  }
  fn put<W: Write>(
    self,
    mut vot: VOTableWrapper<InMemTableDataRows>,
    writer: W,
    pretty: bool,
  ) -> Result<(), VOTableError> {
    match self {
      OutputFormat::Xml => vot.to_ivoa_xml_writer(writer),
      OutputFormat::XmlTabledata => {
        vot.to_tabledata()?;
        vot.to_ivoa_xml_writer(writer)
      }
      OutputFormat::XmlBinary => {
        vot.to_binary()?;
        vot.to_ivoa_xml_writer(writer)
      }
      OutputFormat::XmlBinary2 => {
        vot.to_binary2()?;
        vot.to_ivoa_xml_writer(writer)
      }
      OutputFormat::Json => vot.to_json_writer(writer, pretty),
      OutputFormat::Yaml => vot.to_yaml_writer(writer),
      OutputFormat::Toml => vot.to_toml_writer(writer, pretty),
    }
  }
}

#[derive(Debug, Args)]
pub struct Output {
  /// Path of the output VOTable [default: write to stdout]
  #[clap(short = 'o', long = "out", value_name = "FILE")]
  pub output: Option<PathBuf>,
  /// Format of the output VOTable ('xml', 'xml-td', 'xml-bin', 'xml-bin2', 'json', 'yaml' or 'toml').
  #[clap(short = 'f', long = "out-fmt", value_enum)]
  pub output_fmt: OutputFormat,
  /// Pretty print (for JSON and TOML)
  #[clap(short, long)]
  pub pretty: bool,
}

impl Output {
  pub fn is_streamable(&self) -> bool {
    self.output_fmt.is_streamable()
  }

  pub fn save(&self, vot: VOTableWrapper<InMemTableDataRows>) -> Result<(), VOTableError> {
    match &self.output {
      Some(path) => {
        let file = File::create(path).map_err(VOTableError::Io)?;
        let write = BufWriter::new(file);
        self.output_fmt.put(vot, write, self.pretty)
      }
      None => {
        let stdout = stdout();
        let handle = stdout.lock();
        self.output_fmt.put(vot, handle, self.pretty)
      }
    }
  }
}
