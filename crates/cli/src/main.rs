use clap::Parser;

use votable::error::VOTableError;

use votable_cli::{
  convert::Convert, edit::Edit, get::Get, hcidx::HealpixCumulIndex, hpxsort::HpxSort,
  qhcidx::QueryHealpixCumulIndex, streaming::StreamConvert,
};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None, rename_all = "lower")]
pub enum CliArgs {
  Convert(Convert),
  #[command(verbatim_doc_comment)]
  Sconvert(StreamConvert),
  Edit(Edit),
  Get(Get),
  HSort(HpxSort), // InMem or streaming mode, guess position from pos.eq.ra;meta.main or pos.eq.ra or user provided
  HCIdx(HealpixCumulIndex), //HEALPix Cumulative Index
  QHCIdx(QueryHealpixCumulIndex), // Query using a HEALPix Cumulative Index (name of columns and file taken in the HCI FITS file
}

impl CliArgs {
  pub fn exec(self) -> Result<(), VOTableError> {
    match self {
      Self::Convert(p) => p.exec(),
      Self::Sconvert(p) => p.exec(),
      Self::Edit(p) => p.exec(),
      Self::Get(p) => p.exec(),
      Self::HSort(p) => p.exec(),
      Self::HCIdx(p) => p.exec(),
      Self::QHCIdx(p) => p.exec(),
    }
  }
}

fn main() -> Result<(), VOTableError> {
  let args = CliArgs::parse();
  env_logger::init();
  args.exec()
}
