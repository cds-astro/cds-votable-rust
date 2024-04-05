use std::{fmt::Alignment, io::stdin, slice::Iter};

use clap::{Args, Subcommand, ValueEnum};

use votable::{
  error::VOTableError,
  iter::{SimpleVOTableRowIterator, VOTableIterator},
  TableDataContent, VOTable,
};

use super::{
  input::Input,
  visitors::{
    colnames::ColnamesVisitor, fieldarray::FieldArrayVisitor, votstruct::AsciiStructVisitor,
  },
};

/// Get information from a VOTable, like it structure or fields.
#[derive(Debug, Args)]
pub struct Get {
  #[command(flatten)]
  input: Input,
  /// Stop parsing before reading first data ('xml' input only).
  #[arg(short = 's', long = "early-stop")]
  stop_at_first_data: bool,
  #[command(subcommand)]
  action: GetAction,
}
impl Get {
  pub fn exec(self) -> Result<(), VOTableError> {
    self.input.is_streamable().and_then(|is_streamable| {
      if is_streamable {
        self.exec_streaming()
      } else {
        self.exec_in_mem()
      }
    })
  }

  /// Exec loading the full VOTable in memory in case of JSON/YAML/TOML
  pub fn exec_in_mem(self) -> Result<(), VOTableError> {
    self
      .input
      .load()
      .and_then(|votw| self.action.exec(votw.unwrap()))
  }

  /// Exec in streaming mode if the input is XML.
  pub fn exec_streaming(self) -> Result<(), VOTableError> {
    if self.stop_at_first_data {
      match &self.input.input {
        Some(path) => SimpleVOTableRowIterator::from_file(path).map(|voti| voti.end_of_it()),
        None => {
          let stdin = stdin();
          let handle = stdin.lock();
          SimpleVOTableRowIterator::from_reader(handle).map(|voti| voti.end_of_it())
        }
      }
      .and_then(|vot| self.action.exec(vot))
    } else {
      match &self.input.input {
        Some(path) => VOTableIterator::from_file(path).and_then(|vot| vot.read_all_skipping_data()),
        None => {
          let stdin = stdin();
          let handle = stdin.lock();
          VOTableIterator::from_reader(handle).and_then(|vot| vot.read_all_skipping_data())
        }
      }
      .and_then(|vot| self.action.exec(vot))
    }
  }
}

