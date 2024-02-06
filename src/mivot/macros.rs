macro_rules! write_non_empty_mandatory_attributes {
  ($tag:ident, $self:ident, $($mandatory:ident),*) => {
    $($tag.push_attribute((stringify!($mandatory), $self.$mandatory.as_str())));*
  };
  ($elem_writer:ident, $self:ident, $($mandatory:ident, $name:literal),*) => {
    $($elem_writer = $elem_writer.with_attribute((stringify!($name), $self.$mandatory.as_str())));*
  };
}

macro_rules! write_non_empty_optional_attributes {
  ($tag:ident, $self:ident, $($optional:ident $(, $name:literal)?),*) => {
    $(push2write_opt_string_attr!($self, $tag, $optional $(, $name)?));*
  };
}

macro_rules! write_empty_mandatory_attributes {
  ($elem_writer:ident, $self:ident, $($mandatory:ident),*) => {
    $($elem_writer = $elem_writer.with_attribute((stringify!($mandatory), $self.$mandatory.as_str())));*
  };
  ($elem_writer:ident, $self:ident, $($mandatory:ident, $name:literal),*) => {
    $($elem_writer = $elem_writer.with_attribute((stringify!($name), $self.$mandatory.as_str())));*
  };
}

macro_rules! write_empty_optional_attributes {
  ($elem_writer:ident, $self:ident, $($optional:ident $(, $name:literal)?),*) => {
    $(write_opt_string_attr!($self, $elem_writer, $optional $(, $name)?));*
  };
}

macro_rules! impl_empty_new {
  ([$($mandatory:ident),*], [$($optional:ident),*]) => {
    paste! {
      /*
        function New
        Description:
        *   creates a new instance of the struct
        #returns Self: returns an instance of the struct
      */
      fn new_empty() -> Self {
        const NULL: &str = "@TBD";
        Self {
            // MANDATORY
            $($mandatory: NULL.into(),)*
            // OPTIONAL
            $($optional: None),*
        }
      }
    }
  };
  ([], [$($optional:ident),*], [$($vec:ident),*]) => {
    paste! {
      /*
        function New
        Description:
        *   creates a new instance of the struct
        #returns Self: returns an instance of the struct
      */
      fn new_empty() -> Self {
        Self {
            // OPTIONAL
            $($optional: None,)*
            // ELEMS
            $($vec: vec![]),*
        }
      }
    }
  };
  ([$($mandatory:ident),*], [$($optional:ident),*], [$($vec:ident),*]) => {
    paste! {
      /*
        function New
        Description:
        *   creates a new instance of the struct
        #returns Self: returns an instance of the struct
      */
      fn new_empty() -> Self {
        const NULL: &str = "@TBD";
        Self {
            // MANDATORY
            $($mandatory: NULL.into(),)*
            // OPTIONAL
            $($optional: None,)*
            // ELEMS
            $($vec: vec![]),*
        }
      }
    }
  }
}

macro_rules! impl_new {
  ([$($mandatory:ident),*], [$($optional:ident),*]) => {
    paste! {
      /*
        function New
        Description:
        *   creates a new instance of the struct
        #returns Self: returns an instance of the struct
      */
      pub fn new<I: Into<String>>($($mandatory: I),*) -> Self {
        Self {
            // MANDATORY
            $($mandatory: $mandatory.into(),)*
            // OPTIONAL
            $($optional: None),*
        }
      }
    }
  };
  ([$($mandatory:ident),*], [$($optional:ident),*], [$($vec:ident),*]) => {
    paste! {
      /*
        function New
        Description:
        *   creates a new instance of the struct
        #returns Self: returns an instance of the struct
      */
      pub fn new<I: Into<String>>($($mandatory: I),*) -> Self {
        Self {
            // MANDATORY
            $($mandatory: $mandatory.into(),)*
            // OPTIONAL
            $($optional: None,)*
            // Vecs
            $($vec: vec![]),*
        }
      }
    }
  }
}

