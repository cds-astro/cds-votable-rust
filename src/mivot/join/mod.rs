//! Module dedicated to the `JOIN` tag.
//!
//! `JOIN` is the same for either `GLOBALS` `COLLECTION`s or `TEMPLATE` `COLLECTION`s.

use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{events::Event, Reader, Writer};

use crate::{
  error::VOTableError,
  mivot::VodmlVisitor,
  utils::{discard_comment, discard_event, is_empty, unexpected_attr_err},
  HasSubElements, HasSubElems, QuickXmlReadWrite, VOTableElement,
};

pub mod r#where;
use r#where::Where;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "attr_type")]
pub enum JoinAttributes {
  DmRef { dmref: String },
  SrcRef { sourceref: String },
  BothRef { dmref: String, sourceref: String },
}
impl JoinAttributes {
  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut new_dmref = String::new();
    let mut new_sourceref = String::new();
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "dmref" => new_dmref = val.into(),
        "sourceref" => new_sourceref = val.into(),
        _ => return Err(unexpected_attr_err(key, Join::TAG)),
      }
    }
    match self {
      JoinAttributes::DmRef { dmref } => {
        *dmref = new_dmref;
        if new_sourceref.is_empty() {
          return Err(VOTableError::Custom("Unable to set 'sourceref'".into()));
        }
      }
      JoinAttributes::SrcRef { sourceref } => {
        *sourceref = new_sourceref;
        if new_dmref.is_empty() {
          return Err(VOTableError::Custom("Unable to set 'dmref'".into()));
        }
      }
      JoinAttributes::BothRef { dmref, sourceref } => {
        *dmref = new_dmref;
        *sourceref = new_sourceref;
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    match &self {
      JoinAttributes::DmRef { dmref } => f("dmtype", dmref.as_str()),
      JoinAttributes::SrcRef { sourceref } => f("sourceref", sourceref.as_str()),
      JoinAttributes::BothRef { dmref, sourceref } => {
        f("dmtype", dmref.as_str());
        f("sourceref", sourceref.as_str());
      }
    }
  }
}

/// In`TEMPLATES`, `JOIN` populates a `COLLECTION` with `INSTANCE` elements resulting from the
/// iteration overs a `TEMPLATES`.
/// * If at least one `where`, `sourceref` is mandatory because it is a join with another template
///   referenced by the sourceref.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Join {
  pub attr: JoinAttributes,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub wheres: Vec<Where>,
}

impl Join {
  pub fn from_dmref<S: Into<String>>(dmref: S) -> Self {
    Join {
      attr: JoinAttributes::DmRef {
        dmref: dmref.into(),
      },
      wheres: Default::default(),
    }
  }

  pub fn from_sourceref<S: Into<String>>(sourceref: S) -> Self {
    Join {
      attr: JoinAttributes::SrcRef {
        sourceref: sourceref.into(),
      },
      wheres: Default::default(),
    }
  }

  pub fn from_both_ref<S: Into<String>>(dmref: S, sourceref: S) -> Self {
    Join {
      attr: JoinAttributes::BothRef {
        dmref: dmref.into(),
        sourceref: sourceref.into(),
      },
      wheres: Default::default(),
    }
  }

  pub fn push_where(mut self, r#where: Where) -> Self {
    self.wheres.push(r#where);
    self
  }

  pub fn push_where_by_ref(&mut self, r#where: Where) {
    self.wheres.push(r#where);
  }

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_join_start(self)?;
    for w in self.wheres.iter_mut() {
      w.visit(visitor)?;
    }
    visitor.visit_join_ended(self)
  }
}

impl VOTableElement for Join {
  const TAG: &'static str = "JOIN";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut dmref = String::new();
    let mut sourceref = String::new();
    for (key, val) in attrs {
      let key = key.as_ref();
      let val = val.as_ref();
      if !val.is_empty() {
        match key {
          "dmref" => dmref.push_str(val),
          "sourceref" => sourceref.push_str(val),
          _ => return Err(unexpected_attr_err(key, Self::TAG)),
        }
      }
    }
    match (dmref.is_empty(), sourceref.is_empty()) {
      (false, false) => Ok(Self::from_both_ref(dmref, sourceref)),
      (false, true) => Ok(Self::from_dmref(dmref)),
      (true, false) => Ok(Self::from_sourceref(sourceref)),
      (true, true) => Err(VOTableError::Custom(format!(
        "One of the attribute 'dmref' or 'sourceref' mandatory in {}.",
        Self::TAG
      ))),
    }
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    self.attr.set_attrs_by_ref(attrs)
  }

  fn for_each_attribute<F>(&self, f: F)
  where
    F: FnMut(&str, &str),
  {
    self.attr.for_each_attribute(f)
  }
}

impl HasSubElements for Join {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.wheres.is_empty()
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Where::TAG_BYTES => {
            self.push_where_by_ref(from_event_start_by_ref!(Where, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Where::TAG_BYTES => self.push_where_by_ref(Where::from_event_empty(e)?),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => {
          return Ok(());
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem_vec!(self, wheres, writer, context);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{mivot::test::get_xml, tests::test_read};

  use super::Join;

  #[test]
  fn test_join_read() {
    // OK JOINS
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.1.xml");
    println!("testing 9.1");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.4.xml");
    println!("testing 9.4");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.5.xml");
    println!("testing 9.5");
    test_read::<Join>(&xml);

    // TODO: Wait for the doc to be clarified (PK or FK mandatory?)
    /*
    println!("testing 9.5");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.6.xml");
    println!("testing 9.6");
    test_read::<Join>(&xml);
    let xml = get_xml("./resources/mivot/9/test_9_ok_9.9.xml");
    println!("testing 9.9");
    test_read::<Join>(&xml);

    // KO JOINS
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.2.xml");
    println!("testing 9.2"); // no dmref + sourceref must come with a dmref
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.3.xml");
    println!("testing 9.3"); // must have dmref or sourceref
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.3.xml");
    println!("testing 9.3"); // must have dmref or sourceref
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.7.xml");
    println!("testing 9.7"); // dmref must not be empty
    test_error::<Join>(&xml, false);
    let xml = get_xml("./resources/mivot/9/test_9_ko_9.8.xml");
    println!("testing 9.8"); // sourceref must not be empty
    test_error::<Join>(&xml, false);
    */
  }
}
