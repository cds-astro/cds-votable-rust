//! Module dedicated to the `JOIN` tag.
//!
//! `JOIN` is the same for either `GLOBALS` `COLLECTION`s or `TEMPLATE` `COLLECTION`s.

use std::{
  io::{BufRead, Write},
  str,
};

use quick_xml::{
  events::{attributes::Attributes, BytesStart, Event},
  Reader, Writer,
};

use crate::{
  error::VOTableError,
  mivot::VodmlVisitor,
  utils::{discard_comment, discard_event, is_empty},
  QuickXmlReadWrite,
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
    visitor.visit_join(self)?;
    for w in self.wheres.iter_mut() {
      w.visit(visitor)?;
    }
    Ok(())
  }
}

impl QuickXmlReadWrite for Join {
  const TAG: &'static str = "JOIN";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut dmref = String::new();
    let mut sourceref = String::new();
    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !value.is_empty() {
        match attr.key {
          b"dmref" => dmref.push_str(value),
          b"sourceref" => sourceref.push_str(value),
          _ => return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG)),
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

  fn read_sub_elements<R: BufRead>(
    &mut self,
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, VOTableError> {
    self
      .read_sub_elements_by_ref(&mut reader, reader_buff, _context)
      .map(|()| reader)
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
            self
              .wheres
              .push(from_event_start_by_ref!(Where, reader, reader_buff, e))
          }
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Where::TAG_BYTES => self.wheres.push(Where::from_event_empty(e)?),
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

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
    // Write tag + attributes
    match &self.attr {
      JoinAttributes::DmRef { dmref } => tag.push_attribute(("dmtype", dmref.as_str())),
      JoinAttributes::SrcRef { sourceref } => tag.push_attribute(("sourceref", sourceref.as_str())),
      JoinAttributes::BothRef { dmref, sourceref } => {
        tag.push_attribute(("dmtype", dmref.as_str()));
        tag.push_attribute(("sourceref", sourceref.as_str()));
      }
    }
    writer
      .write_event(Event::Start(tag.to_borrowed()))
      .map_err(VOTableError::Write)?;
    // Write sub-elements
    write_elem_vec!(self, wheres, writer, context);
    // Close tag
    writer
      .write_event(Event::End(tag.to_end()))
      .map_err(VOTableError::Write)
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