///  E.g. `impl_builder_from_attr` leads to
///  ```ignore
///  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
///   const NULL: &str = "@TBD";
///   let mut attribute = Self::new(NULL, NULL);
///   for attr_res in attrs {
///       let attr = attr_res.map_err(VOTableError::Attr)?;
///       let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
///       let value = str::from_utf8(unescaped.as_ref())
///           .map_err(VOTableError::Utf8)?;
///       attribute = match attr.key {
///           b"mandatory" => {
///               attribute.mandatory = value.to_string();
///               attribute
///           }te.set_ref_(value),
///           b"optional" => attribute.set_optional(value),
///           _ => {
///               return Err(
///                   VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG),
///               );
///           }
///       };
///   }
///   if attribute.mandatory.as_str() == NULL {
///       Err(
///           VOTableError::Custom({
///               let res = ::alloc::fmt::format(
///                   ::core::fmt::Arguments::new_v1(
///                       &["Attributes ", "are mandatory in tag "],
///                       &[
///                           ::core::fmt::ArgumentV1::new_display(
///                               &"\'mandatory\'",
///                           ),
///                           ::core::fmt::ArgumentV1::new_display(&Self::TAG),
///                       ],
///                   ),
///               );
///               res
///           }),
///       )
///   } else {
///       Ok(attribute)
///   }
/// }
/// ```
macro_rules! impl_builder_from_attr {
  ([], [$($optional:ident $(, $name:literal)?),*] $(,[$($empty:ident $(, $empname:literal)?),*],)?) => {
    paste! {
      fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
        let mut tag = Self::new_empty();
        for attr_res in attrs {
            let attr = attr_res.map_err(VOTableError::Attr)?;
            let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
            let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
            tag = match attr.key {
              $(opt_bstringify!($optional $(, [<$name>])?) => {
                value_checker(value, &opt_stringify!($optional $(, [<$name>])?))?;
                tag.[<set_ $optional>](value)
              }),*
              $($(opt_bstringify!($empty $(, [<$empname>])?) => {tag}),*)?
                _ => {
                    return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                }
            }
        }
        Ok(tag)
      }
    }
  };
  ([$($mandatory:ident $(, $mandname:literal)?),*], [$($optional:ident $(, $name:literal)?),*] $(,[$($empty:ident $(, $empname:literal)?),*],)?) => {
      paste! {
        fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
          const NULL: &str = "@TBD";
          let mut tag = Self::new_empty();
          for attr_res in attrs {
              let attr = attr_res.map_err(VOTableError::Attr)?;
              let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
              let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
              tag = match attr.key {
                $(opt_bstringify!($mandatory $(, [<$mandname>])?) => {
                  value_checker(value, &opt_stringify!($mandatory $(, [<$mandname>])?))?;
                  tag.$mandatory = value.to_string();
                  tag
                }),*
                $(opt_bstringify!($optional $(, [<$name>])?) => {
                  value_checker(value, &opt_stringify!($optional $(, [<$name>])?))?;
                  tag.[<set_ $optional>](value)
                }),*
                $($(opt_bstringify!($empty $(, [<$empname>])?) => {tag}),*)?
                  _ => {
                      return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                  }
              }
          }
          if $(tag.$mandatory.as_str() == NULL)||* {
              Err(VOTableError::Custom(
                format!("Attributes {} are mandatory in tag {}",
                  concat!($(
                    concat!("'",stringify!($mandatory),"' ")
                  ),*),
                Self::TAG)))
          } else {
              Ok(tag)
          }
        }
      }
  };
}

///  E.g. `` leads to
///  ```ignore
/// ```
macro_rules! impl_write_e {
    ([$($mandatory:ident $(, $mandname:literal)?),*], [$($optional:ident $(, $name:literal)?),*]) => {
      paste! {
        fn write<W: std::io::Write>(
          &mut self,
          writer: &mut Writer<W>,
          _context: &Self::Context,
        ) -> Result<(), crate::error::VOTableError> {
          let mut elem_writer = writer.create_element(Self::TAG_BYTES);
          write_empty_mandatory_attributes!(elem_writer, self, $($mandatory $(, $mandname)?),*);
          write_empty_optional_attributes!(elem_writer, self, $($optional $(, $name)?),*);
          elem_writer.write_empty().map_err(VOTableError::Write)?;
          Ok(())
        }
      }
    };
}

