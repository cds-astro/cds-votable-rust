
use std::{
  fs::File,
  path::Path,
  io::{BufRead, BufReader}
};

use quick_xml::Reader;

use crate::{
  table::{Table, TableElem},
  error::VOTableError,
  data::{
    TableOrBinOrBin2, 
    stream::Stream, binary::Binary, binary2::Binary2
  },
  votable::{VOTable, VOTableWrapper},
  resource::{Resource, ResourceOrTable},
  impls::{Schema, VOTableValue, mem::VoidTableDataContent},
  iter::elems::{Binary2RowValueIterator, BinaryRowValueIterator},
};

pub mod elems;
pub mod strings;

use elems::DataTableRowValueIterator;
// use strings::RowStringIterator;

// Idee:
// Iterator de Tables, iterator give access to
// - current talble
// - current resource
// - current votable
// - next table
//   => iterator on rows (indep of TableData/Binary/Binary2)

pub trait TableIter: Iterator<Item=Result<Vec<VOTableValue>, VOTableError>> {
  fn table(&mut self) -> &mut Table<VoidTableDataContent>; 
}

pub struct VOTableIterator<R: BufRead> {
  reader: Reader<R>, 
  reader_buff: Vec<u8>,
  // fn next_table -> (&votable, &resource, &table, &rows, Enum) 
  // 
  votable: VOTable<VoidTableDataContent>,
  resource_stack: Vec<Resource<VoidTableDataContent>>,
}

impl VOTableIterator<BufReader<File>> {
  
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<VOTableIterator<BufReader<File>>, VOTableError> {
    let mut reader_buff: Vec<u8> = Vec::with_capacity(1024);
    let (votable, resource, reader) = VOTableWrapper::<VoidTableDataContent>::manual_from_ivoa_xml_file(path, &mut reader_buff)?;
    let mut resource_stack = Vec::default();
    resource_stack.push(resource);
    Ok(VOTableIterator::<BufReader<File>> {
      reader,
      reader_buff,
      votable,
      resource_stack,
    })
  }
  
  pub fn end_of_it(self) -> VOTable<VoidTableDataContent> {
    self.votable
  }
}

impl<R: BufRead> VOTableIterator<R> {
  
  /*pub fn next_table_row_string_iter<'a>(&'a mut self) -> Result<Option<RowStringIterator<'a, R>>, VOTableError> {
    loop {
      if let Some(mut resource) = self.resource_stack.pop() {
        match resource.read_till_next_resource_or_table_by_ref(&mut self.reader, &mut self.reader_buff)? {
          Some(ResourceOrTable::<_>::Resource(sub_resource)) => {
            self.resource_stack.push(resource);
            self.resource_stack.push(sub_resource);
          },
          Some(ResourceOrTable::<_>::Table(mut table)) => {
            if let Some(mut data) = table.read_till_data_by_ref(&mut self.reader, &mut self.reader_buff)? {
              match data.read_till_table_bin_or_bin2_or_fits_by_ref(&mut self.reader, &mut self.reader_buff)? {
                Some(TableOrBinOrBin2::TableData) => {
                  table.set_data_by_ref(data);
                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = RowStringIterator::new(
                    &mut self.reader,
                    &mut self.reader_buff,
                    // self.resource_stack.last_mut().unwrap().tables.last_mut().unwrap()
                  );
                  return Ok(Some(row_it));
                },
                Some(TableOrBinOrBin2::Binary) => {
                  let _stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  todo!()
                },
                Some(TableOrBinOrBin2::Binary2) => {
                  let _stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  todo!()
                },
                Some(TableOrBinOrBin2::Fits(_fits)) => {
                  todo!()
                },
                None => {
                  todo!()
                },
              }
            } else {
              resource.push_table_by_ref(table);
              self.resource_stack.push(resource);
            }
          },
          None => self.votable.push_resource_by_ref(resource),
        }
      } else {
        match self.votable.read_till_next_resource_by_ref(&mut self.reader, &mut self.reader_buff)? {
          Some(resource) => self.resource_stack.push(resource),
          None => return Ok(None),
        }
      }
    }
  }*/
  
