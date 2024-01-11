//! Contains common code for `ATTRIBUTE` child of `COLLECTION` and `INSTANCE`
//! in both `GLOBALS` and `TEMPLATES`.

use std::{io::Write, str};

use paste::paste;

use quick_xml::{events::attributes::Attributes, ElementWriter, Reader, Writer};

use crate::{error::VOTableError, mivot::VodmlVisitor, QuickXmlReadWrite};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum RefOrValueOrBoth {
  Ref {
    #[serde(rename = "ref")]
    ref_: String,
  },
  Value {
    value: String,
  },
  /// In this pattern, the value has the role of a default value
  /// in case the referenced `FIELD` contains a NULL value or if the
  /// the referenced `PARAM` does not exists.
  RefAndValue {
    #[serde(rename = "ref")]
    ref_: String,
    value: String,
  },
}

impl RefOrValueOrBoth {
  pub fn from_ref<N: Into<String>>(ref_: N) -> Self {
    RefOrValueOrBoth::Ref { ref_: ref_.into() }
  }

  pub fn from_value<N: Into<String>>(value: N) -> Self {
    RefOrValueOrBoth::Value {
      value: value.into(),
    }
  }

  pub fn from_ref_with_default<N: Into<String>>(ref_: N, default_value: N) -> Self {
    RefOrValueOrBoth::RefAndValue {
      ref_: ref_.into(),
      value: default_value.into(),
    }
  }

  pub fn from_possibly_empty_ref_or_val(ref_: String, value: String) -> Result<Self, VOTableError> {
    match ((!ref_.is_empty() as u8) << 1) + (!value.is_empty() as u8) {
      0 => Err(VOTableError::Custom(String::from(
        "Attributes 'ref' and 'value' are both empty in tag ATTRIBUTE",
      ))),
      1 => Ok(Self::from_value(value)),
      2 => Ok(Self::from_ref(ref_)),
      3 => Ok(Self::from_ref_with_default(ref_, value)),
      _ => unreachable!(),
    }
  }

  pub fn push_in_tag<'a, W: Write>(&'a self, tag: ElementWriter<'a, W>) -> ElementWriter<W> {
    match self {
      Self::Ref { ref_ } => tag.with_attribute(("ref", ref_.as_str())),
      Self::Value { value } => tag.with_attribute(("value", value.as_str())),
      Self::RefAndValue { ref_, value } => tag
        .with_attribute(("ref", ref_.as_str()))
        .with_attribute(("value", value.as_str())),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AttributeChildOfInstance {
  /// Attribute name.
  pub dmrole: String,
  /// Attribute type.
  pub dmtype: String,
  /// Reference (to a PARAM) or value of the attribute.
  #[serde(flatten)]
  pub ref_or_val_or_both: RefOrValueOrBoth,
  /// In case the value/referenced param is an array,
  /// provide the index of the attribute value inh the array.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arrayindex: Option<u32>,
  /// Unit of the attribute.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
}
impl AttributeChildOfInstance {
  pub fn new<N: Into<String>>(dmrole: N, dmtype: N, ref_or_val_or_both: RefOrValueOrBoth) -> Self {
    Self {
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      ref_or_val_or_both,
      arrayindex: None,
      unit: None,
    }
  }

  pub fn from_ref<N: Into<String>>(dmrole: N, dmtype: N, ref_: N) -> Self {
    Self {
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      ref_or_val_or_both: RefOrValueOrBoth::from_ref(ref_),
      arrayindex: None,
      unit: None,
    }
  }

  pub fn from_val<N: Into<String>>(dmrole: N, dmtype: N, value: N) -> Self {
    Self {
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      ref_or_val_or_both: RefOrValueOrBoth::from_value(value),
      arrayindex: None,
      unit: None,
    }
  }

  pub fn from_ref_with_default<N: Into<String>>(
    dmrole: N,
    dmtype: N,
    ref_: N,
    default_value: N,
  ) -> Self {
    Self {
      dmrole: dmrole.into(),
      dmtype: dmtype.into(),
      ref_or_val_or_both: RefOrValueOrBoth::from_ref_with_default(ref_, default_value),
      arrayindex: None,
      unit: None,
    }
  }

  impl_builder_opt_attr!(arrayindex, u32);
  impl_builder_opt_string_attr!(unit);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_attribute_childof_instance(self)
  }
}
impl QuickXmlReadWrite for AttributeChildOfInstance {
  const TAG: &'static str = "ATTRIBUTE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut dmrole = String::from("");
    let mut dmtype = String::from("");
    let mut ref_ = String::from("");
    let mut value = String::from("");
    let mut arrayindex: Option<u32> = None;
    let mut unit = String::from("");

    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let val = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !val.is_empty() {
        match attr.key {
          b"dmrole" => dmrole = val.to_string(),
          b"dmtype" => dmtype = val.to_string(),
          b"ref" => ref_ = val.to_string(),
          b"value" => value = val.to_string(),
          b"arrayindex" => {
            arrayindex = Some(val.parse::<u32>().map_err(|e| {
              VOTableError::Custom(format!(
                "Unable to parse 'arrayindex' attribute '{}': {}",
                val, e
              ))
            })?)
          }
          b"unit" => unit = val.to_string(),
          _ => {
            return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
          }
        }
      };
    }

