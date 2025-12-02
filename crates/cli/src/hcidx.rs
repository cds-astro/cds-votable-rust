// Methode to_owned_tabledata_row_iterator_with_position
// de SimpleVOTableRowIterator

use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Args;
use log::{error, info};

use cdshealpix::{
  n_hash,
  nested::{
    get,
    sort::cindex::{HCIndex, OwnedCIndex},
  },
};

use votable::{
  data::{tabledata::FieldIteratorUnbuffered, TableOrBinOrBin2},
  iter::SimpleVOTableRowIterator,
  VOTableError,
};

use crate::hpxsort::{get_fields, look_for_positional_columns};

/// Create an index on an HEALPix sorted VOTable to then quickly retrieve rows in a given HEALPix cell.
#[derive(Debug, Args)]
pub struct HealpixCumulIndex {
  /// Path of the input, HEALPix sorted, XML-DATATABLE VOTable file.
  #[clap(short = 'i', long = "in", value_name = "FILE")]
  input: PathBuf,
  /// Path of the output FITS file containing the HEALPix Cumulative Index.
  #[clap(short = 'o', long = "out", value_name = "FILE")]
  output: PathBuf,
  /// Column name of the longitude used to compute the HEALPix number that has been used to sort the table.
  /// By default, look for 1st UCD `pos.eq.ra;meta.main`, then for 1st UCD `pos.eq.ra`,
  /// then for 1st colum name starting by `RA`.
  /// WARNING: column must be in decimal degrees.
  #[arg(short = 'l', long = "lon")]
  longitude: Option<String>,
  /// Column name of the latitude used to compute the HEALPix number that has been used to sort the table.
  /// By default, look for 1st UCD `pos.eq.dec;meta.main`, then for 1st UCD `pos.eq.dec`,
  /// then for 1st colum name starting by `Dec`.
  /// WARNING: column must be in decimal degrees.
  #[arg(short = 'b', long = "lat")]
  latitude: Option<String>,
  /// Depth of the HEALPix cumulative index (around 6 to 10, then output file will be large).
  #[arg(long, default_value_t = 8_u8)]
  depth: u8,
}

impl HealpixCumulIndex {
  pub fn exec(self) -> Result<(), VOTableError> {
    // Read from input
    let in_vot_it = SimpleVOTableRowIterator::from_file(self.input.as_path())?;
    // Look for RA/Dec columns
    let fields = get_fields(&in_vot_it.votable);
    let (ilon, ilat) = look_for_positional_columns(
      fields.as_slice(),
      self.longitude.as_ref(),
      self.latitude.as_ref(),
    )?;
    let colname_lon = &fields[ilon].name.clone();
    let colname_lat = &fields[ilat].name.clone();
    info!(
      "Columns used to compute HEALPix index are lon='{}' and lat='{}'.",
      &colname_lon, &colname_lat
    );
    // RA/Dec extraction method
    let (n1, n2, rev) = if ilon < ilat {
      (ilon, ilat - ilon - 1, false)
    } else {
      (ilat, ilon - ilat - 1, true)
    };
    let layer = get(self.depth);
    let hpx = move |raw_td_row: &Vec<u8>| {
      let mut field_it = FieldIteratorUnbuffered::new(raw_td_row);
      let coo1 = field_it
        .nth(n1)
        .map(|s| s.and_then(|s| s.parse::<f64>().map_err(VOTableError::ParseFloat)));
      let coo2 = field_it
        .nth(n2)
        .map(|s| s.and_then(|s| s.parse::<f64>().map_err(VOTableError::ParseFloat)));
      match (coo1, coo2) {
        (Some(Ok(coo1)), Some(Ok(coo2))) => {
          if !rev {
            layer.hash(coo1.to_radians(), coo2.to_radians())
          } else {
            layer.hash(coo2.to_radians(), coo1.to_radians())
          }
        }
        (Some(Err(e)), _) => {
          error!("Error parsing RA value: {}. Assigned HEALPix cell is 0.", e);
          0
        }
        (_, Some(Err(e))) => {
          error!(
            "Error parsing Dec value: {}. Assigned HEALPix cell is 0.",
            e
          );
          0
        }
        _ => {
          error!("Error retrieving coordinates: Check your VOTable with votlint, ensure all rows do have positions, in decimal degrees. Exec with flag 'RUST_LOG=\"trace\"' for more information.");
          0
        }
      }
    };

    // Prepare output
    let fits_file = File::create(self.output).map_err(VOTableError::Io)?;
    let out_fits_write = BufWriter::new(fits_file);
    // Do indexation
    match in_vot_it.data_type() {
      TableOrBinOrBin2::TableData => {
        // Build the cumulative map
        let mut end_byte = 0;
        let len = n_hash(self.depth) + 1;
        let mut map: Vec<u64> = Vec::with_capacity(len as usize);
        for (irow, res) in in_vot_it
          .to_owned_tabledata_row_iterator_with_position()
          .enumerate()
        {
          let (byte_range, raw_row) = res?;
          let icell = hpx(&raw_row);
          if icell + 1 < map.len() as u64 {
            return Err(VOTableError::Custom(format!(
              "HEALPix error at row {}: the file seems not ot be sorted!",
              irow
            )));
          }
          // Push only the strating byte of the first row having a given cell number.
          // Copy the value for all empty cells between two non-empty cells.
          for _ in map.len() as u64..=icell {
            //info!("Push row: {}; bytes: {:?}", irow, &byte_range);
            map.push(byte_range.start as u64);
          }
          end_byte = byte_range.end as u64;
        }
        for _ in map.len() as u64..len {
          map.push(end_byte);
        }
        // Write the cumulative map
        let file_metadata = self.input.metadata().ok();
        OwnedCIndex::new_unchecked(self.depth, map.into_boxed_slice())
          .to_fits(
            out_fits_write,
            self.input.file_name().and_then(|name| name.to_str()),
            file_metadata.as_ref().map(|meta| meta.len()),
            None, // So far we do not compute the md5 of the VOTable!
            file_metadata.as_ref().and_then(|meta| meta.modified().ok()),
            Some(colname_lon.as_str()),
            Some(colname_lat.as_str()),
          )
          .map_err(|e| VOTableError::Custom(e.to_string()))
      }
      _ => Err(VOTableError::Custom(String::from(
        "Only XML-DATATABLE VOTable can be indexed!",
      ))),
    }
  }
}
