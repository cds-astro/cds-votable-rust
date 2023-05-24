use crate::{error::VOTableError, QuickXmlReadWrite};

use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::{io::Write, str};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PrimaryKey {
    dmtype: String,
    value: String,
}
impl PrimaryKey {
    fn new<N: Into<String>>(dmtype: N, value: N) -> Self {
        Self {
            dmtype: dmtype.into(),
            value: value.into(),
        }
    }
}

impl QuickXmlReadWrite for PrimaryKey {
    const TAG: &'static str = "PRIMARY_KEY";
    type Context = ();

    fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
        const NULL: &str = "@TBD";
        let mut primary_key = Self::new(NULL, NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            primary_key = match attr.key {
                b"dmtype" => {
                    primary_key.dmtype = value.to_string();
                    primary_key
                }
                b"value" => {
                    primary_key.value = value.to_string();
                    primary_key
                }
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if primary_key.dmtype.as_str() == NULL || primary_key.value.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attributes 'dmtype' and 'value' are mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(primary_key)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        mut _reader: Reader<R>,
        _reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<Reader<R>, crate::error::VOTableError> {
        unreachable!()
    }

    fn read_sub_elements_by_ref<R: std::io::BufRead>(
        &mut self,
        _reader: &mut Reader<R>,
        _reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        unreachable!()
    }

    fn write<W: Write>(
        &mut self,
        writer: &mut Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut elem_writer = writer.create_element(Self::TAG_BYTES);
        elem_writer = elem_writer.with_attribute(("dmtype", self.dmtype.as_str()));
        elem_writer = elem_writer.with_attribute(("value", self.dmtype.as_str()));
        elem_writer.write_empty().map_err(VOTableError::Write)?;
        Ok(())
    }
}
