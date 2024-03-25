//! Write an ASCII representation of the structure of the VOTable in stdout.
use std::{collections::HashMap, fmt::Display};

use votable::{
  coosys::CooSys,
  data::{fits::Fits, stream::Stream, tabledata::TableData, Data},
  definitions::Definitions,
  desc::Description,
  field::Field,
  fieldref::FieldRef,
  group::{Group, TableGroup},
  info::Info,
  link::Link,
  mivot::visitors::donothing::DoNothing,
  param::Param,
  paramref::ParamRef,
  resource::Resource,
  table::Table,
  timesys::TimeSys,
  values::{Max, Min, Opt, Values},
  votable::VOTable,
  TableDataContent, VOTableElement, VOTableVisitor, VoidTableDataContent,
};

use super::{StringError, Tag};

/// Indentation unit (2 spaces here).
static INDENT: &str = "  ";
/// String separing 2 `key=value` pairs
static SEP: &str = " ";

struct TagPrinter<'a> {
  line_width: usize,
  indent: &'a str,
  tag: String,
  vid: &'a str,
  attrs: Vec<String>,
  content: Option<&'a str>,
  content_size_min: usize,
}

impl<'a> TagPrinter<'a> {
  /// # Params
  /// * `line_width`: maximum length of a line on the terminal
  /// * `indent`: current indentation (a number of time the space character)
  /// * `tag`: current tag
  /// * `vid`: current Virtual IDentifier
  /// * `content_size_min`:
  fn new(
    mut line_width: usize,
    indent: &'a str,
    tag: Tag,
    vid: &'a str,
    content_size_min: usize,
  ) -> Self {
    line_width -= 1; // for the ending '\n'
    Self {
      line_width,
      indent,
      tag: tag.to_string(),
      vid,
      attrs: Default::default(),
      content: None,
      content_size_min,
    }
  }
  /// Push a mandatory attribute
  fn push_attr_mand<D: Display + ?Sized>(&mut self, key: &str, val: &D) {
    self.attrs.push(format!("{}={}", key, val))
  }
  /// Push an optional attribute
  fn push_attr_opt<D: Display>(&mut self, key: &str, opt_val: &Option<D>) {
    if let Some(val) = opt_val {
      self.push_attr_mand(key, val)
    }
  }

  fn set_content(&mut self, content: &'a str) {
    let _ = self.content.replace(content);
  }

  fn print(mut self) -> Result<(), StringError> {
    let mut s = format!("{}{} vid={}", self.indent, self.tag, self.vid);
    // Add attributes (if any)
    self.attrs.reverse(); // to use pop (while preserving input order), hence limit the number of copy
    while let Some(mut kv) = self.attrs.pop() {
      let mut max_len = self.line_width.saturating_sub(s.len() + SEP.len());
      if kv.len() <= max_len {
        s.push_str(SEP);
        s.push_str(kv.as_str());
      } else {
        // first try to complete with other attributes (size min = 3 because: 'k=v')
        while max_len >= 3 && Self::try_append_kv(max_len, &mut s, &mut self.attrs) {
          max_len = self.line_width.saturating_sub(s.len() + SEP.len());
        }
        println!("{}", s);
        s.clear();
        s.push_str(self.indent);
        s.push_str(INDENT); // add an extra indent level
        max_len = self.line_width.saturating_sub(s.len());
        if kv.len() <= max_len {
          s.push_str(kv.as_str());
        } else {
          // Truncate adding '...' at the end (to know it has been truncated)
          kv.truncate(max_len - 3);
          s.push_str(kv.as_str());
          s.push_str("...");
        }
      }
    }
    // Add content (if any)
    if let Some(content) = self.content {
      let mut content_clean = String::with_capacity(content.len() + 8);
      content_clean.push_str("content=");
      let content = content.trim().replace(['\n'], "\\n");
      let mut word_it = content.split_whitespace();
      if let Some(word) = word_it.next() {
        content_clean.push_str(word);
        for word in word_it {
          content_clean.push(' ');
          content_clean.push_str(word);
        }
      }
      let mut content = content_clean;
      let mut max_len = self.line_width.saturating_sub(s.len() + SEP.len());
      if content.len() <= max_len {
        // Print on the same line, no need to truncate
        s.push_str(SEP);
        s.push_str(content.as_str());
      } else if self.content_size_min <= max_len {
        // Print on the same line, truncating (we add '...' at the end to know it has been truncated)
        content.truncate(max_len - 3);
        s.push_str(SEP);
        s.push_str(content.as_str());
        s.push_str("...");
      } else {
        // Print on a new line (possible truncating)
        println!("{}", s);
        s.clear();
        s.push_str(self.indent);
        s.push_str(INDENT); // add an extra indent level
        max_len = self.line_width.saturating_sub(s.len());
        if content.len() <= max_len {
          // Not truncation needed
          s.push_str(content.as_str());
        } else {
          // Truncate adding '...' at the end to know it has been truncated
          content.truncate(max_len - 3);
          s.push_str(content.as_str());
          s.push_str("...");
        }
      }
    }
    println!("{}", s);
    Ok(())
  }