#[derive(Debug, Subcommand)]
enum GetAction {
  /// Print the VOTable structure (useful to get Virtual IDs)
  Struct {
    /// Output line width (min=80)
    #[arg(short = 'w', long = "line-width", default_value = "120")]
    line_width: usize,
    /// Smaller possible size of 'content=xxx' when trying to fit on a single line.
    /// If larger, put on a new line (max=50% of line width)
    #[arg(short = 'c', long = "content-size-min", default_value = "30")]
    content_size_min: usize,
  },
  /// Print column names, one separated values line per table.
  ///
  /// TIP to get the list of the columns of a single table as a column on Linux:
  ///   vot -i myvot.xml get -s colnames | tr '▮' '\n' | egrep -v '^$'
  /// or (if no ',' in any names):
  ///   vot -i myvot.xml get -s colnames -s , | tr ',' '\n'
  #[command(verbatim_doc_comment)]
  Colnames {
    /// Separator use between each column names.
    #[arg(short, long, default_value_t = '▮')]
    separator: char,
  },
  /// Print selected field information as an array
  FieldsArray {
    /// Coma separated list of columns we want in the array of fields attributes
    #[arg(value_enum, value_delimiter(','))]
    fields: Vec<FieldElem>,
    /// Separator use between each column
    #[arg(short, long, default_value_t = ' ')]
    separator: char,
    /// Do not requires columns to be aligned
    #[arg(short, long)]
    not_aligned: bool,
  },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FieldElem {
  Index = 0,
  Id,
  Name,
  Datatype,
  Arraysize,
  Width,
  Precision,
  Unit,
  Ucd,
  Null,
  Min,
  Max,
  Link,
  Description,
}
const FIELDS: [FieldElem; FieldElem::len()] = [
  FieldElem::Index,
  FieldElem::Id,
  FieldElem::Name,
  FieldElem::Datatype,
  FieldElem::Arraysize,
  FieldElem::Width,
  FieldElem::Precision,
  FieldElem::Unit,
  FieldElem::Ucd,
  FieldElem::Null,
  FieldElem::Min,
  FieldElem::Max,
  FieldElem::Link,
  FieldElem::Description,
];
impl FieldElem {
  pub const fn index(&self) -> usize {
    *self as usize
  }
  pub const fn len() -> usize {
    Self::Description as usize + 1
  }
  pub const fn array() -> [FieldElem; FieldElem::len()] {
    FIELDS
  }
  pub fn iterator() -> Iter<'static, FieldElem> {
    FIELDS.iter()
  }
  pub const fn default_alignment(&self) -> Alignment {
    match &self {
      Self::Index => Alignment::Right,
      Self::Id => Alignment::Right,
      Self::Name => Alignment::Right,
      Self::Datatype => Alignment::Right,
      Self::Arraysize => Alignment::Right,
      Self::Width => Alignment::Right,
      Self::Precision => Alignment::Right,
      Self::Unit => Alignment::Center,
      Self::Ucd => Alignment::Left,
      Self::Null => Alignment::Right,
      Self::Min => Alignment::Right,
      Self::Max => Alignment::Right,
      Self::Link => Alignment::Left,
      Self::Description => Alignment::Left,
    }
  }
  pub const fn label(&self) -> &str {
    match &self {
      Self::Index => "i",
      Self::Id => "ID",
      Self::Name => "name",
      Self::Datatype => "dt",
      Self::Arraysize => "a",
      Self::Width => "w",
      Self::Precision => "p",
      Self::Unit => "unit",
      Self::Ucd => "ucd",
      Self::Null => "null",
      Self::Min => "min",
      Self::Max => "max",
      Self::Link => "link",
      Self::Description => "desc",
    }
  }
}

impl GetAction {
  fn exec<C>(&self, mut vot: VOTable<C>) -> Result<(), VOTableError>
  where
    C: TableDataContent,
  {
    match &self {
      Self::Struct {
        line_width,
        content_size_min,
      } => {
        let line_with = 80_usize.max(*line_width);
        let csm = (line_width >> 1).min(*content_size_min);
        let mut visitor = AsciiStructVisitor::new(line_with, csm);
        vot
          .visit(&mut visitor)
          .map_err(|e| VOTableError::Custom(e.to_string()))
      }
      Self::Colnames { separator } => {
        let mut visitor = ColnamesVisitor::new(*separator);
        vot
          .visit(&mut visitor)
          .map_err(|e| VOTableError::Custom(e.to_string()))
      }
      Self::FieldsArray {
        fields,
        separator,
        not_aligned,
      } => {
        let mut visitor = FieldArrayVisitor::new(*separator, fields.clone(), !not_aligned);
        vot
          .visit(&mut visitor)
          .map_err(|e| VOTableError::Custom(e.to_string()))
      }
    }
  }
}

#[cfg(test)]
mod tests {

  use super::FieldElem;

  #[test]
  fn test_field_enum() {
    assert_eq!(FieldElem::Index.index(), 0);
    assert_eq!(FieldElem::Id.index(), 1);
    assert_eq!(FieldElem::Name.index(), 2);
    assert_eq!(FieldElem::Datatype.index(), 3);
    assert_eq!(FieldElem::Arraysize.index(), 4);
    assert_eq!(FieldElem::Width.index(), 5);
    assert_eq!(FieldElem::Precision.index(), 6);
    assert_eq!(FieldElem::Unit.index(), 7);
    assert_eq!(FieldElem::Ucd.index(), 8);
    assert_eq!(FieldElem::Null.index(), 9);
    assert_eq!(FieldElem::Min.index(), 10);
    assert_eq!(FieldElem::Max.index(), 11);
    assert_eq!(FieldElem::Link.index(), 12);
    assert_eq!(FieldElem::Description.index(), 13);
    assert_eq!(FieldElem::len(), 14);
  }

  #[test]
  fn test_field_enum_iter() {
    for (i, field) in FieldElem::iterator().enumerate() {
      assert_eq!(i, field.index());
    }
  }
}
