use clap::Args;

use votable::error::VOTableError;

use super::{input::Input, output::Output};

/// Convert a VOTable from one format to another (full table loaded in memory).
#[derive(Debug, Args)]
pub struct Convert {
  #[command(flatten)]
  input: Input,
  #[command(flatten)]
  output: Output,
}

impl Convert {
  pub fn exec(self) -> Result<(), VOTableError> {
    self.input.load().and_then(|vot| self.output.save(vot))
  }
}
