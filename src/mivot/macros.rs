macro_rules! write_non_empty_mandatory_attributes {
  ($tag:ident, $self:ident, $($mandatory:ident),*) => {
    $($tag.push_attribute((stringify!($mandatory), $self.$mandatory.as_str())));*
  };
}

macro_rules! write_non_empty_optional_attributes {
  ($tag:ident, $self:ident, $($optional:ident),*) => {
    $(push2write_opt_string_attr!($self, $tag, $optional));*
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
                  format!("Attributes {}are mandatory in tag {}",
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

//todo impl_write_empty
//todo impl_write_not_empty

//todo impl_quickrw_e
//todo impl_quickrw_note
//todo impl_quickrw_w_children
//todo impl_quickrw_w_events

macro_rules! opt_bstringify {
    ($ident:ident) => {
        bstringify!($ident)
    };
    ($ident:ident, $name:tt) => {
        bstringify!($name)
    };
}
