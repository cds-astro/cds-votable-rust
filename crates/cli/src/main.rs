use clap::Parser;

use votable::error::VOTableError;
use votable_cli::{convert::Convert, get::Get, streaming::StreamConvert /*update::Update*/};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub enum CliArgs {
  Convert(Convert),
  Sconvert(StreamConvert),
  // Update(Update),
  Get(Get),
}

impl CliArgs {
  pub fn exec(self) -> Result<(), VOTableError> {
    match self {
      Self::Convert(p) => p.exec(),
      Self::Sconvert(p) => p.exec(),
      // Self::Update(p) => p.exec(),
      Self::Get(p) => p.exec(),
    }
  }
}

fn main() -> Result<(), VOTableError> {
  let args = CliArgs::parse();
  env_logger::init();
  args.exec()
}
