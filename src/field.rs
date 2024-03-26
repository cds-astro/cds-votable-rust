//! Struct dedicated to the `FIELD` tag.

use std::{
  collections::HashMap,
  fmt,
  io::{BufRead, Write},
  num::ParseIntError,
  str::{self, FromStr},
};

use log::warn;
use paste::paste;
use quick_xml::{events::Event, Reader, Writer};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::{
  datatype::Datatype,
  desc::Description,
  error::VOTableError,
  link::Link,
  utils::{discard_comment, discard_event, is_empty},
  values::Values,
  HasSubElements, HasSubElems, QuickXmlReadWrite, TableDataContent, VOTableElement, VOTableVisitor,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArraySize {
  // 8
  Fixed1D { size: u32 },
  // 8x5x...
  FixedND { sizes: Vec<u32> },
  // *
  Variable1D,
  // 8x...x*
  VariableND { sizes: Vec<u32> },
  // 8*
  VariableWithUpperLimit1D { upper_limit: u32 },
  // 8x...x5*
  VariableWithUpperLimitND { sizes: Vec<u32>, upper_limit: u32 },
}

impl ArraySize {
  pub fn new_fixed_1d(size: u32) -> Self {
    ArraySize::Fixed1D { size }
  }

  pub fn new_fixed_nd(sizes: Vec<u32>) -> Self {
    ArraySize::FixedND { sizes }
  }

  pub fn new_variable_1d() -> Self {
    ArraySize::Variable1D
  }

  pub fn new_variable_nd(sizes: Vec<u32>) -> Self {
    ArraySize::VariableND { sizes }
  }

  pub fn new_variable_1d_with_upper_limit(upper_limit: u32) -> Self {
    ArraySize::VariableWithUpperLimit1D { upper_limit }
  }

  pub fn new_variable_nd_with_upper_limit(sizes: Vec<u32>, upper_limit: u32) -> Self {
    ArraySize::VariableWithUpperLimitND { sizes, upper_limit }
  }

  pub fn is_fixed(&self) -> bool {
    matches!(self, Self::Fixed1D { size: _ } | Self::FixedND { sizes: _ })
  }

  pub fn is_variable(&self) -> bool {
    !self.is_fixed()
  }

  pub fn has_upper_limit(&self) -> bool {
    matches!(
      self,
      Self::VariableWithUpperLimit1D { upper_limit: _ }
        | Self::VariableWithUpperLimitND {
          sizes: _,
          upper_limit: _,
        }
    )
  }

  /// Returns:
  /// * `None` if the size is variable with no upper limit
  /// * `Some(Ok(size))` if the size is fixed
  /// * `Some(Err(upper_limit))` if the size is variable but with an upper limit
  pub fn n_elems(&self) -> Option<Result<u32, u32>> {
    let compute_size = |elems: &Vec<u32>| elems.iter().fold(1_u32, |acc, n| acc * *n);
    match self {
      Self::Fixed1D { size } => Some(Ok(*size)),
      Self::FixedND { sizes } => Some(Ok(compute_size(sizes))),
      Self::Variable1D | Self::VariableND { sizes: _ } => None,
      Self::VariableWithUpperLimit1D { upper_limit } => Some(Err(*upper_limit)),
      Self::VariableWithUpperLimitND { sizes, upper_limit } => {
        Some(Err(compute_size(sizes) * upper_limit))
      }
    }
  }
}

impl FromStr for ArraySize {
  type Err = ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let arraysize = s.trim();
    let (arraysize, is_variable) = if arraysize.ends_with('*') {
      (arraysize.strip_suffix('*').unwrap_or(""), true)
    } else {
      (arraysize, false)
    };
    if arraysize.is_empty() {
      return Ok(Self::Variable1D);
    }
    let (arraysize, end_with_x) = if arraysize.ends_with('x') {
      (arraysize.strip_suffix('x').unwrap_or(""), true)
    } else {
      (arraysize, false)
    };
    let mut elems = arraysize
      .split('x')
      .map(|v| v.parse::<u32>())
      .collect::<Result<Vec<u32>, ParseIntError>>()?;
    Ok(if !is_variable {
      if elems.len() == 1 {
        Self::Fixed1D { size: elems[0] }
      } else {
        Self::FixedND { sizes: elems }
      }
    } else if end_with_x {
      Self::VariableND { sizes: elems }
    } else if elems.len() == 1 {
      Self::VariableWithUpperLimit1D {
        upper_limit: elems[0],
      }
    } else {
      let upper_limit = elems.pop().unwrap_or(0);
      Self::VariableWithUpperLimitND {
        sizes: elems,
        upper_limit,
      }
    })
  }
}

