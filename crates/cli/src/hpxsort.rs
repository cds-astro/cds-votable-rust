use std::{
  fs::File,
  io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
  path::PathBuf,
};

use clap::Args;
use log::{error, info, warn};

use cdshealpix::nested::{
  get,
  sort::{hpx_external_sort_stream, hpx_internal_sort, SimpleExtSortParams},
};
use votable::{
  data::{tabledata::FieldIteratorUnbuffered, TableOrBinOrBin2},
  datatype::Datatype,
  error::VOTableError,
  iter::SimpleVOTableRowIterator,
  votable::new_xml_writer,
  Field, TableElem, VOTable, VoidTableDataContent,
};

/// Sort a single table VOTable (XML-TD only) by order 29 HEALPix indices, using an external sort.
#[derive(Debug, Args)]
pub struct HpxSort {
  /// Path of the input XML-DATATABLE VOTable file [default: read from stdin]
  #[clap(short = 'i', long = "in", value_name = "FILE")]
  input: Option<PathBuf>,
  /// Path of the output, HEALPix sorted, XML-DATATABLE VOTable file [default: write to stdout]
  #[clap(short = 'o', long = "out", value_name = "FILE")]
  output: Option<PathBuf>,
  /// Column name of the longitude used to compute the HEALPix number. By default, look for 1st UCD
  /// `pos.eq.ra;meta.main`, then for 1st UCD `pos.eq.ra`, then for 1st colum name starting by `RA`.
  /// WARNING: column must be in decimal degrees.
  #[arg(short = 'l', long = "lon")]
  longitude: Option<String>,
  /// Column name of the latitude used to compute the HEALPix number. By default, look for 1st UCD
  /// `pos.eq.dec;meta.main`, then for 1st UCD `pos.eq.dec`, then for 1st colum name starting by `Dec`.
  /// WARNING: column must be in decimal degrees.
  #[arg(short = 'b', long = "lat")]
  latitude: Option<String>,
  /// Set the number of threads used [default: use all threads available on the machine]
  #[arg(long, value_name = "N")]
  parallel: Option<usize>,
  /// Fully in memory, do not use external sort. Faster that external sort, for table holding in RAM..
  #[arg(short = 'f', long = "full-in-mem")]
  fully_in_memory: bool,
  /// Directory containing the temporary directories/files for external sort.
  #[arg(long, default_value = ".sort_tmp/")]
  tmp_dir: PathBuf,
  /// Number of rows per external sort chunk (2 chunks are simultaneously loaded in memory).
  #[arg(long, default_value_t = 50_000_usize)]
  chunk_size: usize,
  /// Depth of the computed HEALPix count map for the external sort. Should be deep enough so that
  /// the largest count map value is smaller than `chunk-size`.
  #[arg(long, default_value_t = 8_u8)]
  depth: u8,
  /// Save the computed count map in the given FITS file path.
  #[arg(long)]
  count_map_path: Option<PathBuf>,
}

impl HpxSort {
  pub fn exec(self) -> Result<(), VOTableError> {
    self.choose_input_and_exec()
  }

  pub fn choose_input_and_exec(self) -> Result<(), VOTableError> {
    match &self.input {
      Some(path) => {
        // We could call a different method to re-parse the file instead of writing temporary
        // rows on the disk (for the two stages of the sort).
        SimpleVOTableRowIterator::from_file(path).and_then(|it| self.choose_output_and_exec(it))
      }
      None => {
        let stdin = stdin();
        SimpleVOTableRowIterator::from_reader(BufReader::new(stdin))
          .and_then(|it| self.choose_output_and_exec(it))
      }
    }
  }

  pub fn choose_output_and_exec<R: BufRead + Send>(
    self,
    it: SimpleVOTableRowIterator<R>,
  ) -> Result<(), VOTableError> {
    match &self.output {
      Some(path) => {
        let file = File::create(path).map_err(VOTableError::Io)?;
        let write = BufWriter::new(file);
        self.do_exec_gen(it, write)
      }
      None => {
        let stdout = stdout();
        let handle = stdout.lock();
        self.do_exec_gen(it, handle)
      }
    }
  }

  pub fn do_exec_gen<R, W>(
    self,
    it: SimpleVOTableRowIterator<R>,
    write: W,
  ) -> Result<(), VOTableError>
  where
    R: BufRead + Send,
    W: Write,
  {
    match it.data_type() {
      TableOrBinOrBin2::TableData => self.process(it, write),
      _ => Err(VOTableError::Custom(String::from(
        "Only XML-DATATABLE VOTable are so far supported!",
      ))),
    }
  }

