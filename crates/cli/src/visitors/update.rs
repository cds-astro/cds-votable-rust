//! Write an ASCII representation of the structure of the VOTable in stdout.
use std::{collections::HashMap, str::from_utf8_unchecked};

use log::{trace, warn};

use votable::{
  coosys::{CooSys, CooSysElem},
  data::{fits::Fits, stream::Stream, tabledata::TableData, Data},
  definitions::{Definitions, DefinitionsElem},
  desc::Description,
  field::Field,
  fieldref::FieldRef,
  group::{Group, GroupElem, TableGroup, TableGroupElem},
  info::Info,
  link::Link,
  mivot::visitors::donothing::DoNothing,
  param::Param,
  paramref::ParamRef,
  resource::{Resource, ResourceElem, ResourceOrTable},
  table::Table,
  timesys::TimeSys,
  values::{Max, Min, Opt, Values},
  votable::{VOTable, VOTableElem},
  QuickXmlReadWrite, TableDataContent, TableElem, VOTableElement, VOTableError, VOTableVisitor,
  VoidTableDataContent,
};

use super::{
  super::update::{
    Action::{Rm, SetAttrs, SetContent, SetDesc},
    TagConditionAction,
  },
  Tag,
};

/// Update a VOTable
pub struct UpdateVisitor {
  /// Current Virtual ID
  cur_vid: Vec<u8>,
  /// Current numbers of TAG of each type in the current tag (map keu=Tag.char, value=count)
  cur_counts: Vec<HashMap<u8, u16>>,
  /// Selectors/Modifiers
  updates_by_tag: [Vec<TagConditionAction>; Tag::len()],
  /// Tag (and vid) of a sub-element to be removed from the current tag, in the order they appear in the sub elem
  tagvid_to_rm_stack: Vec<Vec<(Tag, String)>>,
}