impl fmt::Display for ArraySize {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fn write_sizes(sizes: &Vec<u32>, f: &mut fmt::Formatter) -> fmt::Result {
      let mut first = true;
      for size in sizes {
        if first {
          write!(f, "{}", size)?;
          first = false;
        } else {
          write!(f, "x{}", size)?;
        }
      }
      Ok(())
    }
    match self {
      Self::Fixed1D { size } => write!(f, "{}", size),
      Self::FixedND { sizes } => write_sizes(sizes, f),
      Self::Variable1D => write!(f, "*"),
      Self::VariableND { sizes } => write_sizes(sizes, f).and_then(|()| write!(f, "x*")),
      Self::VariableWithUpperLimit1D { upper_limit } => write!(f, "{}*", upper_limit),
      Self::VariableWithUpperLimitND { sizes, upper_limit } => {
        write_sizes(sizes, f).and_then(|()| write!(f, "x{}*", upper_limit))
      }
    }
  }
}
impl Serialize for ArraySize {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for ArraySize {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    FromStr::from_str(&s).map_err(de::Error::custom)
  }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Precision {
  /// Significant figures after the decimal (for the decimal notation)
  F { n_decimal: u8 },
  /// Number of significant figures (for the scientific notation)
  E { n_significant: u8 },
}
impl Precision {
  pub fn new_dec(n_decimal: u8) -> Self {
    Precision::F { n_decimal }
  }
  pub fn new_sci(n_significant: u8) -> Self {
    Precision::E { n_significant }
  }
  // Add engineer?!! (Not in the VO standard)
}

impl FromStr for Precision {
  type Err = ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(match &s[0..1] {
      "E" => Precision::new_sci(s[1..].parse()?),
      "F" => Precision::new_dec(s[1..].parse()?),
      _ => Precision::new_dec(s.parse()?),
    })
  }
}

impl fmt::Display for Precision {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Precision::F { n_decimal } => write!(f, "{}", n_decimal),
      Precision::E { n_significant } => write!(f, "E{}", n_significant),
    }
  }
}
impl Serialize for Precision {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for Precision {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    FromStr::from_str(&s).map_err(de::Error::custom)
  }
}

/// Struct corresponding to the `FIELD` XML tag.
///
/// From the VOTable [official document](https://www.ivoa.net/documents/VOTable/20191021/REC-VOTable-1.4-20191021.html#sec:values),
/// the 'null' attribute in VALUES is reserved to integer types:
/// "This mechanism is only intended for use with integer types; it should not be used for floating point types, which can use NaN instead."
/// "This mechanism for representing null values is required for integer columns in the BINARY serialization [but not in BINARY2]"
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
  // attributes
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub name: String,
  pub datatype: Datatype, // part of the schema
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unit: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub precision: Option<Precision>, // part of the schema
  #[serde(skip_serializing_if = "Option::is_none")]
  pub width: Option<u16>, // part of the schema
  #[serde(skip_serializing_if = "Option::is_none")]
  pub xtype: Option<String>,
  #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
  pub ref_: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ucd: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub utype: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub arraysize: Option<ArraySize>, // part of the schema ?
  // extra attributes
  #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
  pub extra: HashMap<String, Value>,
  // sub-elements
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<Description>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub values: Option<Values>, // part of the schema (null attribute or Enum coder)
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub links: Vec<Link>,
}
// new_from_schema (may loose information on complex types, like prefix, suffix, ...)
// to_schema (lose information like units, ucds, min, max, ...)

