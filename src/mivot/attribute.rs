use crate::{error::VOTableError, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Attribute {
    dmrole: String,
    dmtype: String,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    array_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}
impl Attribute {
    fn new<N: Into<String>>(dmrole: N, dmtype: N) -> Self {
        Self {
            dmrole: dmrole.into(),
            dmtype: dmtype.into(),
            value: None,
            ref_: None,
            array_index: None,
            unit: None,
        }
    }
    impl_builder_opt_string_attr!(ref_);
    impl_builder_opt_string_attr!(value);
    impl_builder_opt_string_attr!(array_index);
    impl_builder_opt_string_attr!(unit);
}

impl QuickXmlReadWrite for Attribute {
    const TAG: &'static str = "ATTRIBUTE";
    type Context = ();

    fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
        const NULL: &str = "@TBD";
        let mut attribute = Self::new(NULL, NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            attribute = match attr.key {
                b"dmrole" => {
                    attribute.dmrole = value.to_string();
                    attribute
                }
                b"dmtype" => {
                    attribute.dmtype = value.to_string();
                    attribute
                }
                b"value" => attribute.set_value(value),
                b"arrayindex" => attribute.set_array_index(value),
                b"ref" => attribute.set_ref_(value),
                b"unit" => attribute.set_unit(value),
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if attribute.dmrole.as_str() == NULL || attribute.dmtype.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attributes 'dmrole' and 'dmtype' are mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(attribute)
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

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut elem_writer = writer.create_element(Self::TAG_BYTES);
        elem_writer = elem_writer.with_attribute(("dmrole", self.dmrole.as_str()));
        elem_writer = elem_writer.with_attribute(("dmtype", self.dmtype.as_str()));
        write_opt_string_attr!(self, elem_writer, ref_, "ref");
        write_opt_string_attr!(self, elem_writer, ref_, "value");
        write_opt_string_attr!(self, elem_writer, array_index, "arrayindex");
        write_opt_string_attr!(self, elem_writer, unit, "unit");
        elem_writer.write_empty().map_err(VOTableError::Write)?;
        Ok(())
    }
}

////////////////////////////////////////////////

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionAttribute {
    dmtype: String,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    array_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}
impl CollectionAttribute {
    fn new<N: Into<String>>(dmtype: N) -> Self {
        Self {
            dmtype: dmtype.into(),
            ref_: None,
            value: None,
            array_index: None,
            unit: None,
        }
    }
    impl_builder_opt_string_attr!(ref_);
    impl_builder_opt_string_attr!(value);
    impl_builder_opt_string_attr!(array_index);
    impl_builder_opt_string_attr!(unit);
}

impl QuickXmlReadWrite for CollectionAttribute {
    const TAG: &'static str = "ATTRIBUTE";
    type Context = ();

    fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
        const NULL: &str = "@TBD";
        let mut attribute = Self::new(NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            attribute = match attr.key {
                b"dmtype" => {
                    attribute.dmtype = value.to_string();
                    attribute
                }
                b"ref" => attribute.set_ref_(value),
                b"value" => attribute.set_value(value),
                b"arrayindex" => attribute.set_array_index(value),
                b"unit" => attribute.set_unit(value),
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if attribute.dmtype.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attribute 'dmtype' is mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(attribute)
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

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut elem_writer = writer.create_element(Self::TAG_BYTES);
        elem_writer = elem_writer.with_attribute(("dmtype", self.dmtype.as_str()));
        write_opt_string_attr!(self, elem_writer, ref_, "ref");
        write_opt_string_attr!(self, elem_writer, value, "value");
        write_opt_string_attr!(self, elem_writer, array_index, "arrayindex");
        write_opt_string_attr!(self, elem_writer, unit, "unit");
        elem_writer.write_empty().map_err(VOTableError::Write)?;
        Ok(())
    }
}
