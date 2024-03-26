//! Struct dedicated to the `DESCRIPTION` tag.

use std::fmt::{self, Display, Formatter};

use super::{
  error::VOTableError, utils::unexpected_attr_warn, HasContent, HasContentElem, VOTableElement,
};

/// Struct corresponding to the `DESCRIPTION` XML tag.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Description(String);

impl Description {
  pub fn new<I: Into<String>>(content: I) -> Self {
    Self::from(content.into())
  }
}

impl HasContent for Description {
  fn get_content(&self) -> Option<&str> {
    Some(self.0.as_str())
  }
  fn set_content<S: Into<String>>(mut self, content: S) -> Self {
    self.set_content_by_ref(content);
    self
  }
  fn set_content_by_ref<S: Into<String>>(&mut self, content: S) {
    self.0 = content.into()
  }
}

impl Description {
  pub fn get_content_unwrapped(&self) -> &str {
    self.0.as_str()
  }
}

impl From<&str> for Description {
  fn from(content: &str) -> Self {
    content.to_string().into()
  }
}

impl From<String> for Description {
  fn from(content: String) -> Self {
    Self(content)
  }
}

impl Display for Description {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str(self.get_content().unwrap_or(""))
  }
}

impl VOTableElement for Description {
  const TAG: &'static str = "DESCRIPTION";

  type MarkerType = HasContentElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    Self::new("").set_attrs(attrs)
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (k, _) in attrs {
      unexpected_attr_warn(k.as_ref(), Self::TAG);
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, _f: F)
  where
    F: FnMut(&str, &str),
  {
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{events::Event, Reader, Writer};

  use crate::{desc::Description, HasContent, QuickXmlReadWrite, VOTableElement};

  #[test]
  fn test_description_readwrite() {
    let xml = r#"<DESCRIPTION>
   VizieR Astronomical Server vizier.u-strasbg.fr
    Date: 2022-04-19T13:38:24 [V1.99+ (14-Oct-2013)]
   Explanations and Statistics of UCDs:			See LINK below
   In case of problem, please report to:	cds-question@unistra.fr
   In this version, NULL integer columns are written as an empty string
   &lt;TD&gt;&lt;/TD&gt;, explicitely possible from VOTable-1.3</DESCRIPTION>"#;
    // Test read
    let mut reader = Reader::from_reader(Cursor::new(xml.as_bytes()));
    let mut buff: Vec<u8> = Vec::with_capacity(xml.len());
    let mut description = loop {
      let mut event = reader.read_event(&mut buff).unwrap();
      match &mut event {
        Event::Start(ref mut e) if e.local_name() == Description::TAG_BYTES => {
          let desc = Description::from_event_start(&e)
            .and_then(|desc| desc.read_content(&mut reader, &mut buff, &()))
            .unwrap();
          assert_eq!(
            desc.get_content(),
            Some(
              r#"
   VizieR Astronomical Server vizier.u-strasbg.fr
    Date: 2022-04-19T13:38:24 [V1.99+ (14-Oct-2013)]
   Explanations and Statistics of UCDs:			See LINK below
   In case of problem, please report to:	cds-question@unistra.fr
   In this version, NULL integer columns are written as an empty string
   <TD></TD>, explicitely possible from VOTable-1.3"#
            )
          );
          break desc;
        }
        Event::Text(ref mut e) if e.escaped().is_empty() => (), // First even read
        _ => unreachable!(),
      }
    };
    // Test write
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    description.write(&mut writer, &()).unwrap();
    let output = writer.into_inner().into_inner();
    let output_str = unsafe { std::str::from_utf8_unchecked(output.as_slice()) };
    assert_eq!(output_str, xml);
  }
}
