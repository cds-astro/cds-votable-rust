use crate::{error::VOTableError, QuickXmlReadWrite};
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct Attribute => pattern b generic attribute
    @elem dmrole String: Modeled node related => MAND
    @elem dmtype String: Modeled node related => MAND
    @elem value Option<String>: attribute default value => MAND
    @elem array_index Option<String>: attribute size of array, only present if the attribute is an array (example: String = char array) => OPT
    @elem unit Option<String>: the unit used for the value (example: km/s) => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributePatB {
  // MANDATORY
  dmrole: String,
  dmtype: String,
  value: String,
  // OPTIONAL
  #[serde(skip_serializing_if = "Option::is_none")]
  array_index: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  unit: Option<String>,
}
impl AttributePatB {
  /*
      function New
      Description:
      *   creates a new Attribute
      @generic N: Into<String>; a struct implementing the Into<String> trait
      @param dmrole N: a placeholder for the MANDATORY dmrole
      @param dmtype N: a placeholder for the MANDATORY dmtype
      @param value N: a placeholder for the MANDATORY value
      #returns Self: returns an instance of the AttributePatB struct
  */
  fn new<N: Into<String>>(dmrole: N, dmtype: N, value: N) -> Self {
    Self {
      // MANDATORY
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      value: value.into(),
      // OPTIONAL
      array_index: None,
      unit: None,
    }
  }
  /*
      function setters, enable the setting of an optional through self.set_"var"
  */
  impl_builder_opt_string_attr!(array_index);
  impl_builder_opt_string_attr!(unit);
}

impl QuickXmlReadWrite for AttributePatB {
  // The TAG name here : <ATTRIBUTE>
  const TAG: &'static str = "ATTRIBUTE";
  // Potential context, here : ()
  type Context = ();

  /*
      function from_attributes
      Description:
      *   creates Self from deserialized attributes contained inside the passed XML
      @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
      #returns Result<Self, VOTableError>: returns an instance of AttributePatB built using attributes or an error if reading doesn't work
  */
  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    const NULL: &str = "@TBD";
    let mut attribute = Self::new(NULL, NULL, NULL);
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
        b"value" => {
          attribute.value = value.to_string();
          attribute
        }
        b"arrayindex" => attribute.set_array_index(value),
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

  /*
      function read_sub_elements
      ! NO SUBELEMENTS SHOULD BE PRESENT: UNREACHABLE
  */
  fn read_sub_elements<R: std::io::BufRead>(
    &mut self,
    mut _reader: Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<Reader<R>, crate::error::VOTableError> {
    unreachable!()
  }

  /*
      function read_sub_elements_by_ref
      ! NO SUBELEMENTS SHOULD BE PRESENT: UNREACHABLE
  */
  fn read_sub_elements_by_ref<R: std::io::BufRead>(
    &mut self,
    _reader: &mut Reader<R>,
    _reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    unreachable!()
  }

  /*
      function Write
      Description:
      *   function that writes the TAG
      @generic W: Write; a struct that implements the std::io::Write trait.
      @param self &mut: function is used like : self."function"
      @param writer &mut Writer<W>: the writer used to write the elements
      @param context &Self::Context: the context used for writing UNUSED
      #returns Result<(), VOTableError>: returns an error if writing doesn't work
  */
  fn write<W: std::io::Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), crate::error::VOTableError> {
    let mut elem_writer = writer.create_element(Self::TAG_BYTES);
    elem_writer = elem_writer.with_attribute(("dmrole", self.dmrole.as_str()));
    elem_writer = elem_writer.with_attribute(("dmtype", self.dmtype.as_str()));
    elem_writer = elem_writer.with_attribute(("value", self.value.as_str()));
    write_opt_string_attr!(self, elem_writer, array_index, "arrayindex");
    write_opt_string_attr!(self, elem_writer, unit, "unit");
    elem_writer.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}
