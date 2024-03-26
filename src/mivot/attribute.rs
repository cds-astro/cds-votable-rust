//! Contains common code for `ATTRIBUTE` child of `COLLECTION` and `INSTANCE`
//! in both `GLOBALS` and `TEMPLATES`.

use std::str;

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, EmptyElem, VOTableElement,
};

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

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    match self {
      Self::Ref { ref_ } => f("ref", ref_.as_str()),
      Self::Value { value } => f("value", value.as_str()),
      Self::RefAndValue { ref_, value } => {
        f("ref", ref_.as_str());
        f("value", value.as_str())
      }
    };
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

  impl_builder_mandatory_string_attr!(dmrole);
  impl_builder_mandatory_string_attr!(dmtype);
  impl_builder_mandatory_attr!(ref_or_val_or_both, RefOrValueOrBoth);
  impl_builder_opt_attr!(arrayindex, u32);
  impl_builder_opt_string_attr!(unit);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_attribute_childof_instance(self)
  }
}

impl VOTableElement for AttributeChildOfInstance {
  const TAG: &'static str = "ATTRIBUTE";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::from_ref(DEFAULT_VALUE, DEFAULT_VALUE, DEFAULT_VALUE).set_attrs(attrs).and_then(|attr| {
      if attr.dmrole.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'dmrole' not found in tag '{}' child of INSTANCE child of GLOBALS",
          Self::TAG
        )))
      } else if attr.dmtype.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'dmtype' not found in tag '{}' child of INSTANCE child of GLOBALS",
          &Self::TAG
        )))
      } else if let RefOrValueOrBoth::Ref { ref_: e } = &attr.ref_or_val_or_both {
        if e.as_str() == DEFAULT_VALUE {
          Err(VOTableError::Custom(format!(
            "Mandatory attributes 'ref' or 'value' not found in tag '{}' child of INSTANCE child of GLOBALS",
            &Self::TAG
          )))
        } else {
          Ok(attr)
        }
      } else {
        Ok(attr)
      }
    })
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut ref_ = String::from("");
    let mut value = String::from("");
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "dmrole" => self.set_dmrole_by_ref(val),
        "dmtype" => self.set_dmtype_by_ref(val),
        "ref" => ref_ = val.into(),
        "value" => value = val.into(),
        "arrayindex" => {
          self.set_arrayindex_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?)
        }
        "unit" => self.set_unit_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    self.set_ref_or_val_or_both_by_ref(RefOrValueOrBoth::from_possibly_empty_ref_or_val(
      ref_, value,
    )?);
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("dmrole", self.dmrole.as_str());
    f("dmtype", self.dmtype.as_str());
    self.ref_or_val_or_both.for_each_attribute(&mut f);
    if let Some(arrayindex) = &self.arrayindex {
      f("arrayindex", arrayindex.to_string().as_str());
    }
    if let Some(unit) = &self.unit {
      f("unit", unit.as_str());
    }
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

  impl_builder_mandatory_string_attr!(dmtype);
  impl_builder_mandatory_attr!(ref_or_val_or_both, RefOrValueOrBoth);
  impl_builder_opt_attr!(arrayindex, u32);
  impl_builder_opt_string_attr!(unit);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_attribute_childof_collection(self)
  }
}

impl VOTableElement for AttributeChildOfCollection {
  const TAG: &'static str = "ATTRIBUTE";

  type MarkerType = EmptyElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    Self::from_ref(DEFAULT_VALUE, DEFAULT_VALUE).set_attrs(attrs).and_then(|attr| {
      if attr.dmtype.as_str() == DEFAULT_VALUE {
        Err(VOTableError::Custom(format!(
          "Mandatory attribute 'dmtype' not found in tag '{}' child of COLLECTION child of GLOBALS",
          &Self::TAG
        )))
      } else if let RefOrValueOrBoth::Ref { ref_: e } = &attr.ref_or_val_or_both {
        if e.as_str() == DEFAULT_VALUE {
          Err(VOTableError::Custom(format!(
            "Mandatory attributes 'ref' or 'value' not found in tag '{}' child of COLLECTION child of GLOBALS",
            &Self::TAG
          )))
        } else {
          Ok(attr)
        }
      } else {
        Ok(attr)
      }
    })
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut ref_ = String::from("");
    let mut value = String::from("");
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "dmtype" => self.set_dmtype_by_ref(val),
        "ref" => ref_ = val.into(),
        "value" => value = val.into(),
        "arrayindex" => {
          self.set_arrayindex_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?)
        }
        "unit" => self.set_unit_by_ref(val),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    self.set_ref_or_val_or_both_by_ref(RefOrValueOrBoth::from_possibly_empty_ref_or_val(
      ref_, value,
    )?);
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("dmtype", self.dmtype.as_str());
    self.ref_or_val_or_both.for_each_attribute(&mut f);
    if let Some(arrayindex) = &self.arrayindex {
      f("arrayindex", arrayindex.to_string().as_str());
    }
    if let Some(unit) = &self.unit {
      f("unit", unit.as_str());
    }
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
