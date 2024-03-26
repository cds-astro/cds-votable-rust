//! Module dedicated to the `REPORT` tag.

use std::str::{self, FromStr};

use paste::paste;

use crate::{
  error::VOTableError, mivot::VodmlVisitor, utils::unexpected_attr_err, HasContent, HasContentElem,
  VOTableElement,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Status {
  OK,
  FAILED,
}
impl FromStr for Status {
  type Err = String;

  fn from_str(str: &str) -> Result<Self, Self::Err> {
    match str {
      "OK" => Ok(Self::OK),
      "FAILED" => Ok(Self::FAILED),
      _ => Err(format!(
        "Attribute 'status' error in 'REPORT'. Expected: either 'OK' or 'FAILED'. Actual: '{}'.",
        str
      )),
    }
  }
}

impl ToString for Status {
  fn to_string(&self) -> String {
    match self {
      Self::OK => "OK".to_owned(),
      Self::FAILED => "FAILED".to_owned(),
    }
  }
}

/// Structure storing the content of the `REPORT` tag.
/// Tells the client whether the annotation process succeeded or not.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Report {
  /// Status of the annotation process.
  pub status: Status,
  /// Report content.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<String>,
}

impl Report {
  pub fn new(status: Status) -> Self {
    Report {
      status,
      content: None,
    }
  }

  impl_builder_mandatory_attr!(status, Status);
  impl_builder_opt_string_attr!(content);

  pub fn visit<V: VodmlVisitor>(&mut self, visitor: &mut V) -> Result<(), V::E> {
    visitor.visit_report(self)
  }
}
impl_has_content!(Report);

impl VOTableElement for Report {
  const TAG: &'static str = "REPORT";

  type MarkerType = HasContentElem;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    let mut status: Option<Status> = None;
    // Look for attributes (especially mandatory attributes)
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "status" => status = Some(val.as_ref().parse().map_err(VOTableError::Custom)?),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    // Set from found attributes
    if let Some(status) = status {
      Ok(Self::new(status))
    } else {
      Err(VOTableError::Custom(format!(
        "Attributes 'status' is mandatory in tag '{}'",
        Self::TAG
      )))
    }
  }

  fn set_attrs_by_ref<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    for (key, val) in attrs {
      let key = key.as_ref();
      match key {
        "status" => self.set_status_by_ref(val.as_ref().parse().map_err(VOTableError::Custom)?),
        _ => return Err(unexpected_attr_err(key, Self::TAG)),
      }
    }
    Ok(())
  }

  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    f("status", self.status.to_string().as_str());
  }
}