  /// Returns `true` if succeed in appending a `key=val` elem.
  fn try_append_kv(max_len: usize, line: &mut String, attrs: &mut Vec<String>) -> bool {
    let mut index: Option<usize> = None;
    for (i, kv) in attrs.iter().enumerate().rev() {
      if kv.len() <= max_len {
        index = Some(i);
        break;
      }
    }
    if let Some(index) = index {
      let kv = attrs.remove(index);
      line.push_str("; ");
      line.push_str(kv.as_str());
      true
    } else {
      false
    }
  }
}

/// Write an ASCII representation of the structure of the VOTable in stdout.
pub struct AsciiStructVisitor {
  /// Maximum width of an output line
  line_width: usize,
  /// Smaller possible size of 'content=xxx' beofre putting it on a new line
  content_size_min: usize,
  /// Current indentation
  indent: String,
  /// Current Virtual ID
  cur_vid: Vec<u8>,
  /// Current numbers of TAG of each type in the current tag (map keu=Tag.char, value=count)
  cur_counts: Vec<HashMap<u8, u16>>,
}

impl AsciiStructVisitor {
  /// Creates and init a new visitor.
  pub fn new(line_width: usize, content_size_min: usize) -> Self {
    Self {
      line_width,
      content_size_min,
      indent: String::with_capacity(32),
      cur_vid: Vec::with_capacity(8),
      cur_counts: Vec::with_capacity(8),
    }
  }

  fn increase_indent(&mut self) {
    self.indent.push_str(INDENT);
  }

  fn decrease_indent(&mut self) {
    self.indent.truncate(self.indent.len() - INDENT.len());
  }

  /// Add the current tag to the count map (i.e. number of occurrences of each possible sub-tag)
  /// of the parent tag in the hierarchy, and return its count.
  fn add_to_count(&mut self, tag: &Tag) -> u16 {
    let map = self.cur_counts.last_mut().unwrap();
    let c = tag.char();
    match map.get_mut(&c) {
      Some(count) => {
        *count += 1_u16;
        *count
      }
      None => {
        map.insert(c, 1_u16);
        1_u16
      }
    }
  }

  /// Get the current Virtual ID
  pub fn get_vid(&self) -> &str {
    unsafe { std::str::from_utf8_unchecked(&self.cur_vid) }
  }

  /// Get the Virtual ID of the given tag inside the current tag hierarchy.
  pub fn get_sub_elem_vid(&mut self, tag: Tag, may_be_repeated: bool) -> String {
    let mut vid = self.cur_vid.clone();
    Self::append_tag_to_vid(&tag, &mut vid);
    if may_be_repeated {
      let count = self.add_to_count(&tag);
      Self::append_count_to_vid(count, &mut vid);
    }
    unsafe { String::from_utf8_unchecked(vid) }
  }

  /// Go down in the tag hierarchy, "entering" the given tag:
  /// * add the element to the current Virtual ID
  /// * creates a new sub-elements count map
  /// * add an indentation level.
  /// # Params
  /// * `tag`: the tag we start to visit
  /// * `may_be_repeated`: indicates that the parent `tag` my contains several occurrences of the given `tag`.
  pub fn go_down(&mut self, tag: Tag, may_be_repeated: bool) {
    self.increase_indent();
    Self::append_tag_to_vid(&tag, &mut self.cur_vid);
    if may_be_repeated {
      let count = self.add_to_count(&tag);
      Self::append_count_to_vid(count, &mut self.cur_vid);
    }
    // Create the count map of the current element
    self.cur_counts.push(HashMap::with_capacity(20))
  }