impl Field {
  pub fn new<N: Into<String>>(name: N, datatype: Datatype) -> Self {
    Field {
      id: None,
      name: name.into(),
      datatype,
      unit: None,
      precision: None,
      width: None,
      xtype: None,
      ref_: None,
      ucd: None,
      utype: None,
      arraysize: None,
      extra: Default::default(),
      description: None,
      values: None,
      links: vec![],
    }
  }

  // attributes
  impl_builder_opt_string_attr!(id);
  impl_builder_mandatory_string_attr!(name);
  impl_builder_mandatory_attr!(datatype, Datatype);
  impl_builder_opt_string_attr!(unit);
  impl_builder_opt_attr!(precision, Precision);
  impl_builder_opt_attr!(width, u16);
  impl_builder_opt_string_attr!(xtype);
  impl_builder_opt_string_attr!(ref_, ref);
  impl_builder_opt_string_attr!(ucd);
  impl_builder_opt_string_attr!(utype);
  impl_builder_opt_attr!(arraysize, ArraySize);
  // extra attributes
  impl_builder_insert_extra!();
  // sub-elements
  impl_builder_opt_subelem!(description, Description);
  impl_builder_opt_subelem!(values, Values);
  impl_builder_push!(Link);

  /// Look for a NULL value and returns it
  pub fn null_value(&self) -> Option<&String> {
    self.values.as_ref().and_then(|values| values.null.as_ref())
  }

  pub fn visit<C, V>(&mut self, visitor: &mut V) -> Result<(), V::E>
  where
    C: TableDataContent,
    V: VOTableVisitor<C>,
  {
    visitor.visit_field_start(self)?;
    if let Some(description) = &mut self.description {
      visitor.visit_description(description)?;
    }
    if let Some(values) = &mut self.values {
      values.visit(visitor)?;
    }
    for l in &mut self.links {
      visitor.visit_link(l)?;
    }
    visitor.visit_field_ended(self)
  }

  /// Enrich this Field filling its empty elements with the ones provided in the given `other` Field.
  /// Returns an error is the datatypes and/or arraysize are not the same.
  pub fn merge_enrich(&mut self, other: &Field) -> Result<(), String> {
    if self.datatype != other.datatype {
      Err(format!(
        "Different datatypes: {} != {} for columns {} and {}.",
        self.datatype, other.datatype, self.name, other.name
      ))
    } else if self.arraysize != other.arraysize {
      Err(format!(
        "Different arraysizes: {:?} != {:?} for columns {} and {}.",
        self.arraysize, other.arraysize, self.name, other.name
      ))
    } else {
      // Id
      if let (None, Some(id)) = (&self.id, &other.id) {
        self.id = Some(id.clone());
      }
      // Unit
      if let (None, Some(unit)) = (&self.unit, &other.unit) {
        self.unit = Some(unit.clone());
      }
      // Precision
      if let (None, Some(precision)) = (&self.precision, &other.precision) {
        self.precision = Some(*precision);
      }
      // width
      if let (None, Some(width)) = (&self.width, &other.width) {
        self.width = Some(*width);
      }
      // xtype
      if let (None, Some(xtype)) = (&self.xtype, &other.xtype) {
        self.xtype = Some(xtype.clone());
      }
      // ref
      if let (None, Some(ref_)) = (&self.ref_, &other.ref_) {
        self.ref_ = Some(ref_.clone());
      }
      // ucd
      if let (None, Some(ucd)) = (&self.ucd, &other.ucd) {
        self.ucd = Some(ucd.clone());
      }
      // utype
      if let (None, Some(utype)) = (&self.utype, &other.utype) {
        self.utype = Some(utype.clone());
      }
      // extra
      for (k, v) in &other.extra {
        if !self.extra.contains_key(k) {
          self.extra.insert(k.clone(), v.clone());
        }
      }
      // - description
      if let (None, Some(description)) = (&self.description, &other.description) {
        self.description = Some(description.clone());
      }
      // - values
      if let (None, Some(values)) = (&self.values, &other.values) {
        self.values = Some(values.clone());
      }
      // - links
      let curr_size = self.links.len();
      for l in &other.links {
        if !self.links[0..curr_size].contains(l) {
          self.links.push(l.clone());
        }
      }
      Ok(())
    }
  }

