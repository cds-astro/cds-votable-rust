
use std::{
  fs::File,
  str::FromStr,
  path::PathBuf,
  io::{stdin, stdout, BufRead, BufReader, Write, BufWriter}
};

use clap::Parser;

use votable::{
  error::VOTableError,
  votable::VOTableWrapper,
  impls::mem::InMemTableDataRows
};


#[derive(Debug, Copy, Clone)]
pub enum Format {
  XML,
  JSON,
  YAML,
  TOML,
}
impl FromStr for Format {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "xml" => Ok(Format::XML),
      "json" => Ok(Format::JSON),
      "yaml" => Ok(Format::YAML),
      "toml" => Ok(Format::TOML),
      _ => Err(format!("Unrecognized format. Actual: '{}'. Expected: 'xml', 'json', 'yaml' or 'toml'", s)),
    }
  }
}
impl Format {

  fn get<R: BufRead>(self, reader: R) -> Result<VOTableWrapper<InMemTableDataRows>, VOTableError> {
    match self {
      Format::XML => VOTableWrapper::<InMemTableDataRows>::from_ivoa_xml_reader(reader),
      Format::JSON => VOTableWrapper::<InMemTableDataRows>::from_json_reader(reader),
      Format::YAML => VOTableWrapper::<InMemTableDataRows>::from_yaml_reader(reader),
      Format::TOML => VOTableWrapper::<InMemTableDataRows>::from_toml_reader(reader),
    }
  }

  fn put<W: Write>(self, mut vot: VOTableWrapper<InMemTableDataRows>, writer: W, pretty: bool) -> Result<(), VOTableError> {
    match self {
      Format::XML => vot.to_ivoa_xml_writer(writer),
      Format::JSON => vot.to_json_writer(writer, pretty),
      Format::YAML => vot.to_yaml_writer(writer),
      Format::TOML => vot.to_toml_writer(writer, pretty),
    }
  }
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// Format of the input document ('xml', 'json', 'yaml' or 'toml').
  #[clap(value_enum)]
  input_fmt: Format,
  /// Format of the output document ('xml', 'json', 'yaml' or 'toml').
  #[clap(value_enum)]
  output_fmt: Format,
  /// Input file (else read from stdin)
  #[clap(short, long, value_name = "FILE")]
  input: Option<PathBuf>,
  /// Output file (else write to stdout)
  #[clap(short, long, value_name = "FILE")]
  output: Option<PathBuf>,
  /// Pretty print (for JSON and TOML)
  #[clap(short, long)]
  pretty: bool
}

fn main() -> Result<(), VOTableError> {
  let args = Args::parse();
  let vot = match args.input {
    Some(path) => {
      let file = File::open(path).map_err(VOTableError::Io)?;
      let reader = BufReader::new(file);
      args.input_fmt.get(reader)
    },
    None => {
      let stdin = stdin();
      let handle = stdin.lock();
      args.input_fmt.get(handle)
    },
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