  /// Go up in the tag hierarchy:
  /// * removes the last element from the Virtual ID
  /// * removes the last sub-elements count map
  /// * removes an indentation level
  pub fn go_up(&mut self) -> Result<(), StringError> {
    self.decrease_indent();
    // Remove the last count (if any) and the last tag
    while let Some(c) = self.cur_vid.pop() {
      // 48 to 57 is the decimal range value of the ASCII chars 0 to 9
      if !(48_u8..=57_u8).contains(&c) {
        break;
      }
    }
    self.cur_counts.pop();
    Ok(())
  }

  fn append_tag_to_vid(tag: &Tag, vid: &mut Vec<u8>) {
    vid.push(tag.char());
  }

  /// Append the given count to the given Virtual ID.
  fn append_count_to_vid(count: u16, vid: &mut Vec<u8>) {
    if count < 10 {
      // +48 to transform the u8 in [0,9] into its ASCII value
      vid.push(count as u8 + 48);
    } else {
      for c in count.to_string().as_bytes() {
        vid.push(*c);
      }
    }
  }
}

impl<C: TableDataContent> VOTableVisitor<C> for AsciiStructVisitor {
  type E = StringError;

  type M = DoNothing<Self::E>;

  fn visit_votable_start(&mut self, votable: &mut VOTable<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::VOTABLE;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    tagp.push_attr_opt("ID", &votable.id);
    tagp.print()
  }

  fn visit_votable_ended(&mut self, _votable: &mut VOTable<C>) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_description(&mut self, description: &mut Description) -> Result<(), Self::E> {
    const TAG: Tag = Tag::DESCRIPTION;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    tagp.set_content(description.get_content_unwrapped());
    tagp.print()
  }

