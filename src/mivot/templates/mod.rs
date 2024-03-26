//! Module dedicated to the `TEPLATES` tag.
//!
//! The `TEMPLATES` block maps data model instances on the rows of a table in the VOTable.

use std::{io::Write, str};

use paste::paste;
use quick_xml::{events::Event, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::VodmlVisitor,
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_err},
  HasSubElements, HasSubElems, QuickXmlReadWrite, VOTableElement,
};

pub mod instance;
use instance::Instance;
pub mod r#where;
use r#where::Where;

/// Structure storing the content of the `TEMPLATES` tag.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Templates {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tableref: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub wheres: Vec<Where>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub instances: Vec<Instance>,
}
impl Templates {
  fn new() -> Self {
    Self {
      tableref: None,
      wheres: vec![],
      instances: vec![],
    }
  }

  impl_builder_opt_string_attr!(tableref);

  pub fn push_where(mut self, where_: Where) -> Self {
    self.push_where_by_ref(where_);
    self
  }
  pub fn push_where_by_ref(&mut self, where_: Where) {
    self.wheres.push(where_);
  }
  impl_builder_push!(Instance);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_templates_start(self)?;
    for w in self.wheres.iter_mut() {
      w.visit(visitor)?;
    }
    for elem in self.instances.iter_mut() {
      elem.visit(visitor)?;
    }
    visitor.visit_templates_ended(self)
  }
}

impl VOTableElement for Templates {
  const TAG: &'static str = "TEMPLATES";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new().set_attrs(attrs)
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "tableref" => {
          if val.as_ref().is_empty() {
            return Err(VOTableError::Custom(String::from(
              "Attribute 'tableref' must not be empty.",
            )));
          } else {
            self.set_tableref_by_ref(val)
          }
        }
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(tableref) = &self.tableref {
      f("tableref", tableref.as_ref());
    }
  }
}

impl HasSubElements for Templates {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    // Note: Should always be true to be a valid VODML element
    self.wheres.is_empty() && self.instances.is_empty()
  }

  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Instance::TAG_BYTES => push_from_event_start!(self, Instance, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Templates::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Where::TAG_BYTES => push_from_event_empty!(self, Where, e),
          Instance::TAG_BYTES => push_from_event_empty!(self, Instance, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Templates::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Templates::TAG_BYTES => {
          if self.instances.is_empty() {
            return Err(VOTableError::Custom(
              "At least one instance should be present in a templates tag.".to_owned(),
            ));
          } else {
            return Ok(());
          }
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Templates::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Templates::TAG),
        _ => discard_event(event, Templates::TAG),
      }
    }
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec!(self, wheres, writer, context);
    write_elem_vec!(self, instances, writer, context);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::templates::Templates,
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  #[test]
  fn test_templates_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/4/test_4_ok_4.1.xml");
    println!("testing 4.1");
    test_read::<Templates>(&xml);
    let xml = get_xml("./resources/mivot/4/test_4_ok_4.2.xml");
    println!("testing 4.2");
    test_read::<Templates>(&xml);
    let xml = get_xml("./resources/mivot/4/test_4_ok_4.5.xml");
    println!("testing 4.5");
    test_read::<Templates>(&xml);
    let xml = get_xml("./resources/mivot/4/test_4_ok_4.6.xml");
    println!("testing 4.6");
    test_read::<Templates>(&xml);
    let xml = get_xml("./resources/mivot/4/test_4_ok_4.8.xml");
    println!("testing 4.8");
    test_read::<Templates>(&xml);
    // KO MODELS
    let xml = get_xml("./resources/mivot/4/test_4_ko_4.3.xml");
    println!("testing 4.3"); // WHERE only; INSTANCE required
    test_error::<Templates>(&xml, false);
    let xml = get_xml("./resources/mivot/4/test_4_ko_4.4.xml");
    println!("testing 4.4"); // Where should be before instance (parser can overlook this and write it correctly later)
    test_read::<Templates>(&xml); // Should read correctly
    let xml = get_xml("./resources/mivot/4/test_4_ko_4.7.xml");
    println!("testing 4.7"); // includes invalid subnode
    test_error::<Templates>(&xml, false);
    let xml = get_xml("./resources/mivot/4/test_4_ko_4.9.xml");
    println!("testing 4.9"); // tableref must not be empty
    test_error::<Templates>(&xml, true);
  }
}