  fn process<R: BufRead, W: Write>(
    self,
    mut it: SimpleVOTableRowIterator<R>,
    write: W,
  ) -> Result<(), VOTableError> {
    let mut writer = new_xml_writer(write, None, None);
    if it
      .votable
      .write_to_data_beginning(&mut writer, &(), false)?
    {
      // Look for RA/Dec columns
      let fields = get_fields(&it.votable);
      let (ilon, ilat) = look_for_positional_columns(
        fields.as_slice(),
        self.longitude.as_ref(),
        self.latitude.as_ref(),
      )?;
      info!(
        "Columns used to compute HEALPix index are lon='{}' and lat='{}'.",
        fields[ilon].name, fields[ilat].name
      );
      // RA/Dec extraction method
      let (n1, n2, rev) = if ilon < ilat {
        (ilon, ilat - ilon - 1, false)
      } else {
        (ilat, ilon - ilat - 1, true)
      };
      let layer29 = get(29);
      let hpx29 = move |raw_td_row: &Vec<u8>| {
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
              layer29.hash(coo1.to_radians(), coo2.to_radians())
            } else {
              layer29.hash(coo2.to_radians(), coo1.to_radians())
            }
          }
          (Some(Err(e)), _) => {
            warn!("Error parsing RA value: {}. Assigned HEALPix cell is 0.", e);
            0
          }
          (_, Some(Err(e))) => {
            warn!(
              "Error parsing Dec value: {}. Assigned HEALPix cell is 0.",
              e
            );
            0
          }
          _ => {
            error!("Error retrieving coordinates: Check your VOTable with votlint, ensure all rows have positions, ensure positions at indices ({}, {}) are in decimal degrees.", ilon, ilat);
            0
          }
        }
      };

      let mut raw_row_it = it.to_owned_tabledata_row_iterator();
      if self.fully_in_memory {
        // Internal sort
        let mut rows = (&mut raw_row_it).collect::<Result<Vec<Vec<u8>>, _>>()?;
        hpx_internal_sort(rows.as_mut_slice(), hpx29, self.parallel);
        let w = &mut writer.inner();
        // Write the result
        for row in rows {
          w.write_all(b"<TR>")
            .and_then(|()| w.write_all(row.as_slice()))
            .and_then(|()| w.write_all(b"</TR>\n"))
            .map_err(VOTableError::Io)?;
        }
      } else {
        // External sort
        let w = &mut writer.inner();
        let params =
          SimpleExtSortParams::new(self.tmp_dir, self.chunk_size as u32, self.parallel, true);
        let sorted_raw_row_it = hpx_external_sort_stream(
          &mut raw_row_it,
          hpx29,
          self.depth,
          self.count_map_path,
          Some(params),
        )
          .map_err(|e| VOTableError::Custom(e.to_string()))?;
        for row_res in sorted_raw_row_it {
          let row = row_res.map_err(|e| VOTableError::Custom(e.to_string()))?;
          w.write_all(b"<TR>")
            .and_then(|()| w.write_all(row.as_slice()))
            .and_then(|()| w.write_all(b"</TR>\n"))
            .map_err(VOTableError::Io)?;
        }
      }
      raw_row_it
        .read_to_end()
        .and_then(|mut out_vot| out_vot.write_from_data_end(&mut writer, &(), false))
    } else {
      // No table in the VOTable (all the VOTable already written in output).
      Ok(())
    }
  }
}

/// # Panics
/// if the given VOTable does not contain a table.
pub(crate) fn get_fields(votable: &VOTable<VoidTableDataContent>) -> Vec<&Field> {
  votable
    .get_first_table()
    .expect("No table found!")
    .elems
    .iter()
    .filter_map(|table_elem| match table_elem {
      TableElem::Field(field) => Some(field),
      _ => None,
    })
    .collect()
}

pub(crate) fn look_for_positional_columns(
  fields: &[&Field],
  lon_name: Option<&String>,
  lat_name: Option<&String>,
) -> Result<(usize, usize), VOTableError> {
  let ilon = lon_name
    .map(|colname| look_for_float_col_index_having_name(fields, colname.as_str()))
    .or_else(|| look_for_float_col_index_having_ucd(fields, "pos.eq.ra;meta.main"))
    .or_else(|| look_for_float_col_index_having_ucd(fields, "pos.eq.ra"))
    .unwrap_or_else(|| look_for_float_col_index_name_startswith(fields, "RA"))
    .map_err(VOTableError::Custom)?;
  let ilat = lat_name
    .map(|colname| look_for_float_col_index_having_name(fields, colname.as_str()))
    .or_else(|| look_for_float_col_index_having_ucd(fields, "pos.eq.dec;meta.main"))
    .or_else(|| look_for_float_col_index_having_ucd(fields, "pos.eq.de"))
    .unwrap_or_else(|| look_for_float_col_index_name_startswith(fields, "DE"))
    .map_err(VOTableError::Custom)?;
  Ok((ilon, ilat))
}

/// Returns the index in the given field array of the the first column of given column name.
/// The datatype of the column **must be** either Float or Double.
/// An error is raised if the column is not found (or if the datatype is not Float or Double).
fn look_for_float_col_index_having_name(fields: &[&Field], colname: &str) -> Result<usize, String> {
  for (i, field) in fields.iter().enumerate() {
    if field.name == colname {
      return match field.datatype {
        Datatype::Float | Datatype::Double => Ok(i),
        _ => Err(format!("Column '{}' is not a float or a double.", &colname)),
      };
    }
  }
  Err(format!("Column '{}' not found!", &colname))
}

fn look_for_float_col_index_having_ucd(
  fields: &[&Field],
  target_ucd: &str,
) -> Option<Result<usize, String>> {
  for (i, field) in fields.iter().enumerate() {
    if let Some(ucd) = &field.ucd && ucd.as_str() == target_ucd {
      return Some(match field.datatype {
        Datatype::Float | Datatype::Double => Ok(i),
        _ => Err(format!(
          "Column '{}' is not a float or a double.",
          &field.name
        )),
      });
    }
  }
  None
}

fn look_for_float_col_index_name_startswith(
  fields: &[&Field],
  prefix: &str,
) -> Result<usize, String> {
  for (i, field) in fields.iter().enumerate() {
    match field.datatype {
      Datatype::Float | Datatype::Double
      if field
        .name
        .to_lowercase()
        .starts_with(prefix.to_lowercase().as_str()) =>
        {
          return Ok(i);
        }
      _ => continue,
    }
  }
  Err(format!("Column starting with '{}' not found!", &prefix))
}
