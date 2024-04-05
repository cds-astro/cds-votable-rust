use std::io::{stderr, stdout, Write};

use clap::Parser;

use votable::error::VOTableError;
use votable_cli::{convert::Convert, edit::Edit, get::Get, streaming::StreamConvert};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub enum CliArgs {
  Convert(Convert),
  #[command(verbatim_doc_comment)]
  Sconvert(StreamConvert),
  Edit(Edit),
  Get(Get),
}

impl CliArgs {
  pub fn exec(self) -> Result<(), VOTableError> {
    match self {
      Self::Convert(p) => p.exec(),
      Self::Sconvert(p) => p.exec(),
      Self::Edit(p) => p.exec(),
      Self::Get(p) => p.exec(),
    }
  }
}

fn main() -> Result<(), VOTableError> {
  let args = CliArgs::parse();
  env_logger::init();
  args.exec().map_err(|e| {
    if let Err(e) = stdout().flush() {
      eprintln!("Error flushing stdout: {:?}", e);
    }
    if let Err(e) = stderr().flush() {
      eprintln!("Error flushing stderr: {:?}", e);
    }
    e
  })
}
