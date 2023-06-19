macro_rules! write_non_empty_mandatory_attributes {
  ($tag:ident, $self:ident, $($mandatory:ident),*) => {
    $($tag.push_attribute((stringify!($mandatory), $self.$mandatory.as_str())));*
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
}

macro_rules! write_empty_optional_attributes {
  ($elem_writer:ident, $self:ident, $($optional:ident $(, $name:literal)?),*) => {
    $(write_opt_string_attr!($self, $elem_writer, $optional $(, $name)?));*
  };
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
    ([$($mandatory:ident),*], [$($optional:ident $(, $name:literal)?),*]) => {
        paste! {
          fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
            const NULL: &str = "@TBD";
            let mut attribute = Self::new(NULL, NULL);
            for attr_res in attrs {
                let attr = attr_res.map_err(VOTableError::Attr)?;
                let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
                let value = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
                attribute = match attr.key {
                  $(bstringify!($mandatory) => {
                    attribute.$mandatory = value.to_string();
                    attribute
                  }),*
                  $(opt_bstringify!($optional $(, [<$name>])?) => {
                    attribute.[<set_ $optional>](value)
                  }),*
                    _ => {
                        return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
                    }
                }
            }
            if $(attribute.$mandatory.as_str() == NULL)||* {
                Err(VOTableError::Custom(
                  format!("Attributes {} are mandatory in tag {}",
                    concat!($(
                      concat!("'",stringify!($mandatory),"' ")
                    ),*),
                  Self::TAG)))
            } else {
                Ok(attribute)
            }
          }
        }
    };
}

///  E.g. `` leads to
///  ```ignore
/// ```
macro_rules! impl_write_e {
    ([$($mandatory:ident),*], [$($optional:ident $(, $name:literal)?),*]) => {
      paste! {
        fn write<W: std::io::Write>(
          &mut self,
          writer: &mut Writer<W>,
          _context: &Self::Context,
        ) -> Result<(), crate::error::VOTableError> {
          let mut elem_writer = writer.create_element(Self::TAG_BYTES);
          write_empty_mandatory_attributes!(elem_writer, self, $($mandatory),*);
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
    ([$($mandatory:ident),*], [$($optional:ident $(, $name:literal)?),*], [$($orderelem:ident),*] $(,[$($elems:ident),*])?) => {
        paste! {
          fn write<W: std::io::Write>(
            &mut self,
            writer: &mut Writer<W>,
            _context: &Self::Context,
          ) -> Result<(), crate::error::VOTableError> {
            let mut tag = BytesStart::borrowed_name(Self::TAG_BYTES);
            //MANDATORY
            write_non_empty_mandatory_attributes!(tag, self, $($mandatory),*);
            //OPTIONAL
            write_non_empty_optional_attributes!(tag, self, $($optional $(, $name)?),*);
            writer
                .write_event(Event::Start(tag.to_borrowed()))
                .map_err(VOTableError::Write)?;
              $(write_elem_vec_empty_context!(self, $orderelem, writer);)*
              $($(write_elem_vec_no_context!(self, $elems, writer);)*)?
            writer
                .write_event(Event::End(tag.to_end()))
                .map_err(VOTableError::Write)
          }
        }
    };
}

macro_rules! impl_quickrw_e {
    ([$($mandatory:ident),*], [$($optional:ident $(, $name:literal)?),*], $tag:literal, $struct:ident, $context:tt) => {
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
            impl_builder_from_attr!([$($mandatory),*], [$($optional $(, $name)?),*]);

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
                todo!()
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
            impl_write_e!([$($mandatory),*], [$($optional $(, $name)?),*]);
          }
        }
    };
}

macro_rules! impl_quickrw_not_e {
  ([$($mandatory:ident),*], [$($optional:ident $(, $name:literal)?),*], $tag:literal, $struct:ident, $context:tt, [$($orderelem:ident),*], $readfn:ident $(,[$($elems:ident),*])?) => {
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
        impl_builder_from_attr!([$($mandatory),*], [$($optional $(, $name)?),*]);

        /*
          function read_sub_elements
          Description:
          *   see function read_sub_elem from caller
        */
        fn read_sub_elements<R: std::io::BufRead>(
            &mut self,
            reader: Reader<R>,
            reader_buff: &mut Vec<u8>,
            context: &Self::Context,
        ) -> Result<Reader<R>, crate::error::VOTableError> {
            $readfn(self, context, reader, reader_buff)
        }

        /*
            function read_sub_elements_by_ref
            todo UNIMPLEMENTED
        */
        fn read_sub_elements_by_ref<R: std::io::BufRead>(
            &mut self,
            __reader: &mut Reader<R>,
            __reader_buff: &mut Vec<u8>,
            __context: &Self::Context,
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
        impl_write_not_e!([$($mandatory),*], [$($optional $(, $name)?),*] ,[$($orderelem),*] $(,[$($elems),*])?);
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
