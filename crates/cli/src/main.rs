use std::{
  fs::File,
  io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
  path::PathBuf,
  str::FromStr,
};

use clap::Parser;

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
      "xml" => Ok(InputFormat::Xml),
      "json" => Ok(InputFormat::Json),
      "yaml" => Ok(InputFormat::Yaml),
      "toml" => Ok(InputFormat::Toml),
      _ => Err(format!(
        "Unrecognized format. Actual: '{}'. Expected: 'xml', 'json', 'yaml' or 'toml'",
        s
      )),
    }
  }
}
impl InputFormat {
  fn get<R: BufRead>(self, reader: R) -> Result<VOTableWrapper<InMemTableDataRows>, VOTableError> {
    match self {
      InputFormat::Xml => VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_reader(reader),
      InputFormat::Json => VOTableWrapper::<InMemTableDataRows>::from_json_reader(reader),
      InputFormat::Yaml => VOTableWrapper::<InMemTableDataRows>::from_yaml_reader(reader),
      InputFormat::Toml => VOTableWrapper::<InMemTableDataRows>::from_toml_reader(reader),
    }
  }
}

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

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// Format of the input document ('xml', 'json', 'yaml' or 'toml').
  #[clap(value_enum)]
  input_fmt: InputFormat,
  /// Format of the output document ('xml', 'xml-td', 'xml-bin', 'xml-bin2', 'json', 'yaml' or 'toml').
  #[clap(value_enum)]
  output_fmt: OutputFormat,
  /// Input file (else read from stdin)
  #[clap(short, long, value_name = "FILE")]
  input: Option<PathBuf>,
  /// Output file (else write to stdout)
  #[clap(short, long, value_name = "FILE")]
  output: Option<PathBuf>,
  /// Pretty print (for JSON and TOML)
  #[clap(short, long)]
  pretty: bool,
}

fn main() -> Result<(), VOTableError> {
  let args = Args::parse();
  env_logger::init();
  let vot = match args.input {
    Some(path) => {
      let file = File::open(path).map_err(VOTableError::Io)?;
      let reader = BufReader::new(file);
      args.input_fmt.get(reader)
    }
    None => {
      let stdin = stdin();
      let handle = stdin.lock();
      args.input_fmt.get(handle)
    }
  }?;
  match args.output {
    Some(path) => {
      let file = File::create(path).map_err(VOTableError::Io)?;
      let write = BufWriter::new(file);
      args.output_fmt.put(vot, write, args.pretty)
    }
    None => {
      let stdout = stdout();
      let handle = stdout.lock();
      args.output_fmt.put(vot, handle, args.pretty)
    }
  }
}
