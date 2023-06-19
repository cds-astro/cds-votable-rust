/////////////////////////
/////// PATTERN A ///////
/////////////////////////

/*
    struct Globals or templates instance => pattern a
    @elem dmtype String: Modeled node related => MAND
    @elem dmid Option<String>: Mapping element identification => OPT
    @elem primary_keys: identification key to an INSTANCE (at least one)
    @elem elems: different elems defined in enum InstanceElem that can appear in any order
*/
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobOrTempInstance {
    // MANDATORY
    dmtype: String,
    // OPTIONAL
    #[serde(skip_serializing_if = "Option::is_none")]
    dmid: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    primary_keys: Vec<PrimaryKey>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    elems: Vec<InstanceElem>,
}
impl GlobOrTempInstance {
    /*
        function New
        Description:
        *   creates a new Instance
        @generic N: Into<String>; a struct implementing the Into<String> trait
        @param dmtype N: a placeholder for the MANDATORY dmtype
        #returns Self: returns an instance of the GlobOrTempInstance struct
    */
    fn new<N: Into<String>>(dmtype: N) -> Self {
        Self {
            // MANDATORY
            dmtype: dmtype.into(),
            // OPTIONAL
            dmid: None,
            primary_keys: vec![],
            elems: vec![],
        }
    }
    /*
        function set_dmid
        Description:
        *   macro creating a function to set the optional string dmid
        @param dmid String: the dmid to set inside the struct
        #returns ()
    */
    impl_builder_opt_string_attr!(dmid);
}
impl ElemImpl<InstanceElem> for GlobOrTempInstance {
    /*
        function push_to_elems
        Description:
        *   pushes an InstanceElem to the elems contained in struct
        @param self &mut: function is used like : self."function"
        @param dmid InstanceElem: the elem that needs to be pushed
        #returns ()
    */
    fn push_to_elems(&mut self, elem: InstanceElem) {
        self.elems.push(elem)
    }
}
impl QuickXmlReadWrite for GlobOrTempInstance {
    // The TAG name here : <INSTANCE>
    const TAG: &'static str = "INSTANCE";
    // Potential context, here : ()
    type Context = ();

    /*
        function from_attributes
        Description:
        *   creates Self from deserialized attributes contained inside the passed XML
        @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
        #returns Result<Self, VOTableError>: returns an instance of GlobOrTempInstance built using attributes or an error if reading doesn't work
    */
    fn from_attributes(
        attrs: quick_xml::events::attributes::Attributes,
    ) -> Result<Self, VOTableError> {
        const NULL: &str = "@TBD";
        let mut instance = Self::new(NULL);
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            instance = match attr.key {
                b"dmid" => instance.set_dmid(value),
                b"dmtype" => {
                    instance.dmtype = value.to_string();
                    instance
                }
                b"dmrole" => instance, // * This is in case of an empty dmrole which shouldn't be taken into account
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        if instance.dmtype.as_str() == NULL {
            Err(VOTableError::Custom(format!(
                "Attribute 'dmtype' is mandatory in tag '{}'",
                Self::TAG
            )))
        } else {
            Ok(instance)
        }
    }

    /*
        function read_sub_elements
        Description:
        *   see function read_instance_sub_elem
    */
    fn read_sub_elements<R: std::io::BufRead>(
        &mut self,
        reader: quick_xml::Reader<R>,
        reader_buff: &mut Vec<u8>,
        _context: &Self::Context,
    ) -> Result<quick_xml::Reader<R>, crate::error::VOTableError> {
        read_instance_sub_elem(self, reader, reader_buff)
    }

    /*
        function read_sub_elements
        todo UNIMPLEMENTED
    */
    fn read_sub_elements_by_ref<R: std::io::BufRead>(
        &mut self,
        _reader: &mut quick_xml::Reader<R>,
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
        #returns Result<(), VOTableError>: returns an error if writing doesn't work
    */
    fn write<W: Write>(
        &mut self,
        writer: &mut quick_xml::Writer<W>,
        _context: &Self::Context,
    ) -> Result<(), crate::error::VOTableError> {
        let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
        //MANDATORY
        tag.push_attribute(("dmtype", self.dmtype.as_str()));
        //OPTIONAL
        push2write_opt_string_attr!(self, tag, dmid);
        writer
            .write_event(Event::Start(tag.to_borrowed()))
            .map_err(VOTableError::Write)?;
        write_elem_vec_empty_context!(self, primary_keys, writer);
        write_elem_vec_no_context!(self, elems, writer);
        writer
            .write_event(Event::End(tag.to_end()))
            .map_err(VOTableError::Write)
    }
}
