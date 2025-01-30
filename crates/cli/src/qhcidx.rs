use std::{fs, fs::File, path::PathBuf};

use clap::Args;
use memmap2::MmapOptions;

use cdshealpix::{
  n_hash,
  nested::sort::cindex::{FITSCIndex, HCIndex},
};

use votable::VOTableError;

/// Query a HEALPix sorted and indexed VOTable.
#[derive(Debug, Args)]
pub struct QueryHealpixCumulIndex {
  /// Path of the input HEALPix Cumulative Index FITS file (it contains the name of the VOTable to query).
  #[clap(short = 'i', long = "in", value_name = "NAME.hci.fits")]
  input: PathBuf,
  /// Depth of the queried HEALPix cell.
  #[arg(long)]
  depth: u8,
  /// Index of the queried HEALPix cell.
  #[arg(long)]
  ipix: u64,
  // add: HPX Range, MOC, STC-S and MultiCone queries?
}

impl QueryHealpixCumulIndex {
  pub fn exec(self) -> Result<(), VOTableError> {
    match FITSCIndex::from_fits_file(self.input).map_err(|e| VOTableError::Custom(e.to_string()))? {
      FITSCIndex::U64(fits_hci) => {
        let votable_name = fits_hci
          .get_indexed_file_name()
          .expect("No file name found in the FITS HCI file.");
        let expected_vot_len = fits_hci
          .get_indexed_file_len()
          .expect("No file length found in the FITS HCI file.");
        // Check if file exists
        fs::exists(votable_name)
          .map_err(|e| VOTableError::Custom(e.to_string()))
          .and_then(|exists| {
            if exists {
              Ok(())
            } else {
              Err(VOTableError::Custom(format!(
                "File `{}` not found in the current directory.",
                votable_name
              )))
            }
          })?;
        // Check file len
        let actual_vot_len = fs::metadata(votable_name)
          .map(|metadata| metadata.len())
          .map_err(|e| VOTableError::Custom(e.to_string()))?;
        if actual_vot_len != expected_vot_len {
          return Err(VOTableError::Custom(format!(
            "Local VOTable `{}` len does not match index info. Expected: {}. Actual: {}.",
            votable_name, expected_vot_len, actual_vot_len
          )));
        }
        // Ok, load index data
        let hci = fits_hci.get_hcindex();
        // Checl len
        let depth = hci.depth();
        if depth < self.depth {
          return Err(VOTableError::Custom(format!(
            "Query depth ({}) larger than index depth ({}) not yet supported.",
            depth, self.depth,
          )));
        }
        // Performs the query
        let data_start = hci.get(0) as usize;
        let data_end = hci.get(n_hash(depth) as usize) as usize;
        let bytes_range = hci.get_cell(self.depth, self.ipix);
        let file = File::open(votable_name).map_err(VOTableError::Io)?;
        let mmap = unsafe { MmapOptions::new().map(&file).map_err(VOTableError::Io)? };
        let mut stdout = std::io::stdout();
        std::io::copy(&mut &mmap[0..data_start], &mut stdout)
          .and_then(|_| {
            std::io::copy(
              &mut &mmap[bytes_range.start as usize..bytes_range.end as usize],
              &mut stdout,
            )
          })
          .and_then(|_| std::io::copy(&mut &mmap[data_end..actual_vot_len as usize], &mut stdout))
          .map(|_| ())
          .map_err(VOTableError::Io)
      }
      _ => Err(VOTableError::Custom(String::from(
        "Wrong data type in the FITS Healpix Cumulative Index type. Expected: u64.",
      ))),
    }
  }
}
