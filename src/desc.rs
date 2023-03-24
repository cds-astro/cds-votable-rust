use std::io::{BufRead, Write};

use quick_xml::{Reader, Writer, events::{BytesText, Event, attributes::Attributes}};

use super::{
  QuickXmlReadWrite,
  error::VOTableError
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Description(pub String);

impl From<&str> for Description {
  fn from(s: &str) -> Self {
    s.to_string().into()
  }
}

impl From<String> for Description {
  fn from(s: String) -> Self {
    Description(s)
  }
}

impl QuickXmlReadWrite for Description {

  const TAG: &'static str = "DESCRIPTION";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    if attrs.count() > 0 {
     eprintln!("Unexpected attributes in DESCRIPTION (not serialized!)");
    }
    Ok(Description(Default::default()))
  }

  fn read_sub_elements<R: BufRead>(
    &mut self, 
    mut reader: Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context
  ) -> Result<Reader<R>, VOTableError> {
    // I dot not like the fact that we first create an empty String that we replace here... :o/
    read_content!(Self, self, reader, reader_buff, 0)
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    reader: &mut Reader<R>,
    reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    read_content_by_ref!(Self, self, reader, reader_buff, 0)
  }

  fn write<W: Write>(&mut self, writer: &mut Writer<W>, _context: &Self::Context) -> Result<(), VOTableError> {
    let elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer.write_text_content(
      BytesText::from_plain_str(self.0.as_str())
    ).map_err(VOTableError::Write)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::Cursor;

  use quick_xml::{Reader, events::Event, Writer};
  use crate::{
    QuickXmlReadWrite,
    desc::Description
  };

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
          let mut desc = Description::from_attributes(e.attributes()).unwrap();
          desc.read_sub_elements_and_clean(reader, &mut buff, &()).unwrap();
          assert_eq!(
            desc.0,
            r#"
   VizieR Astronomical Server vizier.u-strasbg.fr
    Date: 2022-04-19T13:38:24 [V1.99+ (14-Oct-2013)]
   Explanations and Statistics of UCDs:			See LINK below
   In case of problem, please report to:	cds-question@unistra.fr
   In this version, NULL integer columns are written as an empty string
   <TD></TD>, explicitely possible from VOTable-1.3"#);
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
