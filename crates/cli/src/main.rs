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
  XML,
  JSON,
  YAML,
  TOML,
}
impl FromStr for InputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "xml" => Ok(InputFormat::XML),
      "json" => Ok(InputFormat::JSON),
      "yaml" => Ok(InputFormat::YAML),
      "toml" => Ok(InputFormat::TOML),
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
      InputFormat::XML => VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_reader(reader),
      InputFormat::JSON => VOTableWrapper::<InMemTableDataRows>::from_json_reader(reader),
      InputFormat::YAML => VOTableWrapper::<InMemTableDataRows>::from_yaml_reader(reader),
      InputFormat::TOML => VOTableWrapper::<InMemTableDataRows>::from_toml_reader(reader),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub enum OutputFormat {
  XML,
  XML_TABLEDATA,
  XML_BINARY,
  XML_BINARY2,
  JSON,
  YAML,
  TOML,
}
impl FromStr for OutputFormat {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "xml" => Ok(OutputFormat::XML),
      "xml-td" => Ok(OutputFormat::XML_TABLEDATA),
      "xml-bin" => Ok(OutputFormat::XML_BINARY),
      "xml-bin2" => Ok(OutputFormat::XML_BINARY2),
      "json" => Ok(OutputFormat::JSON),
      "yaml" => Ok(OutputFormat::YAML),
      "toml" => Ok(OutputFormat::TOML),
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
      OutputFormat::XML => vot.to_ivoa_xml_writer(writer),
      OutputFormat::XML_TABLEDATA => {
        vot.to_tabledata()?;
        vot.to_ivoa_xml_writer(writer)
      }
      OutputFormat::XML_BINARY => {
        vot.to_binary()?;
        vot.to_ivoa_xml_writer(writer)
      }
      OutputFormat::XML_BINARY2 => {
        vot.to_binary2()?;
        vot.to_ivoa_xml_writer(writer)
      }
      OutputFormat::JSON => vot.to_json_writer(writer, pretty),
      OutputFormat::YAML => vot.to_yaml_writer(writer),
      OutputFormat::TOML => vot.to_toml_writer(writer, pretty),
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
