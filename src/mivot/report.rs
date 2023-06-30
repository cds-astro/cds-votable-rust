use crate::{error::VOTableError, QuickXmlReadWrite};
use quick_xml::{
    events::{attributes::Attributes, BytesText, Event},
    Reader, Writer,
};
use std::str;

/*
    enum Status
    Description
    *    Enum of the status that can be applied to the <REPORT>.
*/
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Status {
    OK,
    FAILED,
    NULL,
}
impl Status {
    pub fn from_str(str: &str) -> Status {
        match str {
            "OK" => Self::OK,
            "FAILED" => Self::FAILED,
            _ => Self::NULL,
        }
    }
}
impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Self::OK => "OK".to_owned(),
            Self::FAILED => "FAILED".to_owned(),
            Self::NULL => "NULL".to_owned(),
        }
    }
}

/*
    struct Report
    @elem status Status: Status of the annotation process; must be either OK or FAILED, NULL is an error => MAND
    @elem content Option<String>: Other annotations => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Report {
    pub status: Status,
    // content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl Report {
    fn new_empty() -> Self {
        Report {
            status: Status::NULL,
            content: None,
        }
    }
}

impl QuickXmlReadWrite for Report {
    const TAG: &'static str = "REPORT";
    type Context = ();

    fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
        let mut report = Self::new_empty();
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let value = std::str::from_utf8(attr.value.as_ref()).map_err(VOTableError::Utf8)?;
            report = match attr.key {
                b"status" => {
                    if value.is_empty() {
                        report.status = Status::from_str(value);
                        report
                    } else {
                        return Err(VOTableError::Custom(format!(
                            "Attribute status is mandatory it musn't be empty"
                        )));
                    }
                }
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if report.status == Status::NULL {
            Err(VOTableError::Custom(format!(
                "Attribute status is mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(report)
        }
    }

    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        mut reader: Reader<R>,
        reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<Reader<R>, crate::error::VOTableError> {
        // I dot not like the fact that we first create an empty String that we replace here... :o/
        read_content!(Self, self, reader, reader_buff)
    }

    fn read_sub_elements_by_ref<R: std::io::BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        read_content_by_ref!(Self, self, reader, reader_buff)
    }

    fn write<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut elem_writer = writer.create_element(Self::TAG_BYTES);
        elem_writer = elem_writer.with_attribute(("status", self.status.to_string().as_str()));
        write_content!(self, elem_writer);
        Ok(())
    }
}
