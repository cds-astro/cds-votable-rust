use crate::{error::VOTableError, QuickXmlReadWrite};
use bstringify::bstringify;
use paste::paste;
use quick_xml::{events::attributes::Attributes, Reader, Writer};
use std::str;

/*
    struct Attribute => pattern a valid in Templates
    @elem dmrole String: Modeled node related => MAND
    @elem dmtype String: Modeled node related => MAND
    @elem ref Option<String>: reference to a VOTable element => OPT
    @elem value Option<String>: attribute default value => OPT
    @elem array_index Option<String>: attribute size of array, only present if the attribute is an array (example: String = char array) => OPT
    @elem unit Option<String>: the unit used for the value (example: km/s) => OPT
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AttributePatA {
    // MANDATORY
    dmrole: String,
    dmtype: String,
    // OPTIONAL
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    array_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}
impl AttributePatA {
    /*
        function New
        Description:
        *   creates a new Attribute
        @generic N: Into<String>; a struct implementing the Into<String> trait
        @param dmrole N: a placeholder for the MANDATORY dmrole
        @param dmtype N: a placeholder for the MANDATORY dmtype
        #returns Self: returns an instance of the AttributePatA struct
    */
    fn new<N: Into<String>>(dmrole: N, dmtype: N) -> Self {
        Self {
            // MANDATORY
            dmrole: dmrole.into(),
            dmtype: dmtype.into(),
            // OPTIONAL
            ref_: None,
            value: None,
            array_index: None,
            unit: None,
        }
    }
    /*
        function setters, enable the setting of an optional through self.set_"var"
    */
    impl_builder_opt_string_attr!(ref_);
    impl_builder_opt_string_attr!(value);
    impl_builder_opt_string_attr!(array_index);
    impl_builder_opt_string_attr!(unit);
}

impl QuickXmlReadWrite for AttributePatA {
    // The TAG name here : <ATTRIBUTE>
    const TAG: &'static str = "ATTRIBUTE";
    // Potential context, here : ()
    type Context = ();

    /*
        function from_attributes
        Description:
        *   creates Self from deserialized attributes contained inside the passed XML
        @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
        #returns Result<Self, VOTableError>: returns an instance of AttributePatA built using attributes or an error if reading doesn't work
    */
    impl_builder_from_attr!([dmrole, dmtype], [ref_, "ref", value, array_index, unit]);

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
        todo UNIMPLEMENTED
    */
    fn read_sub_elements_by_ref<R: std::io::BufRead>(
        &mut self,
        _reader: &mut Reader<R>,
        _reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        todo!()
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
        write_empty_mandatory_attributes!(elem_writer, self, dmrole, dmtype);
        write_empty_optional_attributes!(elem_writer, self, ref_, "ref", value, array_index, unit);
        elem_writer.write_empty().map_err(VOTableError::Write)?;
        Ok(())
    }
}