  fn visit_coosys_start(&mut self, coosys: &mut CooSys) -> Result<(), Self::E> {
    const TAG: Tag = Tag::COOSYS;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    coosys.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_coosys_ended(&mut self, _coosys: &mut CooSys) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_timesys(&mut self, timesys: &mut TimeSys) -> Result<(), Self::E> {
    const TAG: Tag = Tag::TIMESYS;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    timesys.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }

  fn visit_group_start(&mut self, group: &mut Group) -> Result<(), Self::E> {
    const TAG: Tag = Tag::GROUP;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    group.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_group_ended(&mut self, _group: &mut Group) -> Result<(), Self::E> {
    self.go_up()
  }

  fn get_mivot_visitor(&mut self) -> Self::M {
    Self::M::new()
  }

  fn visit_table_group_start(&mut self, group: &mut TableGroup) -> Result<(), Self::E> {
    const TAG: Tag = Tag::GROUP;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    group.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_table_group_ended(&mut self, _group: &mut TableGroup) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_paramref(&mut self, paramref: &mut ParamRef) -> Result<(), Self::E> {
    const TAG: Tag = Tag::PARAMRef;
    let vid = self.get_sub_elem_vid(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    paramref.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    if let Some(content) = &paramref.content {
      tagp.set_content(content.as_str())
    }
    tagp.print()
  }

  fn visit_fieldref(&mut self, fieldref: &mut FieldRef) -> Result<(), Self::E> {
    const TAG: Tag = Tag::FIELDRef;
    let vid = self.get_sub_elem_vid(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    fieldref.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    if let Some(content) = &fieldref.content {
      tagp.set_content(content.as_str())
    }
    tagp.print()
  }

  fn visit_param_start(&mut self, param: &mut Param) -> Result<(), Self::E> {
    const TAG: Tag = Tag::PARAM;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    param.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_param_ended(&mut self, _param: &mut Param) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_field_start(&mut self, field: &mut Field) -> Result<(), Self::E> {
    const TAG: Tag = Tag::FIELD;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    field.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_field_ended(&mut self, _field: &mut Field) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_info(&mut self, info: &mut Info) -> Result<(), Self::E> {
    const TAG: Tag = Tag::INFO;
    let vid = self.get_sub_elem_vid(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    info.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    if let Some(content) = &info.content {
      tagp.set_content(content.as_str())
    }
    tagp.print()
  }

  fn visit_definitions_start(&mut self, _coosys: &mut Definitions) -> Result<(), Self::E> {
    const TAG: Tag = Tag::DEFINITION;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    tagp.print()
  }
  fn visit_definitions_ended(&mut self, _coosys: &mut Definitions) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_resource_start(&mut self, resource: &mut Resource<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::RESOURCE;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    resource.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_resource_ended(&mut self, _resource: &mut Resource<C>) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_post_info(&mut self, info: &mut Info) -> Result<(), Self::E> {
    <AsciiStructVisitor as VOTableVisitor<C>>::visit_info(self, info)
  }

  fn visit_resource_sub_elem_start(&mut self) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_resource_sub_elem_ended(&mut self) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_link(&mut self, link: &mut Link) -> Result<(), Self::E> {
    const TAG: Tag = Tag::LINK;
    let vid = self.get_sub_elem_vid(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    link.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    if let Some(content) = &link.content {
      tagp.set_content(content.as_str())
    }
    tagp.print()
  }

  fn visit_table_start(&mut self, table: &mut Table<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::TABLE;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    table.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_table_ended(&mut self, _table: &mut Table<C>) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_data_start(&mut self, _data: &mut Data<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::DATA;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, false);
    let tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    tagp.print()
  }
  fn visit_data_ended(&mut self, _data: &mut Data<C>) -> Result<(), Self::E> {
    self.go_up()
  }

  fn visit_tabledata(&mut self, _table: &mut TableData<C>) -> Result<(), Self::E> {
    println!("{}TABLEDATA", self.indent);
    Ok(())
  }
  fn visit_binary_stream(&mut self, stream: &mut Stream<C>) -> Result<(), Self::E> {
    println!("{}BINARY", self.indent);
    self.increase_indent();
    const TAG: Tag = Tag::STREAM;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    stream.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print().unwrap(); // Ok since 'print' always returns 'Ok(())'
    self.decrease_indent();
    Ok(())
  }
  fn visit_binary2_stream(&mut self, stream: &mut Stream<C>) -> Result<(), Self::E> {
    println!("{}BINARY2", self.indent);
    self.increase_indent();
    const TAG: Tag = Tag::STREAM;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    stream.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print().unwrap(); // Ok since 'print' always returns 'Ok(())'
    self.decrease_indent();
    Ok(())
  }
  fn visit_fits_start(&mut self, fits: &mut Fits) -> Result<(), Self::E> {
    match fits.extnum {
      None => println!("{}FITS", self.indent),
      Some(extnum) => println!("{}FITS extnum={}", self.indent, extnum),
    }
    self.increase_indent();
    Ok(())
  }
  fn visit_fits_stream(
    &mut self,
    stream: &mut Stream<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    const TAG: Tag = Tag::STREAM;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    stream.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_fits_ended(&mut self, _fits: &mut Fits) -> Result<(), Self::E> {
    self.decrease_indent();
    Ok(())
  }

  fn visit_values_start(&mut self, values: &mut Values) -> Result<(), Self::E> {
    const TAG: Tag = Tag::VALUES;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    values.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_values_min(&mut self, min: &mut Min) -> Result<(), Self::E> {
    const TAG: Tag = Tag::MIN;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    min.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }

  fn visit_values_max(&mut self, max: &mut Max) -> Result<(), Self::E> {
    const TAG: Tag = Tag::MAX;
    let vid = self.get_sub_elem_vid(TAG, false);
    let mut tagp = TagPrinter::new(
      self.line_width,
      self.indent.as_str(),
      TAG,
      vid.as_str(),
      self.content_size_min,
    );
    max.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }

  fn visit_values_opt_start(&mut self, opt: &mut Opt) -> Result<(), Self::E> {
    const TAG: Tag = Tag::OPTION;
    let curr_indent = self.indent.clone();
    self.go_down(TAG, true);
    let mut tagp = TagPrinter::new(
      self.line_width,
      curr_indent.as_str(),
      TAG,
      self.get_vid(),
      self.content_size_min,
    );
    opt.for_each_attribute(|key: &str, val: &str| tagp.push_attr_mand(key, val));
    tagp.print()
  }
  fn visit_values_opt_ended(&mut self, _opt: &mut Opt) -> Result<(), Self::E> {
    self.go_up()
  }
  fn visit_values_ended(&mut self, _values: &mut Values) -> Result<(), Self::E> {
    self.go_up()
  }
}
