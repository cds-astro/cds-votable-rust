extern crate core;

use crate::error::VOTableError;
use quick_xml::Writer;
use std::io::Write;

use self::{foreignkey::ForeignKey, primarykey::PrimaryKey};

#[macro_use]
pub mod macros;

pub mod attribute_a;
pub mod attribute_b;
pub mod attribute_c;
pub mod collection;
pub mod foreignkey;
pub mod globals;
pub mod instance;
pub mod join;
pub mod model;
pub mod primarykey;
pub mod reference;
pub mod report;
pub mod templates;
pub mod vodml;
pub mod r#where;

pub trait ElemType {
  fn write<W: Write>(&mut self, writer: &mut Writer<W>) -> Result<(), VOTableError>;
}
pub trait ElemImpl<T: ElemType> {
  fn push_to_elems(&mut self, elem: T);
}

pub trait InstanceType {
  fn push2_pk(&mut self, pk: PrimaryKey);
}

pub trait ReferenceType {
  fn push2_fk(&mut self, fk: ForeignKey);
}

#[cfg(test)]
mod test {
  use crate::QuickXmlReadWrite;
  use quick_xml::{events::Event, Reader};
  use std::{
    fs::File,
    io::{Cursor, Read},
  };

  pub(crate) fn test_error<X: QuickXmlReadWrite<Context = ()>>(xml: &str, special_cond: bool) {
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == X::TAG_BYTES => {
          if !special_cond {
            let mut info = X::from_attributes(e.attributes()).unwrap();
            assert!(info
              .read_sub_elements_and_clean(reader.clone(), &mut buff, &())
              .is_err());
          } else {
            assert!(X::from_attributes(e.attributes()).is_err())
          }
          break;
        }
        Event::Empty(ref mut e) if e.local_name() == X::TAG_BYTES => {
          if special_cond {
            let mut info = X::from_attributes(e.attributes()).unwrap();
            assert!(info
              .read_sub_elements_and_clean(reader.clone(), &mut buff, &())
              .is_err());
          } else {
            assert!(X::from_attributes(e.attributes()).is_err())
          };
          break;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        Event::Comment(_) => (),
        Event::DocType(_) => (),
        Event::Decl(_) => (),
        _ => {
          println!("{:?}", event);
          unreachable!()
        }
      }
    }
  }

  pub(crate) fn get_xml(path: &str) -> String {
    let mut file = File::open(path).expect("Unable to open the file");
    let mut xml = String::new();
    file
      .read_to_string(&mut xml)
      .expect("Unable to read the file");
    xml.replace(&['\n', '\t'][..], "")
  }
}