  /// Enrich this Field filling its elements with the ones provided in the given `other` Field.
  /// Returns an error is the datatypes and/or arraysize are not the same.
  /// Elements already defined in thuis Field but also present in `other` are overwritten,
  /// **including the name** (which is mandatory and thus always overwritten).
  pub fn merge_overwrite(&mut self, other: &Field) -> Result<(), String> {
    if self.datatype != other.datatype {
      Err(format!(
        "Different datatypes: {} != {} for columns {} and {}.",
        self.datatype, other.datatype, self.name, other.name
      ))
    } else if self.arraysize != other.arraysize {
      Err(format!(
        "Different arraysizes: {:?} != {:?} for columns {} and {}.",
        self.arraysize, other.arraysize, self.name, other.name
      ))
    } else {
      // Id
      if let Some(id) = &other.id {
        self.id = Some(id.clone());
      }
      self.name = other.name.clone();
      // Unit
      if let Some(unit) = &other.unit {
        self.unit = Some(unit.clone());
      }
      // Precision
      if let Some(precision) = &other.precision {
        self.precision = Some(*precision);
      }
      // width
      if let Some(width) = &other.width {
        self.width = Some(*width);
      }
      // xtype
      if let Some(xtype) = &other.xtype {
        self.xtype = Some(xtype.clone());
      }
      // ref
      if let Some(ref_) = &other.ref_ {
        self.ref_ = Some(ref_.clone());
      }
      // ucd
      if let Some(ucd) = &other.ucd {
        self.ucd = Some(ucd.clone());
      }
      // utype
      if let Some(utype) = &other.utype {
        self.utype = Some(utype.clone());
      }
      // extra
      for (k, v) in &other.extra {
        self.extra.insert(k.clone(), v.clone());
      }
      // - description
      if let Some(description) = &other.description {
        self.description = Some(description.clone());
      }
      // - values
      if let Some(values) = &other.values {
        self.values = Some(values.clone());
      }
      // - links
      let curr_size = self.links.len();
      for l in &other.links {
        if !self.links[0..curr_size].contains(l) {
          self.links.push(l.clone());
        }
      }
      Ok(())
    }
  }
}

impl VOTableElement for Field {
  const TAG: &'static str = "FIELD";

  type MarkerType = HasSubElems;