    if dmrole.is_empty() {
      Err(VOTableError::Custom(format!(
        "Attribute 'dmrole' mandatory in tag {} child of INSTANCE child of GLOBALS",
        &Self::TAG
      )))
    } else if dmtype.is_empty() {
      Err(VOTableError::Custom(format!(
        "Attribute 'dmtype'  mandatory in tag {} child of INSTANCE child of GLOBALS",
        &Self::TAG
      )))
    } else {
      let ref_or_val = RefOrValueOrBoth::from_possibly_empty_ref_or_val(ref_, value)?;
      let mut tag = Self::new(dmrole, dmtype, ref_or_val);
      if let Some(arrayindex) = arrayindex {
        tag = tag.set_arrayindex(arrayindex);
      }
      if !unit.is_empty() {
        tag = tag.set_unit(unit);
      }
      Ok(tag)
    }
  }

  empty_read_sub!();

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = writer
      .create_element(Self::TAG_BYTES)
      .with_attribute(("dmrole", self.dmrole.as_str()))
      .with_attribute(("dmtype", self.dmtype.as_str()));
    tag = self.ref_or_val_or_both.push_in_tag(tag);
    write_opt_tostring_attr!(self, tag, arrayindex, "arrayindex");
    write_opt_string_attr!(self, tag, unit);
    tag.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AttributeChildOfCollection {
  /// Attribute type.
  pub dmtype: String,
  /// Reference (to a PARAM) or value of the attribute.
  #[serde(flatten)]
  pub ref_or_val_or_both: RefOrValueOrBoth,
  /// In case the value/referenced param is an array,
  /// provide the index of the attribute value inh the array.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arrayindex: Option<u32>,
  /// Unit of the attribute.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
}
impl AttributeChildOfCollection {
  pub fn new<N: Into<String>>(dmtype: N, ref_or_val_or_both: RefOrValueOrBoth) -> Self {
    Self {
      dmtype: dmtype.into(),
      ref_or_val_or_both,
      arrayindex: None,
      unit: None,
    }
  }

  pub fn from_ref<N: Into<String>>(dmtype: N, ref_: N) -> Self {
    Self {
      dmtype: dmtype.into(),
      ref_or_val_or_both: RefOrValueOrBoth::from_ref(ref_),
      arrayindex: None,
      unit: None,
    }
  }

  pub fn from_val<N: Into<String>>(dmtype: N, value: N) -> Self {
    Self {
      dmtype: dmtype.into(),
      ref_or_val_or_both: RefOrValueOrBoth::from_value(value),
      arrayindex: None,
      unit: None,
    }
  }

  pub fn from_ref_with_default<N: Into<String>>(dmtype: N, ref_: N, default_value: N) -> Self {
    Self {
      dmtype: dmtype.into(),
      ref_or_val_or_both: RefOrValueOrBoth::from_ref_with_default(ref_, default_value),
      arrayindex: None,
      unit: None,
    }
  }

  impl_builder_opt_attr!(arrayindex, u32);
  impl_builder_opt_string_attr!(unit);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_attribute_childof_collection(self)
  }
}
impl QuickXmlReadWrite for AttributeChildOfCollection {
  const TAG: &'static str = "ATTRIBUTE";
  type Context = ();

