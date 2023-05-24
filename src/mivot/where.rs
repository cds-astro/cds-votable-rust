use crate::{error::VOTableError, QuickXmlReadWrite};
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Where {
    primary_key: String,
    foreign_key: String,
}
impl Where {
    fn new<N: Into<String>>(primary_key: N, foreign_key: N) -> Self {
        Self {
            primary_key: primary_key.into(),
            foreign_key: foreign_key.into(),
        }
    }
}
impl QuickXmlReadWrite for Where {
    const TAG: &'static str = "WHERE";
    type Context = ();

    fn from_attributes(
        attrs: quick_xml::events::attributes::Attributes,
    ) -> Result<Self, crate::error::VOTableError> {
        const NULL: &str = "@TBD";
        let mut where_ = Self::new(NULL, NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            where_ = match attr.key {
                b"primary_key" => {
                    where_.primary_key = value.to_string();
                    where_
                }
                b"foreign_key" => {
                    where_.foreign_key = value.to_string();
                    where_
                }
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if where_.primary_key.as_str() == NULL || where_.foreign_key.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attributes 'primary_key' and 'foreign_key' are mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(where_)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        _reader: quick_xml::Reader<R>,
        _reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
        unreachable!()
    }

    fn read_sub_elements_by_ref<R: std::io::BufRead>(
        &mut self,
        _reader: &mut quick_xml::Reader<R>,
        _reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        todo!()
    }

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut quick_xml::Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut elem_writer = writer.create_element(Self::TAG_BYTES);
        elem_writer = elem_writer.with_attribute(("primary_key", self.primary_key.as_str()));
        elem_writer = elem_writer.with_attribute(("foreign_key", self.foreign_key.as_str()));
        elem_writer.write_empty().map_err(VOTableError::Write)?;
        Ok(())
    }
}