  fn from_attrs<K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    const DEFAULT_VALUE: &str = "@TBD";
    const DEFAULT_DT: Datatype = Datatype::Logical;
    let mut name_found = false;
    let mut dt_found = false;
    Self::new(DEFAULT_VALUE, DEFAULT_DT)
      .set_attrs(attrs.map(|(k, v)| {
        match k.as_ref() {
          "name" => name_found = true,
          "datatype" => dt_found = true,
          _ => {}
        };
        (k, v)
      }))
      .and_then(|field| {
        if name_found && dt_found {
          Ok(field)
        } else {
          Err(VOTableError::Custom(format!(
            "Attributes 'name' and 'datatype' are mandatory in tag '{}'",
            Self::TAG
          )))
        }
      })
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
        "ID" => self.set_id_by_ref(val),
        "name" => self.set_name_by_ref(val),
        "datatype" => {
          self.set_datatype_by_ref(val.as_ref().parse().map_err(VOTableError::ParseDatatype)?)
        }
        "unit" => self.set_unit_by_ref(val),
        "precision" => {
          if val.as_ref().is_empty() {
            warn!(
              "Emtpy 'precision' attribute in tag {}: attribute ignored",
              Self::TAG
            )
          } else {
            self.set_precision_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?)
          }
        }
        "width" => self.set_width_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?),
        "xtype" => self.set_xtype_by_ref(val),
        "ref" => self.set_ref_by_ref(val),
        "ucd" => self.set_ucd_by_ref(val),
        "utype" => self.set_utype_by_ref(val),
        "arraysize" => {
          self.set_arraysize_by_ref(val.as_ref().parse().map_err(VOTableError::ParseInt)?)
        }
        _ => self.insert_extra_str_by_ref(key, val),
      }
    }
    Ok(())
  }

  /// Calls a closure on each (key, value) attribute pairs.
  fn for_each_attribute<F>(&self, mut f: F)
  where
    F: FnMut(&str, &str),
  {
    if let Some(id) = &self.id {
      f("ID", id.as_str());
    }
    f("name", self.name.as_str());
    f("datatype", self.datatype.to_string().as_str());
    if let Some(arraysize) = &self.arraysize {
      f("arraysize", arraysize.to_string().as_str());
    }
    if let Some(width) = &self.width {
      f("width", width.to_string().as_str());
    }
    if let Some(precision) = &self.precision {
      f("precision", precision.to_string().as_str());
    }
    if let Some(unit) = &self.unit {
      f("unit", unit.as_str());
    }
    if let Some(ucd) = &self.ucd {
      f("ucd", ucd.as_str());
    }
    if let Some(utype) = &self.utype {
      f("utype", utype.as_str());
    }
    if let Some(xtype) = &self.xtype {
      f("xtype", xtype.as_str());
    }
    if let Some(ref_) = &self.ref_ {
      f("ref", ref_.as_str());
    }
    for_each_extra_attribute!(self, f);
  }
}

impl HasSubElements for Field {
  type Context = ();

  fn has_no_sub_elements(&self) -> bool {
    self.description.is_none() && self.values.is_none() && self.links.is_empty()
  }