///  E.g. `` leads to
///  ```ignore
/// fn write<W: std::io::Write>(
/// &mut self,
/// writer: &mut Writer<W>,
/// _context: &Self::Context,
/// ) -> Result<(), crate::error::VOTableError> {
///   let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
///   tag.push_attribute(("mandatory", self.mandatory.as_str()));
///   if let Some(dmid) = &self.optional {
///      tag.push_attribute(("optional", optional.as_str()));
///   }
///   writer
///       .write_event(Event::Start(tag.to_borrowed()))
///       .map_err(VOTableError::Write)?;
///   for elem in &mut self.orderelem {
///      elem.write(writer, &())?;
///   }
///   for elem in &mut self.elems {
///       elem.write(writer)?;
///   }
///   writer.write_event(Event::End(tag.to_end())).map_err(VOTableError::Write)
/// }
/// ```
macro_rules! impl_write_not_e {
  ([] $(,[$($elems:ident),*])?) => {
    paste! {
      fn write<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        _context: &Self::Context,
      ) -> Result<(), crate::error::VOTableError> {
        if $($(self.$elems.is_empty())&&*)? {
          let elem_writer = writer.create_element(Self::TAG_BYTES);
          elem_writer.write_empty().map_err(VOTableError::Write)?;
          Ok(())
        } else {
          let tag = BytesStart::borrowed_name(Self::TAG_BYTES);
          writer
              .write_event(Event::Start(tag.to_borrowed()))
              .map_err(VOTableError::Write)?;
            $($(write_elem_vec_no_context!(self, $elems, writer);)*)?
          writer
              .write_event(Event::End(tag.to_end()))
              .map_err(VOTableError::Write)
        }
      }
    }
  };
  ([$($mandatory:ident $(, $mandname:literal)?),*], [$($optional:ident $(, $name:literal)?),*], [] $(,[$($elems:ident),*])?) => {
    paste! {
      fn write<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        _context: &Self::Context,
      ) -> Result<(), crate::error::VOTableError> {
        if $($(self.$elems.is_empty())&&*)? {
          let mut elem_writer = writer.create_element(Self::TAG_BYTES);
          write_empty_mandatory_attributes!(elem_writer, self, $($mandatory $(, $mandname)?),*);
          write_empty_optional_attributes!(elem_writer, self, $($optional $(, $name)?),*);
          elem_writer.write_empty().map_err(VOTableError::Write)?;
          Ok(())
        } else {
          let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
          //MANDATORY
          write_non_empty_mandatory_attributes!(tag, self, $($mandatory $(, $mandname)?),*);
          //OPTIONAL
          write_non_empty_optional_attributes!(tag, self, $($optional $(, $name)?),*);
          writer
              .write_event(Event::Start(tag.to_borrowed()))
              .map_err(VOTableError::Write)?;
            $($(write_elem_vec_no_context!(self, $elems, writer);)*)?
          writer
              .write_event(Event::End(tag.to_end()))
              .map_err(VOTableError::Write)
        }
      }
    }
  };
    ([$($mandatory:ident $(, $mandname:literal)?),*], [$($optional:ident $(, $name:literal)?),*], [$($orderelem:ident),*] $(,[$($elems:ident),*])?) => {
        paste! {
          fn write<W: std::io::Write>(
            &mut self,
            writer: &mut Writer<W>,
            _context: &Self::Context,
          ) -> Result<(), crate::error::VOTableError> {
            // Check if all $orderelem are empty
            let all_orderelem_empty = $(self.$orderelem.is_empty())&&*;

            let _all_elems_empty = true;
            // Check if all $elems are empty, if $elems is present
            $(let _all_elems_empty = $(self.$elems.is_empty();)&&*)?

            if all_orderelem_empty && _all_elems_empty {
              let mut elem_writer = writer.create_element(Self::TAG_BYTES);
              write_empty_mandatory_attributes!(elem_writer, self, $($mandatory $(, $mandname)?),*);
              write_empty_optional_attributes!(elem_writer, self, $($optional $(, $name)?),*);
              elem_writer.write_empty().map_err(VOTableError::Write)?;
              return Ok(())
            } else {
              let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
              //MANDATORY
              write_non_empty_mandatory_attributes!(tag, self, $($mandatory $(, $mandname)?),*);
              //OPTIONAL
              write_non_empty_optional_attributes!(tag, self, $($optional $(, $name)?),*);
              writer
                  .write_event(Event::Start(tag.to_borrowed()))
                  .map_err(VOTableError::Write)?;
                $(write_elem_vec_empty_context!(self, $orderelem, writer);)*
                $($(write_elem_vec_no_context!(self, $elems, writer);)*)?
              return writer
                  .write_event(Event::End(tag.to_end()))
                  .map_err(VOTableError::Write)
            };
          }
        }
    };
}

macro_rules! non_empty_read_sub {
  ($readfn_by_ref:ident) => {
    paste! {
        fn read_sub_elements<R: std::io::BufRead>(
          &mut self,
          mut reader: Reader<R>,
          reader_buff: &mut Vec<u8>,
          context: &Self::Context,
      ) -> Result<Reader<R>, crate::error::VOTableError> {
        self.read_sub_elements_by_ref(&mut reader, reader_buff, context).map(|()| reader)
      }

      fn read_sub_elements_by_ref<R: std::io::BufRead>(
          &mut self,
          reader: &mut Reader<R>,
          reader_buff: &mut Vec<u8>,
          context: &Self::Context,
      ) -> Result<(), crate::error::VOTableError> {
          $readfn_by_ref(self, context, reader, reader_buff)
      }
    }
  };
}