  fn from_attributes(attrs: Attributes) -> Result<Self, VOTableError> {
    let mut dmtype = String::from("");
    let mut ref_ = String::from("");
    let mut value = String::from("");
    let mut arrayindex: Option<u32> = None;
    let mut unit = String::from("");

    for attr_res in attrs {
      let attr = attr_res.map_err(VOTableError::Attr)?;
      let unescaped = attr.unescaped_value().map_err(VOTableError::Read)?;
      let val = str::from_utf8(unescaped.as_ref()).map_err(VOTableError::Utf8)?;
      if !val.is_empty() {
        match attr.key {
          b"dmtype" => dmtype = val.to_string(),
          b"ref" => ref_ = val.to_string(),
          b"value" => value = val.to_string(),
          b"arrayindex" => {
            arrayindex = Some(val.parse::<u32>().map_err(|e| {
              VOTableError::Custom(format!(
                "Unable to parse 'arrayindex' attribute '{}': {}",
                val, e
              ))
            })?)
          }
          b"unit" => unit = val.to_string(),
          _ => {
            return Err(VOTableError::UnexpectedAttr(attr.key.to_vec(), Self::TAG));
          }
        }
      };
    }

    if dmtype.is_empty() {
      Err(VOTableError::Custom(format!(
        "Attribute 'dmtype'  mandatory in tag {} child of COLLECTION child of GLOBALS",
        &Self::TAG
      )))
    } else {
      let ref_or_val = RefOrValueOrBoth::from_possibly_empty_ref_or_val(ref_, value)?;
      let mut tag = Self::new(dmtype, ref_or_val);
      if let Some(arrayindex) = arrayindex {
        tag = tag.set_arrayindex(arrayindex);
      }
      if !unit.is_empty() {
        tag = tag.set_unit(unit);
      }
      Ok(tag)
    }
  }

  empty_read_sub!();

  fn write<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    let mut tag = writer
      .create_element(Self::TAG_BYTES)
      .with_attribute(("dmtype", self.dmtype.as_str()));
    tag = self.ref_or_val_or_both.push_in_tag(tag);
    write_opt_tostring_attr!(self, tag, arrayindex);
    write_opt_tostring_attr!(self, tag, unit);
    tag.write_empty().map_err(VOTableError::Write)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    mivot::test::{get_xml, test_error},
    tests::test_read,
  };

  use super::AttributeChildOfInstance;

  #[test]
  fn test_attribute_a_read() {
    // OK MODELS
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.1.xml");
    println!("testing 7.1");
    test_read::<AttributeChildOfInstance>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.2.xml");
    println!("testing 7.2");
    test_read::<AttributeChildOfInstance>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.3.xml");
    println!("testing 7.3");
    test_read::<AttributeChildOfInstance>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.5.xml");
    println!("testing 7.5");
    test_read::<AttributeChildOfInstance>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.6.xml");
    println!("testing 7.6");
    test_read::<AttributeChildOfInstance>(&xml);
    let xml = get_xml("./resources/mivot/7/test_7_ok_7.7.xml");
    println!("testing 7.7");
    test_read::<AttributeChildOfInstance>(&xml);

    // Ref and value are null, should not be ok...
    //  let xml = get_xml("./resources/mivot/7/test_7_ok_7.14.xml");
    //  println!("testing 7.14");
    //  test_read::<AttributeChildOfInstance>(&xml);

    // KO MODELS
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.4.xml");
    println!("testing 7.4"); // valid dmrole + dmtype; must have one or both of (value, ref)
    test_error::<AttributeChildOfInstance>(&xml, false);
    /*
    // Laurent told me a value can be an array so it is ok to have an arrayindex with a value...
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.8.xml");
    println!("testing 7.8"); // valid dmrole + dmtype + value + arrayindex; arrayindex only assoc. with ref.
    test_error::<AttributeChildOfInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.9.xml");
    println!("testing 7.9"); // no dmrole + dmtype + value; must have non-empty dmrole
    test_error::<AttributeChildOfInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.10.xml");
    println!("testing 7.10"); // valid dmrole + no dmtype + value; must have non-empty dmtype.
    test_error::<AttributeChildOfInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.11.xml");
    println!("testing 7.11"); // empty dmrole + dmtype + ref; must have non-empty dmrole in this context
    test_error::<AttributeChildOfInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.12.xml");
    println!("testing 7.12"); // dmtype must not be empty
    test_error::<AttributeChildOfInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.13.xml");
    println!("testing 7.13"); // ref must not be empty
    test_error::<AttributeChildOfInstance>(&xml, false);
    let xml = get_xml("./resources/mivot/7/test_7_ko_7.15.xml");
    println!("testing 7.15"); // valid dmrole + dmtype + ref + (arrayindex < 0); invalid arrayindex value (< 0)
    test_error::<AttributeChildOfInstance>(&xml, false);
    */
  }
}