  fn read_sub_elements_by_ref<R: BufRead>(
    &mut self,
    mut reader: &mut Reader<R>,
    mut reader_buff: &mut Vec<u8>,
    _context: &Self::Context,
  ) -> Result<(), VOTableError> {
    loop {
      let mut event = reader.read_event(reader_buff).map_err(VOTableError::Read)?;
      match &mut event {
        Event::Start(ref e) => match e.local_name() {
          Description::TAG_BYTES => {
            set_from_event_start!(self, Description, reader, reader_buff, e)
          }
          Values::TAG_BYTES => set_from_event_start!(self, Values, reader, reader_buff, e),
          Link::TAG_BYTES => push_from_event_start!(self, Link, reader, reader_buff, e),
          _ => {
            return Err(VOTableError::UnexpectedStartTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Empty(ref e) => match e.local_name() {
          Values::TAG_BYTES => set_from_event_empty!(self, Values, e),
          Link::TAG_BYTES => push_from_event_empty!(self, Link, e),
          _ => {
            return Err(VOTableError::UnexpectedEmptyTag(
              e.local_name().to_vec(),
              Self::TAG,
            ))
          }
        },
        Event::Text(e) if is_empty(e) => {}
        Event::End(e) if e.local_name() == Self::TAG_BYTES => return Ok(()),
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }

  fn write_sub_elements_by_ref<W: Write>(
    &mut self,
    writer: &mut Writer<W>,
    context: &Self::Context,
  ) -> Result<(), VOTableError> {
    write_elem!(self, description, writer, context);
    write_elem!(self, values, writer, context);
    write_elem_vec!(self, links, writer, context);
    Ok(())
  }
}

#[cfg(test)]
mod tests {

  use std::str::FromStr;

  use crate::{
    datatype::Datatype,
    field::ArraySize,
    field::Field,
    tests::{test_read, test_writer},
  };

  #[test]
  fn test_arraysize_from_str() {
    assert_eq!(
      ArraySize::from_str("12"),
      Ok(ArraySize::Fixed1D { size: 12 })
    );
    assert_eq!(ArraySize::from_str("*"), Ok(ArraySize::Variable1D));
    assert_eq!(
      ArraySize::from_str("12*"),
      Ok(ArraySize::VariableWithUpperLimit1D { upper_limit: 12 })
    );
    assert_eq!(
      ArraySize::from_str("12x8"),
      Ok(ArraySize::FixedND { sizes: vec![12, 8] })
    );
    assert_eq!(
      ArraySize::from_str("12x*"),
      Ok(ArraySize::VariableND { sizes: vec![12] })
    );
    assert_eq!(
      ArraySize::from_str("12x8*"),
      Ok(ArraySize::VariableWithUpperLimitND {
        sizes: vec![12],
        upper_limit: 8
      })
    );
    assert_eq!(
      ArraySize::from_str("12x8x15"),
      Ok(ArraySize::FixedND {
        sizes: vec![12, 8, 15]
      })
    );
    assert_eq!(
      ArraySize::from_str("12x8x*"),
      Ok(ArraySize::VariableND { sizes: vec![12, 8] })
    );
    assert_eq!(
      ArraySize::from_str("12x8x15*"),
      Ok(ArraySize::VariableWithUpperLimitND {
        sizes: vec![12, 8],
        upper_limit: 15
      })
    );

    let elems = [
      "12", "*", "12*", "12x8", "12x*", "12x8*", "12x8x15", "12x8x*", "12x8x15*",
    ];
    for elem in elems {
      assert_eq!(ArraySize::from_str(elem).unwrap().to_string(), elem);
    }
  }

  #[test]
  fn test_field_read_write() {
    let xml = r#"<FIELD ID="id" name="nomo" datatype="float" arraysize="5" width="5" precision="1" unit="unit" ucd="UCD" utype="ut" xtype="xt"/>"#; // Test read
    let field = test_read::<Field>(xml);
    // Test read
    assert_eq!(field.id, Some("id".to_string()));
    assert_eq!(field.name, "nomo".to_string());
    assert_eq!(field.datatype, Datatype::Float);
    assert_eq!(field.unit, Some("unit".to_string()));
    let prec = format!("{}", field.precision.as_ref().unwrap());
    assert_eq!(prec, "1");
    assert_eq!(field.width, Some(5));
    assert_eq!(field.xtype, Some("xt".to_string()));
    assert_eq!(field.utype, Some("ut".to_string()));
    assert_eq!(field.ucd, Some("UCD".to_string()));
    // Test write
    test_writer(field, xml)
  }

  #[test]
  fn test_field_read_write_w_desc() {
    let xml = r#"<FIELD name="band" datatype="char" arraysize="*" ucd="instr.bandpass" utype="ssa:DataID.Bandpass"><DESCRIPTION>Description</DESCRIPTION></FIELD>"#;
    let field = test_read::<Field>(xml);
    assert_eq!(
      field.description.as_ref().unwrap().get_content_unwrapped(),
      "Description"
    );
    // Test write
    test_writer(field, xml)
  }

  #[test]
  fn test_field_read_write_w_link() {
    let xml = r#"<FIELD name="band" datatype="char" arraysize="*" ucd="instr.bandpass" utype="ssa:DataID.Bandpass"><LINK href="http://127.0.0.1/"/></FIELD>"#;
    let field = test_read::<Field>(xml);
    assert_eq!(
      field.links.get(0).as_ref().unwrap().href,
      Some("http://127.0.0.1/".to_string())
    );
    // Test write
    test_writer(field, xml)
  }

  #[test]
  fn test_field_read_write_w_val() {
    let xml = r#"<FIELD name="gmag" datatype="float" width="6" precision="3" unit="mag" ucd="phot.mag;em.opt.B"><VALUES null="NaN"/></FIELD>"#;
    let field = test_read::<Field>(xml);
    assert_eq!(field.values.as_ref().unwrap().null, Some("NaN".to_string()));
    // Test write
    test_writer(field, xml)
  }
}