macro_rules! empty_read_sub {
  () => {
    paste! {
      /*
          function read_sub_elements
          ! NO SUBELEMENTS SHOULD BE PRESENT
      */
      fn read_sub_elements<R: std::io::BufRead>(
          &mut self,
          mut _reader: Reader<R>,
          _reader_buff: &mut Vec<u8>,
          _context: &Self::Context,
      ) -> Result<Reader<R>, crate::error::VOTableError> {
          Err(VOTableError::Custom("This tag should be empty".to_owned()))
      }

      /*
          function read_sub_elements_by_ref
          ! NO SUBELEMENTS SHOULD BE PRESENT
      */
      fn read_sub_elements_by_ref<R: std::io::BufRead>(
          &mut self,
          _reader: &mut Reader<R>,
          _reader_buff: &mut Vec<u8>,
          _context: &Self::Context,
      ) -> Result<(), crate::error::VOTableError> {
          todo!()
      }
    }
  };
}

macro_rules! impl_quickrw_e {
    ([$($mandatory:ident $(, $mandname:literal)?),*], [$($optional:ident $(, $name:literal)?),*], $([$($empty:ident $(, $empname:literal)?),*],)? $tag:literal, $struct:ident, $context:tt) => {
        paste! {
          impl QuickXmlReadWrite for $struct {
            // The TAG name here: e.g "ATTRIBUTE"
            const TAG: &'static str = $tag;
            // Potential context e.g : ()
            type Context = $context;

            /*
                function from_attributes
                Description:
                *   creates Self from deserialized attributes contained inside the passed XML
                @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
                #returns Result<Self, VOTableError>: returns an instance of AttributePatA built using attributes or an error if reading doesn't work
            */
            impl_builder_from_attr!([$($mandatory $(, $mandname)?),*], [$($optional $(, $name)?),*] $(,[$($empty $(, $empname)?),*],)?);

            empty_read_sub!();

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
            impl_write_e!([$($mandatory $(, $mandname)?),*], [$($optional $(, $name)?),*]);
          }
        }
    };
}

macro_rules! impl_quickrw_not_e_no_a {
  ($tag:literal, $struct:ident, $context:tt, [$($orderelem:ident),*], $readfn:ident $(,[$($elems:ident),*])?) => {
    paste! {
      impl QuickXmlReadWrite for $struct {
        // The TAG name here: e.g "ATTRIBUTE"
        const TAG: &'static str = stringify!([<$tag>]);
        // Potential context e.g : ()
        type Context = $context;

        /*
            function from_attributes
            Description:
            *   creates Self from deserialized attributes contained inside the passed XML
            @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
            #returns Result<Self, VOTableError>: returns an instance of Self built using attributes or an error if reading doesn't work
        */
        fn from_attributes(
          attrs: quick_xml::events::attributes::Attributes,
      ) -> Result<Self, crate::error::VOTableError> {
          if attrs.count() > 0 {
              warn!("Unexpected attributes in {} (not serialized!)", $tag);
          }
          Ok(Self::default())
      }

        non_empty_read_sub!($readfn);

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
        impl_write_not_e!([$($orderelem),*] $(,[$($elems),*])?);
      }
    }
  };
}

macro_rules! impl_quickrw_not_e {
  ([$($mandatory:ident $(, $mandname:literal)?),*], [$($optional:ident $(, $name:literal)?),*], $([$($empty:ident $(, $empname:literal)?),*],)? $tag:literal, $struct:ident, $context:tt, [$($orderelem:ident),*], $readfn:ident $(,[$($elems:ident),*])?) => {
    paste! {
      impl QuickXmlReadWrite for $struct {
        // The TAG name here: e.g "ATTRIBUTE"
        const TAG: &'static str = stringify!([<$tag>]);
        // Potential context e.g : ()
        type Context = $context;

        /*
            function from_attributes
            Description:
            *   creates Self from deserialized attributes contained inside the passed XML
            @param attrs quick_xml::events::attributes::Attributes: attributes from the quick_xml reader
            #returns Result<Self, VOTableError>: returns an instance of Self built using attributes or an error if reading doesn't work
        */
        impl_builder_from_attr!([$($mandatory $(, $mandname)?),*], [$($optional $(, $name)?),*] $(,[$($empty $(, $empname)?),*],)?);

        non_empty_read_sub!($readfn);

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
        impl_write_not_e!([$($mandatory $(, $mandname)?),*], [$($optional $(, $name)?),*] ,[$($orderelem),*] $(,[$($elems),*])?);
      }
    }
  };
}

macro_rules! opt_bstringify {
  ($ident:ident) => {
    bstringify!($ident)
  };
  ($ident:ident, $name:tt) => {
    bstringify!($name)
  };
}

macro_rules! opt_stringify {
  ($ident:ident) => {
    stringify!($ident)
  };
  ($ident:ident, $name:tt) => {
    stringify!($name)
  };
}