  pub fn next_table_row_value_iter<'a>(&'a mut self) -> Result<Option<Box<dyn 'a + TableIter>>, VOTableError> {
    loop {
      if let Some(mut resource) = self.resource_stack.pop() {
        match resource.read_till_next_resource_or_table_by_ref(&mut self.reader, &mut self.reader_buff)? {
          Some(ResourceOrTable::<_>::Resource(sub_resource)) => {
            self.resource_stack.push(resource);
            self.resource_stack.push(sub_resource);
          },
          Some(ResourceOrTable::<_>::Table(mut table)) => {
            if let Some(mut data) = table.read_till_data_by_ref(&mut self.reader, &mut self.reader_buff)? {
              match data.read_till_table_bin_or_bin2_or_fits_by_ref(&mut self.reader, &mut self.reader_buff)? {
                Some(TableOrBinOrBin2::TableData) => {
                  table.set_data_by_ref(data);

                  let schema: Vec<Schema> = table.elems.iter()
                    .filter_map(|table_elem|
                      match table_elem {
                        TableElem::Field(field) =>  Some(field.into()),
                        _ => None
                      }
                    ).collect();

                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = DataTableRowValueIterator::new(
                    &mut self.reader,
                    &mut self.reader_buff,
                    self.resource_stack.last_mut().unwrap().tables.last_mut().unwrap(),
                    schema,
                  );
                  return Ok(Some(Box::new(row_it)));
                },
                Some(TableOrBinOrBin2::Binary) => {
                  let stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  let binary = Binary::from_stream(stream);
                  data.set_binary_by_ref(binary);
                  table.set_data_by_ref(data);

                  let schema: Vec<Schema> = table.elems.iter()
                    .filter_map(|table_elem|
                      match table_elem {
                        TableElem::Field(field) =>  Some(field.into()),
                        _ => None
                      }
                    ).collect();

                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = BinaryRowValueIterator::new(
                    &mut self.reader,
                    self.resource_stack.last_mut().unwrap().tables.last_mut().unwrap(),
                    schema,
                  );
                  return Ok(Some(Box::new(row_it)));
                },
                Some(TableOrBinOrBin2::Binary2) => {
                  let stream = Stream::open_stream(&mut self.reader, &mut self.reader_buff)?;
                  let binary2 = Binary2::from_stream(stream);
                  data.set_binary2_by_ref(binary2);
                  table.set_data_by_ref(data);

                  let schema: Vec<Schema> = table.elems.iter()
                    .filter_map(|table_elem|
                      match table_elem {
                        TableElem::Field(field) =>  Some(field.into()),
                        _ => None
                      }
                    ).collect();

                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                  let row_it = Binary2RowValueIterator::new(
                    &mut self.reader,
                    self.resource_stack.last_mut().unwrap().tables.last_mut().unwrap(),
                    schema,
                  );
                  return Ok(Some(Box::new(row_it)));
                },
                Some(TableOrBinOrBin2::Fits(fits)) => {
                  data.set_fits_by_ref(fits);
                  table.set_data_by_ref(data);
                  resource.push_table_by_ref(table);
                  self.resource_stack.push(resource);
                },
                None => {
                  return Err(VOTableError::Custom(String::from("Unexpected empty DATA")));
                },
              }
            } else {
              resource.push_table_by_ref(table);
              self.resource_stack.push(resource);
            }
          },
          None => self.votable.push_resource_by_ref(resource),
        }
      } else {
        match self.votable.read_till_next_resource_by_ref(&mut self.reader, &mut self.reader_buff)? {
          Some(resource) => self.resource_stack.push(resource),
          None => return Ok(None),
        }
      }
    }
  }
}