impl UpdateVisitor {
  /// Creates and init a new visitor.
  pub fn new(elems: Vec<TagConditionAction>) -> Self {
    let mut updates_by_tag = Tag::new_array_of_vec::<TagConditionAction>();
    for elem in elems {
      updates_by_tag[elem.tag.index()].push(elem);
    }
    trace!("Updates by tag array: {:?}", &updates_by_tag);
    Self {
      cur_vid: Vec::with_capacity(8),
      cur_counts: Vec::with_capacity(8),
      updates_by_tag,
      tagvid_to_rm_stack: Vec::with_capacity(8),
    }
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

  fn get_current_count(&self, tag: Tag) -> usize {
    self
      .cur_counts
      .last()
      .unwrap()
      .get(&tag.char())
      .cloned()
      .unwrap_or(0) as usize
  }

  fn clear_current_counts(&mut self) {
    if let Some(last) = self.cur_counts.last_mut() {
      last.clear();
    }
  }

  /// Get the current Virtual ID
  pub fn get_vid(&self) -> &str {
    unsafe { from_utf8_unchecked(&self.cur_vid) }
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
  /// # Params
  /// * `tag`: the tag we start to visit
  /// * `may_be_repeated`: indicates that the parent `tag` my contains several occurrences of the given `tag`.
  pub fn go_down(&mut self, tag: Tag, may_be_repeated: bool) {
    Self::append_tag_to_vid(&tag, &mut self.cur_vid);
    if may_be_repeated {
      let count = self.add_to_count(&tag);
      Self::append_count_to_vid(count, &mut self.cur_vid);
    }
    // Create the count map of the current element
    self.cur_counts.push(HashMap::with_capacity(20));
  }

  /// Go up in the tag hierarchy:
  /// * removes the last element from the Virtual ID
  /// * removes the last sub-elements count map
  /// * removes an indentation level
  pub fn go_up(&mut self) -> Result<(), VOTableError> {
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

  fn extract_last_digit(vid: &str) -> usize {
    let bytes = vid.as_bytes();
    let to = bytes.len();
    let mut from = to - 1;
    while bytes[from].is_ascii_digit() {
      from -= 1;
    }
    unsafe { from_utf8_unchecked(&bytes[from + 1..to]) }
      .parse::<usize>()
      .unwrap()
  }
}

fn append_to_rm_list(tagvid_to_rm_stack: &mut Vec<Vec<(Tag, String)>>, tag: Tag, vid: String) {
  tagvid_to_rm_stack.last_mut().unwrap().push((tag, vid));
}

fn no_attrs(tag: Tag) -> VOTableError {
  VOTableError::Custom(format!("{} tag does not support 'set_attrs'!", tag))
}

fn no_content(tag: Tag) -> VOTableError {
  VOTableError::Custom(format!("{} tag does not support 'set_content'!", tag))
}

fn no_desc(tag: Tag) -> VOTableError {
  VOTableError::Custom(format!("{} tag does not support 'set_description'!", tag))
}

fn no_rm(tag: Tag) -> VOTableError {
  VOTableError::Custom(format!("{} tag does not support 'rm'!", tag))
}

fn new_descr(s: &String) -> Description {
  Description::from(s.as_str())
}

fn set_attrs<T: VOTableElement>(
  elem: &mut T,
  attrs: &Vec<(String, String)>,
) -> Result<(), VOTableError> {
  elem.set_attrs_by_ref(attrs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
}

impl<C: TableDataContent> VOTableVisitor<C> for UpdateVisitor {
  type E = VOTableError;

  type M = DoNothing<Self::E>;

  fn visit_votable_start(&mut self, votable: &mut VOTable<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::VOTABLE;
    self.go_down(TAG, false);
    let vid = self.get_vid();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid, votable.id.as_ref(), None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(votable, attributes)?,
          SetDesc { new_desc } => votable.reset_description_by_ref(new_descr(new_desc)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => return Err(no_rm(TAG)),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }

  fn visit_votable_ended(&mut self, votable: &mut VOTable<C>) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!(
        "In {}, rm tag {} vid={}",
        votable.tag(),
        tag_to_rm,
        vid_to_rm
      );
      match tag_to_rm {
        Tag::DESCRIPTION => votable.description = None,
        Tag::RESOURCE => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          votable.resources.remove(index);
        }
        _ => {
          self.clear_current_counts();
          // Sub-elems
          let mut rm_index = None;
          for (i, elem) in votable.elems.iter().enumerate() {
            if vid_to_rm
              == match elem {
                VOTableElem::CooSys(_) => self.get_sub_elem_vid(Tag::COOSYS, true),
                VOTableElem::TimeSys(_) => self.get_sub_elem_vid(Tag::TIMESYS, true),
                VOTableElem::Group(_) => self.get_sub_elem_vid(Tag::GROUP, true),
                VOTableElem::Param(_) => self.get_sub_elem_vid(Tag::PARAM, true),
                VOTableElem::Info(_) => self.get_sub_elem_vid(Tag::INFO, true),
                VOTableElem::Definitions(_) => self.get_sub_elem_vid(Tag::DEFINITION, true),
              }
            {
              rm_index = Some(i);
              break;
            }
          }
          if let Some(index) = rm_index {
            votable.elems.remove(index);
          } else {
            // Post_infos
            assert!(matches!(tag_to_rm, Tag::INFO));
            let n_prev_info = self.get_current_count(Tag::INFO);
            let index = Self::extract_last_digit(vid_to_rm.as_str());
            votable.post_infos.remove(index - n_prev_info);
          }
        }
      }
    }
    self.go_up()
  }

  fn visit_description(&mut self, description: &mut Description) -> Result<(), Self::E> {
    const TAG: Tag = Tag::DESCRIPTION;
    let vid = self.get_sub_elem_vid(TAG, false);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { .. } => return Err(no_attrs(TAG)),
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { new_content } => description.set_content_by_ref(new_content),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_coosys_start(&mut self, coosys: &mut CooSys) -> Result<(), Self::E> {
    const TAG: Tag = Tag::COOSYS;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), Some(&coosys.id), None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(coosys, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_coosys_ended(&mut self, coosys: &mut CooSys) -> Result<(), Self::E> {
    // TODO: this is not efficient (we should first create the list of indices to remove and
    // TODO: remove them at once) but the lists are small, so not a real problem so far
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!(
        "In {}, rm tag {} vid={}",
        coosys.tag(),
        tag_to_rm,
        vid_to_rm
      );
      self.clear_current_counts();
      let mut rm_index = None;
      for (i, elem) in coosys.elems.iter().enumerate() {
        if vid_to_rm
          == match elem {
            CooSysElem::FieldRef(_) => self.get_sub_elem_vid(Tag::FIELDRef, true),
            CooSysElem::ParamRef(_) => self.get_sub_elem_vid(Tag::PARAMRef, true),
          }
        {
          rm_index = Some(i);
          break;
        }
      }
      if let Some(index) = rm_index {
        coosys.elems.remove(index);
      }
    }
    self.go_up()
  }

  fn visit_timesys(&mut self, timesys: &mut TimeSys) -> Result<(), Self::E> {
    const TAG: Tag = Tag::TIMESYS;
    let vid = self.get_sub_elem_vid(TAG, true);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), Some(&timesys.id), None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(timesys, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_group_start(&mut self, group: &mut Group) -> Result<(), Self::E> {
    const TAG: Tag = Tag::GROUP;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e
        .condition
        .is_ok(vid.as_str(), group.id.as_ref(), group.name.as_ref())
      {
        match &e.action {
          SetAttrs { attributes } => set_attrs(group, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_group_ended(&mut self, group: &mut Group) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!("In {}, rm tag {} vid={}", group.tag(), tag_to_rm, vid_to_rm);
      match tag_to_rm {
        Tag::DESCRIPTION => group.description = None,
        _ => {
          self.clear_current_counts();
          let mut rm_index = None;
          for (i, elem) in group.elems.iter().enumerate() {
            if vid_to_rm
              == match elem {
                GroupElem::ParamRef(_) => self.get_sub_elem_vid(Tag::PARAMRef, true),
                GroupElem::Param(_) => self.get_sub_elem_vid(Tag::PARAM, true),
                GroupElem::Group(_) => self.get_sub_elem_vid(Tag::GROUP, true),
              }
            {
              rm_index = Some(i);
              break;
            }
          }
          if let Some(index) = rm_index {
            group.elems.remove(index);
          }
        }
      }
    }
    self.go_up()
  }

  fn get_mivot_visitor(&mut self) -> Self::M {
    Self::M::new()
  }

  fn visit_table_group_start(&mut self, group: &mut TableGroup) -> Result<(), Self::E> {
    const TAG: Tag = Tag::GROUP;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e
        .condition
        .is_ok(vid.as_str(), group.id.as_ref(), group.name.as_ref())
      {
        match &e.action {
          SetAttrs { attributes } => set_attrs(group, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_table_group_ended(&mut self, group: &mut TableGroup) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!("In {}, rm tag {} vid={}", group.tag(), tag_to_rm, vid_to_rm);
      match tag_to_rm {
        Tag::DESCRIPTION => group.description = None,
        _ => {
          self.clear_current_counts();
          // Sub-elems
          let mut rm_index = None;
          for (i, elem) in group.elems.iter().enumerate() {
            if vid_to_rm
              == match elem {
                TableGroupElem::FieldRef(_) => self.get_sub_elem_vid(Tag::FIELDRef, true),
                TableGroupElem::ParamRef(_) => self.get_sub_elem_vid(Tag::PARAMRef, true),
                TableGroupElem::Param(_) => self.get_sub_elem_vid(Tag::PARAM, true),
                TableGroupElem::TableGroup(_) => self.get_sub_elem_vid(Tag::GROUP, true),
              }
            {
              rm_index = Some(i);
              break;
            }
          }
          if let Some(index) = rm_index {
            group.elems.remove(index);
          }
        }
      }
    }
    self.go_up()
  }

  fn visit_paramref(&mut self, paramref: &mut ParamRef) -> Result<(), Self::E> {
    const TAG: Tag = Tag::PARAMRef;
    let vid = self.get_sub_elem_vid(TAG, true);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(paramref, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { new_content } => paramref.set_content_by_ref(new_content),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_fieldref(&mut self, fieldref: &mut FieldRef) -> Result<(), Self::E> {
    const TAG: Tag = Tag::FIELDRef;
    let vid = self.get_sub_elem_vid(TAG, true);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(fieldref, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { new_content } => fieldref.set_content_by_ref(new_content),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_param_start(&mut self, param: &mut Param) -> Result<(), Self::E> {
    const TAG: Tag = Tag::PARAM;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(
        vid.as_str(),
        param.field.id.as_ref(),
        Some(&param.field.name),
      ) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(param, attributes)?,
          SetDesc { new_desc } => param.reset_description_by_ref(new_descr(new_desc)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_param_ended(&mut self, param: &mut Param) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!("In {}, rm tag {} vid={}", param.tag(), tag_to_rm, vid_to_rm);
      match tag_to_rm {
        Tag::DESCRIPTION => param.field.description = None, // id already checked going down
        Tag::VALUES => param.field.values = None,           // id already checked going down
        Tag::LINK => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          param.field.links.remove(index);
        }
        _ => {}
      }
    }
    self.go_up()
  }

  fn visit_field_start(&mut self, field: &mut Field) -> Result<(), Self::E> {
    const TAG: Tag = Tag::FIELD;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e
        .condition
        .is_ok(vid.as_str(), field.id.as_ref(), Some(&field.name))
      {
        match &e.action {
          SetAttrs { attributes } => set_attrs(field, attributes)?,
          SetDesc { new_desc } => field.reset_description_by_ref(new_descr(new_desc)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_field_ended(&mut self, field: &mut Field) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!("In {}, rm tag {} vid={}", field.tag(), tag_to_rm, vid_to_rm);
      match tag_to_rm {
        Tag::DESCRIPTION => field.description = None,
        Tag::VALUES => field.values = None,
        Tag::LINK => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          field.links.remove(index);
        }
        _ => {}
      }
    }
    self.go_up()
  }

  fn visit_info(&mut self, info: &mut Info) -> Result<(), Self::E> {
    const TAG: Tag = Tag::INFO;
    let vid = self.get_sub_elem_vid(TAG, true);
    for e in &self.updates_by_tag[TAG.index()] {
      if e
        .condition
        .is_ok(vid.as_str(), info.id.as_ref(), Some(&info.name))
      {
        match &e.action {
          SetAttrs { attributes } => set_attrs(info, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { new_content } => info.set_content_by_ref(new_content),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_definitions_start(&mut self, definitions: &mut Definitions) -> Result<(), Self::E> {
    const TAG: Tag = Tag::DEFINITION;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(definitions, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_definitions_ended(&mut self, definitions: &mut Definitions) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!(
        "In {}, rm tag {} vid={}",
        definitions.tag(),
        tag_to_rm,
        vid_to_rm
      );
      self.clear_current_counts();
      let mut rm_index = None;
      for (i, elem) in definitions.elems.iter().enumerate() {
        if vid_to_rm
          == match elem {
            DefinitionsElem::CooSys(_) => self.get_sub_elem_vid(Tag::COOSYS, true),
            DefinitionsElem::Param(_) => self.get_sub_elem_vid(Tag::PARAM, true),
          }
        {
          rm_index = Some(i);
          break;
        }
      }
      if let Some(index) = rm_index {
        definitions.elems.remove(index);
      }
    }
    self.go_up()
  }

  fn visit_resource_start(&mut self, resource: &mut Resource<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::RESOURCE;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e
        .condition
        .is_ok(vid.as_str(), resource.id.as_ref(), resource.name.as_ref())
      {
        match &e.action {
          SetAttrs { attributes } => set_attrs(resource, attributes)?,
          SetDesc { new_desc } => resource.reset_description_by_ref(new_descr(new_desc)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_resource_ended(&mut self, resource: &mut Resource<C>) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!(
        "In {}, rm tag {} vid={}",
        resource.tag(),
        tag_to_rm,
        vid_to_rm
      );
      match tag_to_rm {
        Tag::DESCRIPTION => resource.description = None,
        // In infos or in sub-elems
        Tag::INFO => {
          let mut index = Self::extract_last_digit(vid_to_rm.as_str());
          let mut infos_len = resource.infos.len();
          if index < infos_len {
            resource.infos.remove(index);
          } else {
            index -= infos_len;
            // is in sub-elements
            for sub in resource.sub_elems.iter_mut() {
              infos_len = sub.infos.len();
              if index < infos_len {
                sub.infos.remove(index);
                break;
              } else {
                index -= infos_len;
              }
            }
          }
        }
        // In resource elems
        Tag::COOSYS | Tag::TIMESYS | Tag::GROUP | Tag::PARAM => {
          self.clear_current_counts();
          // Sub-elems
          let mut rm_index = None;
          for (i, elem) in resource.elems.iter().enumerate() {
            if vid_to_rm
              == match elem {
                ResourceElem::CooSys(_) => self.get_sub_elem_vid(Tag::COOSYS, true),
                ResourceElem::TimeSys(_) => self.get_sub_elem_vid(Tag::TIMESYS, true),
                ResourceElem::Group(_) => self.get_sub_elem_vid(Tag::GROUP, true),
                ResourceElem::Param(_) => self.get_sub_elem_vid(Tag::PARAM, true),
              }
            {
              rm_index = Some(i);
              break;
            }
          }
          if let Some(index) = rm_index {
            resource.elems.remove(index);
          }
        }
        // In resource sub-elems (info already tested before)
        Tag::LINK => {
          self.clear_current_counts();
          let mut index = Self::extract_last_digit(vid_to_rm.as_str());
          for sub in resource.sub_elems.iter_mut() {
            let links_len = sub.links.len();
            if index < links_len {
              sub.infos.remove(index);
              break;
            } else {
              index -= links_len;
            }
          }
        }
        // In resource sub-elems (info and links already tested before)
        Tag::RESOURCE | Tag::TABLE => {
          self.clear_current_counts();
          let mut rm_index = None;
          for (i, sub) in resource.sub_elems.iter().enumerate() {
            if vid_to_rm
              == match sub.resource_or_table {
                ResourceOrTable::Resource(_) => self.get_sub_elem_vid(Tag::RESOURCE, true),
                ResourceOrTable::Table(_) => self.get_sub_elem_vid(Tag::TABLE, true),
              }
            {
              rm_index = Some(i);
              if !sub.infos.is_empty() || !sub.links.is_empty() {
                warn!(
                  "Removing vid='{}', you also remove the associated info(s) and/or link(s).",
                  vid_to_rm
                );
              }
              break;
            }
          }
          if let Some(index) = rm_index {
            resource.sub_elems.remove(index);
          }
        }
        Tag::VODML => resource.vodml = None,
        _ => {}
      }
    }
    self.go_up()
  }

  fn visit_post_info(&mut self, info: &mut Info) -> Result<(), Self::E> {
    <UpdateVisitor as VOTableVisitor<C>>::visit_info(self, info)
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
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), link.id.as_ref(), None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(link, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { new_content } => link.set_content_by_ref(new_content),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_table_start(&mut self, table: &mut Table<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::TABLE;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e
        .condition
        .is_ok(vid.as_str(), table.id.as_ref(), table.name.as_ref())
      {
        match &e.action {
          SetAttrs { attributes } => set_attrs(table, attributes)?,
          SetDesc { new_desc } => table.reset_description_by_ref(new_descr(new_desc)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_table_ended(&mut self, table: &mut Table<C>) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!("In {}, rm tag {} vid={}", table.tag(), tag_to_rm, vid_to_rm);
      match tag_to_rm {
        Tag::DESCRIPTION => table.description = None,
        // Table elems
        Tag::FIELD | Tag::PARAM | Tag::GROUP => {
          self.clear_current_counts();
          let mut rm_index = None;
          for (i, elem) in table.elems.iter().enumerate() {
            if vid_to_rm
              == match elem {
                TableElem::Field(_) => self.get_sub_elem_vid(Tag::FIELD, true),
                TableElem::Param(_) => self.get_sub_elem_vid(Tag::PARAM, true),
                TableElem::TableGroup(_) => self.get_sub_elem_vid(Tag::GROUP, true),
              }
            {
              rm_index = Some(i);
              break;
            }
          }
          if let Some(index) = rm_index {
            table.elems.remove(index);
          }
        }
        Tag::LINK => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          table.links.remove(index);
        }
        Tag::DATA => table.data = None,
        Tag::INFO => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          table.infos.remove(index);
        }
        _ => {}
      }
    }
    self.go_up()
  }

  fn visit_data_start(&mut self, data: &mut Data<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::DATA;
    self.go_down(TAG, false);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(data, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_data_ended(&mut self, _data: &mut Data<C>) -> Result<(), Self::E> {
    self.tagvid_to_rm_stack.pop();
    self.go_up()
  }

  fn visit_tabledata(&mut self, _table: &mut TableData<C>) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_binary_stream(&mut self, stream: &mut Stream<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::STREAM;
    let vid = self.get_sub_elem_vid(TAG, false);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(stream, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => warn!("Content of tag {} cannot be set.", TAG),
          Rm => warn!("Tag {} mandatory in BINARY: it can't be removed.", TAG),
        }
      }
    }
    Ok(())
  }
  fn visit_binary2_stream(&mut self, stream: &mut Stream<C>) -> Result<(), Self::E> {
    const TAG: Tag = Tag::STREAM;
    let vid = self.get_sub_elem_vid(TAG, false);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(stream, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => warn!("Content of tag {} cannot be set.", TAG),
          Rm => warn!("Tag {} mandatory in BINARY2: it can't be removed.", TAG),
        }
      }
    }
    Ok(())
  }
  fn visit_fits_start(&mut self, _fits: &mut Fits) -> Result<(), Self::E> {
    Ok(())
  }
  fn visit_fits_stream(
    &mut self,
    stream: &mut Stream<VoidTableDataContent>,
  ) -> Result<(), Self::E> {
    const TAG: Tag = Tag::STREAM;
    let vid = self.get_sub_elem_vid(TAG, false);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(stream, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => warn!("Content of tag {} cannot be set.", TAG),
          Rm => warn!("Tag {} mandatory in FITS: it can't be removed.", TAG),
        }
      }
    }
    Ok(())
  }
  fn visit_fits_ended(&mut self, _fits: &mut Fits) -> Result<(), Self::E> {
    Ok(())
  }

  fn visit_values_start(&mut self, values: &mut Values) -> Result<(), Self::E> {
    const TAG: Tag = Tag::VALUES;
    self.go_down(TAG, false);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), values.id.as_ref(), None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(values, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_values_min(&mut self, min: &mut Min) -> Result<(), Self::E> {
    const TAG: Tag = Tag::MIN;
    let vid = self.get_sub_elem_vid(TAG, false);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(min, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_values_max(&mut self, max: &mut Max) -> Result<(), Self::E> {
    const TAG: Tag = Tag::MAX;
    let vid = self.get_sub_elem_vid(TAG, false);
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, None) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(max, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    Ok(())
  }

  fn visit_values_opt_start(&mut self, opt: &mut Opt) -> Result<(), Self::E> {
    const TAG: Tag = Tag::OPTION;
    self.go_down(TAG, true);
    let vid = self.get_vid().to_string();
    for e in &self.updates_by_tag[TAG.index()] {
      if e.condition.is_ok(vid.as_str(), None, opt.name.as_ref()) {
        match &e.action {
          SetAttrs { attributes } => set_attrs(opt, attributes)?,
          SetDesc { .. } => return Err(no_desc(TAG)),
          SetContent { .. } => return Err(no_content(TAG)),
          Rm => append_to_rm_list(&mut self.tagvid_to_rm_stack, TAG, vid.clone()),
        }
      }
    }
    self.tagvid_to_rm_stack.push(Default::default());
    Ok(())
  }
  fn visit_values_opt_ended(&mut self, opt: &mut Opt) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!("In {}, rm tag {} vid={}", opt.tag(), tag_to_rm, vid_to_rm);
      match tag_to_rm {
        Tag::OPTION => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          opt.opts.remove(index);
        }
        _ => {}
      }
    }
    self.go_up()
  }
  fn visit_values_ended(&mut self, values: &mut Values) -> Result<(), Self::E> {
    // Going reverse, we do not change the vid of the elements before the ones already removed
    for (tag_to_rm, vid_to_rm) in self.tagvid_to_rm_stack.pop().unwrap().into_iter().rev() {
      trace!(
        "In {}, rm tag {} vid={}",
        values.tag(),
        tag_to_rm,
        vid_to_rm
      );
      match tag_to_rm {
        Tag::MIN => values.min = None,
        Tag::MAX => values.max = None,
        Tag::OPTION => {
          let index = Self::extract_last_digit(vid_to_rm.as_str());
          values.opts.remove(index);
        }
        _ => {}
      }
    }
    self.go_up()
  }
}
