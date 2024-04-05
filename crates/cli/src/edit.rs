use std::{
  fs::File,
  io::{stdin, stdout, BufRead, BufReader, BufWriter, Write},
  iter::Peekable,
  str::FromStr,
};

use clap::Args;

use votable::{
  data::TableOrBinOrBin2, error::VOTableError, iter::SimpleVOTableRowIterator,
  votable::new_xml_writer, CooSys, Description, Field, FieldRef, Group, Info, Link, Max, Min, Opt,
  Param, ParamRef, Resource, TableDataContent, TableGroup, TimeSys, VOTableElement, Values,
  VoidTableDataContent,
};

use super::{
  input::Input,
  output::{Output, OutputFormat},
  visitors::{update::UpdateVisitor, Tag},
  wrappedelems::{VOTableWrappedElem, VOTableWrappedElemMut},
};

/// Edit metadata adding/removing/updating attributes and/or elements.
#[derive(Debug, Args)]
pub struct Edit {
  #[command(flatten)]
  input: Input,
  #[command(flatten)]
  output: Output,
  /// List of "TAG CONDITION ACTION ARGS", e.g.:
  /// -e 'INFO name=Target rm' -e 'FIELD ID=RA set_attrs ucd=pos.eq.ra;meta.main unit=deg'
  /// CONDITIONS:
  ///   name=VAL  name (if any) equals a given value
  ///     id=VAL  id (if any) equals a given value
  ///    vid=VAL  virtual id equals a given value
  /// ACTIONS ARGS:
  ///   rm                                                 Remove the TAG
  ///   set_attrs        KEY=VAL (KEY=VAL) ...               Set TAG attributes
  ///   set_content      CONTENT                             Set the content for `DESCRIPTION`, `INFO`, `LINK`, `PARAMRef` or `FIELDRef`
  ///   set_desc         DESC                                Set the `DESCRIPTION` for `VOTABLE`, `RESOURCE`, `TABLE`, `FIELD`, `PARAM` or `GROUP`
  ///   push_timesys     KEY=VAL (KEY=VAL) ...               Push a new `TIMESYS` in `VOTABLE` or `RESOURCE`
  ///   set_min          KEY=VAL (KEY=VAL) ...               Set a new `MIN` for `VALUES`.
  ///   set_max          KEY=VAL (KEY=VAL) ...               Set a new `MAX` for `VALUES`.
  ///   push_option      KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `OPTION` in `VALUES` or `OPTION`
  ///   set_values       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Set the new `VALUES` for `FIELD` or `PARAM`
  ///   push_info        KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `INFO` in `VOTABLE`, `RESOURCE` or `TABLE`.
  ///   push_post_info   KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new post-`INFO` in `VOTABLE`.
  ///   push_link        KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `LINK` in  `RESOURCE`, `TABLE`, `FIELD` or `PARAM`.
  ///   push_fieldref    KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `FIELDRef` in `COOSYS` or table-`GROUP`.
  ///   push_paramref    KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `PARAMRef` in `COOSYS` or `GROUP`.
  ///   push_coosys      KEY=VAL (KEY=VAL) ... (SUB_ACTION)  Push the new `COOSYS` in `VOTABLE` or `RESOURCE`.
  ///   push_group       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `GROUP` in `VOTABLE` or `RESOURCE`.
  ///   push_tablegroup  KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `GROUP` in `TABLE`.
  ///   push_param       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `PARAM` in `VOTABLE`, `RESOURCE`, `TABLE` or `GROUP`.
  ///   push_field       KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `FIELD` in `TABLE`.
  ///   prepend_resource KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Preprend the new `RESOURCE` in `VOTABLE` or `RESOURCE`.
  ///   push_resource    KEY=VAL (KEY=VAL) ... (SUB_ACTIONS) Push the new `RESOURCE` in `VOTABLE` or `RESOURCE`.
  /// SUB ACTIONS:
  ///   Sub-actions are the same as the ACTIONS (except `rm` which is not allowed).
  ///   A sub-action stars with a `@`. Actually one can see `@` as an action separator with the
  ///   main action at the left of the first `@` and all other actions being sub-actions applying on
  ///   the element created by the main action (the parent element).
  ///   E.g, in:
  ///     `push_param KEY=VAL ... @set_description ... @push_link ...`
  ///   Both `set_description` an `push_link` are executed on the new `PARAM` built by `push_param`.
  ///   For sub-actions to be executed on the last created element, you can double once the `@`:
  ///     `push_param KEY=VAL ... @set_description ... @push_link ... @@set_content CONTENT`
  ///   Here `set_content` will be applied on the new `LINK` before pushing it in the new `PARAM`.
  ///   After a `@@`, all sub-commands are executed on the last created element.
  ///   To go up from one level in the element hierarchy, use `@<`:
  ///     `push_param KEY=VAL ... @set_description ... @push_link ... @@set_content CONTENT @< @push_link ...`
  ///   You can use arbitrary deeply nested elements using `@@` and `@<`.
  ///   Those three commands do not lead to the same hierarchy:
  ///     `push_group ... @push_group ... @push_group @@push_group @push_group (@<)`
  ///     `push_group ... @push_group ... @push_group @@push_group @@push_group (@<@<)`
  ///     `push_group ... @push_group ... @push_group @@push_group @< @push_group`
  ///   Remark: `@@xxx` is a short version of `@> @xxx`.
  #[arg(short = 'e', long = "edit", verbatim_doc_comment)]
  elems: Vec<TagConditionAction>,
  /// Use streaming mode: only for large XML files with a single table, and if the input format
  /// is the same as the output format.
  #[arg(short, long)]
  streaming: bool,
}

impl Edit {
  pub fn exec(self) -> Result<(), VOTableError> {
    if self.streaming {
      if self.input.is_streamable()? {
        self.choose_input_and_exec()
      } else {
        Err(VOTableError::Custom(
          "Only the 'xml' input format is supporting with option '--streaming'.".into(),
        ))
      }
    } else {
      self
        .input
        .load()
        .and_then(|vot| {
          let mut visitor = UpdateVisitor::new(self.elems);
          let mut vot = vot.unwrap();
          vot
            .visit(&mut visitor)
            .map_err(|e| VOTableError::Custom(e.to_string()))
            .map(|_| vot.wrap())
        })
        .and_then(|vot| self.output.save(vot))
    }
  }

  pub fn choose_input_and_exec(self) -> Result<(), VOTableError> {
    match &self.input.input {
      Some(path) => {
        SimpleVOTableRowIterator::from_file(path).and_then(|it| self.choose_output_and_exec(it))
      }
      None => {
        let stdin = stdin();
        // let handle = stdin.lock();
        SimpleVOTableRowIterator::from_reader(BufReader::new(stdin))
          .and_then(|it| self.choose_output_and_exec(it))
      }
    }
  }

  pub fn choose_output_and_exec<R: BufRead + Send>(
    self,
    it: SimpleVOTableRowIterator<R>,
  ) -> Result<(), VOTableError> {
    match &self.output.output {
      Some(path) => {
        let file = File::create(path).map_err(VOTableError::Io)?;
        let write = BufWriter::new(file);
        self.do_exec_gen(it, write)
      }
      None => {
        let stdout = stdout();
        let handle = stdout.lock();
        self.do_exec_gen(it, handle)
      }
    }
  }

  pub fn do_exec_gen<R, W>(
    self,
    it: SimpleVOTableRowIterator<R>,
    write: W,
  ) -> Result<(), VOTableError>
  where
    R: BufRead + Send,
    W: Write,
  {
    let visitor = UpdateVisitor::new(self.elems);
    match (it.data_type(), self.output.output_fmt) {
      (TableOrBinOrBin2::TableData, OutputFormat::XmlTabledata) => to_same(it, write, visitor),
      (TableOrBinOrBin2::Binary, OutputFormat::XmlBinary) => to_same(it, write, visitor),
      (TableOrBinOrBin2::Binary2, OutputFormat::XmlBinary2) => to_same(it, write, visitor),
      _ => Err(VOTableError::Custom(format!(
        "Input format '{:?}' not compatible with output format '{:?}'",
        it.data_type(),
        self.output.output_fmt
      ))),
    }
  }
}

fn to_same<R: BufRead, W: Write>(
  mut it: SimpleVOTableRowIterator<R>,
  write: W,
  mut visitor: UpdateVisitor,
) -> Result<(), VOTableError> {
  let mut writer = new_xml_writer(write, None, None);
  let mut vot = it.votable.clone();
  vot.visit(&mut visitor)?; // Modif and write the cloned version to avoid destructive modifications
  if vot.write_to_data_beginning(&mut writer, &(), false)? {
    it.copy_remaining_data(&mut writer.inner())
      .and_then(|_| it.read_to_end())
      .and_then(|mut out_vot| {
        out_vot
          .visit(&mut visitor) // Re-visit the full VOTable before writting its tail
          .and_then(|()| out_vot.write_from_data_end(&mut writer, &(), false))
      })
  } else {
    // No table in the VOTable
    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct TagConditionAction {
  pub tag: Tag,
  pub condition: Condition,
  pub action: Action,
}

impl FromStr for TagConditionAction {
  type Err = VOTableError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim_start().split_once(|c: char| c.is_whitespace()) {
      Some((tag, rem)) => Tag::from_str(tag).and_then(|tag| {
        match rem.trim_start().split_once(|c: char| c.is_whitespace()) {
          Some((condition, action)) => Condition::from_str(condition).and_then(|condition| {
            Action::from_str(action.trim_start()).and_then(|action| {
              if action.is_compatible_with(&tag) {
                Ok(Self {
                  tag,
                  condition,
                  action,
                })
              } else {
                Err(VOTableError::Custom(format!(
                  "Action {:?} not compatible with tag {}",
                  &action, &tag
                )))
              }
            })
          }),
          None => Err(VOTableError::Custom(format!(
            "No whitespace to spit on in '{}'",
            &rem
          ))),
        }
      }),
      None => Err(VOTableError::Custom(format!(
        "No whitespace to spit on in '{}'",
        s
      ))),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Condition {
  /// `vid=VID`: element match the given Virtual ID
  VirtualIdEq(String),
  /// `id=ID`: element match the given ID
  IdEq(String),
  /// `name=NAME`: element match the given name
  NameEq(String),
}
impl Condition {
  pub fn is_ok(&self, vid: &str, id: Option<&String>, name: Option<&String>) -> bool {
    let res = match self {
      Self::VirtualIdEq(s) => vid == s.as_str(),
      Self::IdEq(s) => id.map(|id| id.as_str() == s.as_str()).unwrap_or(false),
      Self::NameEq(s) => name
        .map(|name| name.as_str() == s.as_str())
        .unwrap_or(false),
    };
    res
  }
}
impl FromStr for Condition {
  type Err = VOTableError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.split_once('=') {
      Some((key, val)) => match key {
        "vid" => Ok(Self::VirtualIdEq(val.to_string())),
        "id" => Ok(Self::IdEq(val.to_string())),
        "name" => Ok(Self::NameEq(val.to_string())),
        _ => Err(VOTableError::Custom(format!(
          "Unrecognized condition. Actual: {}. Expected: one of [vid, id, name].",
          key
        ))),
      },
      None => Err(VOTableError::Custom(format!(
        "No '=' to spit on in '{}'",
        s
      ))),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Action {
  /// `rm`: remove
  Rm,
  /// `set_attrs KEY=VALUE KEY=VALUE`: set attributes like the name
  SetAttrs { attributes: Vec<(String, String)> },
  /// `set_content STRING` set the content for `DESCRIPTION`, `INFO`, `LINK`, `PARAMRef` and `FIELDRef`.
  SetContent { new_content: String },
  /// `set_desc STRING` set the `DESCRIPTION` for `VOTABLE`, `RESOURCE`, `TABLE`, `FIELD`, 'PARAM' and `GROUP`.
  SetDesc { new_desc: String },
  /// `push_timesys KEY=VAL KEY=VAL ...` push the new `TIMESYS` in `VOTABLE` or `RESOURCE`.
  PushTimeSys { timesys: TimeSys },
  /// `set_min KEY=VAL ...` set the new `MIN` in `VALUES`.
  SetMin { min: Min },
  /// `set_min KEY=VAL ...` set the new `MAX` in `VALUES`.
  SetMax { max: Max },
  /// `push_option KEY=VAL ... @push_option ...` push the new `OPTION` in `VALUES` or `OPTION`.
  PushOption { option: Opt },
  /// `set_values KEY=VAL ... @set_min @set_max @set_option` set the new `VALUES` in `FIELD` or `PARAM`.
  SetValues { values: Values },
  /// `push_info KEY=VAL KEY=VAL ... @set_content CONTENT` push the new `INFO` in `VOTABLE`, `RESOURCE` or `TABLE`.
  PushInfo { info: Info },
  /// `push_post_info KEY=VAL KEY=VAL ... @set_content CONTENT` push the new post-`INFO` in `VOTABLE`.
  PushPostInfo { info: Info },
  /// `push_link KEY=VAL KEY=VAL ... @set_content CONTENT` push the new `LINK` in  `RESOURCE`,  `TABLE`, `FIELD` or `PARAM`.
  PushLink { link: Link },
  /// `push_fieldref KEY=VAL KEY=VAL ... @set_content CONTENT` push the new `FIELDRef` in  `COOSYS` or table-`GROUP`.
  PushFielRef { fieldref: FieldRef },
  /// `push_paramref KEY=VAL KEY=VAL ... @set_content CONTENT` push the new `PARAMRef` in  `COOSYS` or `GROUP`.
  PushParamRef { paramref: ParamRef },
  /// `push_coosys KEY=VAL KEY=VAL ... @push_fieldref ... @@set_content ... @<  ...` push the new `COOSYS` in `VOTABLE` or `RESOURCE`.
  PushCooSys { coosys: CooSys },
  /// `push_group KEY=VAL ... @push_group ... @@push_group @< @push_group ...` push the new `GROUP` in `VOTABLE` or `RESOURCE`.
  PushGroup { group: Group },
  /// `push_tablegroup KEY=VAL ...` push the new `GROUP` in `TABLE`.
  PushTableGroup { group: TableGroup },
  /// `push_param KEY=VAL ... @set_description ... @push_link ... @@set_content ... @< ...` push the new `PARAM` in `VOTABLE`, `RESOURCE`, `TABLE` or `GROUP`.
  PushParam { param: Param },
  /// `push_field KEY=VAL ... @set_description ... @push_link ... @@set_content ... @< ...` push the new `FIELD` in `TABLE`.
  PushField { field: Field },
  /// `prepend_resource KEY=VAL ... @set_... @push...` preprend the new `RESOURCE` in `VOTABLE` or `RESOURCE`.
  PrependResource {
    resource: Resource<VoidTableDataContent>,
  },
  /// `push_resource KEy=VAL ... @set_... @push...` push the new `RESOURCE` in `VOTABLE` or `RESOURCE`.
  PushResource {
    resource: Resource<VoidTableDataContent>,
  },
  // FITS, STREAM, ...
}
impl Action {
  fn str2map(s: &str) -> Vec<(String, String)> {
    let mut kv: Vec<(String, String)> = Vec::new();
    let mut it = s.split('=');
    if let Some(first_key) = it.next() {
      let mut key = first_key.trim();
      while let Some(rem) = it.next() {
        if let Some((val, next_key)) = rem.rsplit_once(' ') {
          kv.push((key.to_string(), val.trim().to_string()));
          key = next_key.trim();
        } else if !it.next().is_none() {
          panic!("Unable to parse value and key in {}", rem)
        } else {
          kv.push((key.to_string(), rem.trim().to_string()));
        }
      }
    }
    kv
  }

  pub fn is_compatible_with(&self, tag: &Tag) -> bool {
    match self {
      Action::Rm => !matches!(tag, Tag::VOTABLE),
      Action::SetAttrs { .. } => !matches!(tag, Tag::DEFINITION),
      Action::SetContent { .. } => {
        matches!(tag, Tag::INFO | Tag::LINK | Tag::PARAMRef | Tag::FIELDRef)
      }
      Action::SetDesc { .. } => matches!(
        tag,
        Tag::VOTABLE | Tag::RESOURCE | Tag::TABLE | Tag::FIELD | Tag::PARAM | Tag::GROUP
      ),
      Action::PushTimeSys { .. } => matches!(tag, Tag::VOTABLE | Tag::RESOURCE),
      Action::SetMin { .. } => matches!(tag, Tag::VALUES),
      Action::SetMax { .. } => matches!(tag, Tag::VALUES),
      Action::PushOption { .. } => matches!(tag, Tag::VALUES | Tag::OPTION),
      Action::SetValues { .. } => matches!(tag, Tag::FIELD | Tag::PARAM),
      Action::PushInfo { .. } => matches!(tag, Tag::VOTABLE | Tag::RESOURCE | Tag::TABLE),
      Action::PushPostInfo { .. } => matches!(tag, Tag::VOTABLE),
      Action::PushLink { .. } => {
        matches!(tag, Tag::RESOURCE | Tag::TABLE | Tag::FIELD | Tag::PARAM)
      }
      Action::PushFielRef { .. } => matches!(tag, Tag::COOSYS | Tag::GROUP),
      Action::PushParamRef { .. } => matches!(tag, Tag::COOSYS | Tag::GROUP),
      Action::PushCooSys { .. } => matches!(tag, Tag::VOTABLE | Tag::RESOURCE),
      Action::PushGroup { .. } => matches!(tag, Tag::VOTABLE | Tag::RESOURCE),
      Action::PushTableGroup { .. } => matches!(tag, Tag::TABLE),
      Action::PushParam { .. } => {
        matches!(tag, Tag::VOTABLE | Tag::RESOURCE | Tag::TABLE | Tag::GROUP)
      }
      Action::PushField { .. } => matches!(tag, Tag::TABLE),
      Action::PrependResource { .. } => matches!(tag, Tag::VOTABLE | Tag::RESOURCE),
      Action::PushResource { .. } => matches!(tag, Tag::VOTABLE | Tag::RESOURCE),
    }
  }

  pub fn apply_on_wrapped_elem<C: TableDataContent>(
    self,
    elem: &mut VOTableWrappedElem<C>,
  ) -> Result<(), VOTableError> {
    self.apply_on_wrapped_elem_mut(elem.as_mut())
  }

  pub fn apply_on_wrapped_elem_mut<C: TableDataContent>(
    self,
    elem: VOTableWrappedElemMut<C>,
  ) -> Result<(), VOTableError> {
    match self {
      Action::Rm => Err(VOTableError::Custom(
        "'rm' action cannot be used in this context".into(),
      )),
      Action::SetAttrs { attributes } => elem.set_attributes(attributes.into_iter()),
      Action::SetContent { new_content } => elem.set_content(new_content),
      Action::SetDesc { new_desc } => elem.set_description(Description::new(new_desc)),
      Action::PushTimeSys { timesys } => elem.push_timesys(timesys),
      Action::SetMin { min } => elem.set_min(min),
      Action::SetMax { max } => elem.set_max(max),
      Action::PushOption { option } => elem.push_option(option),
      Action::SetValues { values } => elem.set_values(values),
      Action::PushInfo { info } => elem.push_info(info),
      Action::PushPostInfo { info } => elem.push_post_info(info),
      Action::PushLink { link } => elem.push_link(link),
      Action::PushFielRef { fieldref } => elem.push_fieldref(fieldref),
      Action::PushParamRef { paramref } => elem.push_paramref(paramref),
      Action::PushCooSys { coosys } => elem.push_coosys(coosys),
      Action::PushGroup { group } => elem.push_group(group),
      Action::PushTableGroup { group } => elem.push_tablegroup(group),
      Action::PushParam { param } => elem.push_param(param),
      Action::PushField { field } => elem.push_field(field),
      Action::PrependResource { resource } => {
        elem.prepend_resource(Resource::from_void_table_data_content(resource))
      }
      Action::PushResource { resource } => {
        elem.push_resource(Resource::from_void_table_data_content(resource))
      }
    }
  }

  ///
  /// # Params
  /// * `sub`: current sub-action
  /// * `it`: iterator over the `@` separated sub-actions of the total action
  pub fn from_sub_action<'a, I>(
    current: &'a str,
    it: Peekable<I>,
    first_elem: bool,
  ) -> Result<(Self, Peekable<I>), VOTableError>
  where
    I: Iterator<Item = &'a str>,
  {
    match current.split_once(|c: char| c.is_whitespace()) {
      Some((action, args)) => {
        match action {
          "set_attrs" => Ok((
            Self::SetAttrs {
              attributes: Action::str2map(args.trim_end()),
            },
            it,
          )),
          "set_content" => Ok((
            Self::SetContent {
              new_content: args.to_string(),
            },
            it,
          )),
          "set_description" => Ok((
            Self::SetDesc {
              new_desc: args.to_string(),
            },
            it,
          )),
          "push_timesys" => Self::create_generic_elem::<_, TimeSys>(args, it, first_elem).and_then(
            |(wrapped, it)| {
              wrapped
                .time_sys()
                .map(|timesys| (Self::PushTimeSys { timesys }, it))
            },
          ),
          "set_min" => Self::create_generic_elem::<_, Min>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.min().map(|min| (Self::SetMin { min }, it))),
          "set_max" => Self::create_generic_elem::<_, Max>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.max().map(|max| (Self::SetMax { max }, it))),
          "push_option" => {
            Self::create_generic_elem::<_, Opt>(args, it, first_elem).and_then(|(wrapped, it)| {
              wrapped
                .option()
                .map(|option| (Self::PushOption { option }, it))
            })
          }
          "set_values" => Self::create_generic_elem::<_, Values>(args, it, first_elem).and_then(
            |(wrapped, it)| {
              wrapped
                .values()
                .map(|values| (Self::SetValues { values }, it))
            },
          ),
          "push_info" => Self::create_generic_elem::<_, Info>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.info().map(|info| (Self::PushInfo { info }, it))),
          "push_post_info" => Self::create_generic_elem::<_, Info>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.info().map(|info| (Self::PushPostInfo { info }, it))),
          "push_link" => Self::create_generic_elem::<_, Link>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.link().map(|link| (Self::PushLink { link }, it))),
          "push_fieldref" => Self::create_generic_elem::<_, FieldRef>(args, it, first_elem)
            .and_then(|(wrapped, it)| {
              wrapped
                .field_ref()
                .map(|fieldref| (Self::PushFielRef { fieldref }, it))
            }),
          "push_paramref" => Self::create_generic_elem::<_, ParamRef>(args, it, first_elem)
            .and_then(|(wrapped, it)| {
              wrapped
                .param_ref()
                .map(|paramref| (Self::PushParamRef { paramref }, it))
            }),
          "push_coosys" => Self::create_generic_elem::<_, CooSys>(args, it, first_elem).and_then(
            |(wrapped, it)| {
              wrapped
                .coo_sys()
                .map(|coosys| (Self::PushCooSys { coosys }, it))
            },
          ),
          "push_group" => Self::create_generic_elem::<_, Group>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.group().map(|group| (Self::PushGroup { group }, it))),
          "push_tablegroup" => Self::create_generic_elem::<_, TableGroup>(args, it, first_elem)
            .and_then(|(wrapped, it)| {
              wrapped
                .table_group()
                .map(|group| (Self::PushTableGroup { group }, it))
            }),
          "push_param" => Self::create_generic_elem::<_, Param>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.param().map(|param| (Self::PushParam { param }, it))),
          "push_field" => Self::create_generic_elem::<_, Field>(args, it, first_elem)
            .and_then(|(wrapped, it)| wrapped.field().map(|field| (Self::PushField { field }, it))),
          "prepend_resource" => {
            Self::create_generic_elem::<_, Resource<VoidTableDataContent>>(args, it, first_elem)
              .and_then(|(wrapped, it)| {
                wrapped
                  .resource()
                  .map(|resource| (Self::PrependResource { resource }, it))
              })
          }
          "push_resource" => {
            Self::create_generic_elem::<_, Resource<VoidTableDataContent>>(args, it, first_elem)
              .and_then(|(wrapped, it)| {
                wrapped
                  .resource()
                  .map(|resource| (Self::PushResource { resource }, it))
              })
          }
          _ => Err(VOTableError::Custom(format!(
            "Unrecognized Action. Actual: {}. Expected: one of [rm, set_attrs, set_content, \
          set_description, push_timesys, set_min, set_max, push_option, set_values, push_info, \
          push_post_info, push_link, push_fieldref, push_paramref, push_coosys, push_group, \
           push_table_group, push_param, push_field, prepend_resource, push_resource, ...].",
            action
          ))),
        }
      }
      None => Err(VOTableError::Custom(format!(
        "No whitespace to spit on in '{}'",
        current
      ))),
    }
  }

  fn create_generic_elem<'a, I, T>(
    args: &'a str,
    mut it: Peekable<I>,
    first_elem: bool,
  ) -> Result<(VOTableWrappedElem<VoidTableDataContent>, Peekable<I>), VOTableError>
  where
    I: Iterator<Item = &'a str>,
    T: VOTableElement + Into<VOTableWrappedElem<VoidTableDataContent>>,
  {
    let attrs = Action::str2map(args);
    let mut elem = VOTableWrappedElem::from_attrs::<T, _, _, _>(attrs.into_iter())?;
    if first_elem {
      // Continue applying actions on this object untill '@<' is found
      while let Some(curr) = it.next() {
        it = Action::from_sub_action(curr, it, false)
          .and_then(|(action, it)| action.apply_on_wrapped_elem(&mut elem).map(|()| it))?;
      }
      Ok((elem, it))
    } else {
      match it.peek() {
        Some(e) if e.trim().is_empty() => {
          // '@@' encountered, apply next commands on this item
          let _ = it.next();
          // Continue applying actions on this object untill '@<' is found
          while let Some(curr) = it.next() {
            if curr.trim() == "<" {
              return Ok((elem, it));
            } else {
              it = Action::from_sub_action(curr, it, false)
                .and_then(|(action, it)| action.apply_on_wrapped_elem(&mut elem).map(|()| it))?;
            }
          }
          Ok((elem, it))
        }
        _ => Ok((elem, it)),
      }
    }
  }
}
impl FromStr for Action {
  type Err = VOTableError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.trim_end() == "rm" {
      Ok(Self::Rm)
    } else {
      let mut it = s.split('@').peekable();
      let (action, mut it) = Self::from_sub_action(it.next().unwrap(), it, true)?;
      match it.next() {
        None => Ok(action),
        Some(e) => Err(VOTableError::Custom(format!(
          "Action iterator not empty, remaining: '{}'",
          e
        ))),
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::edit::Action;

  #[test]
  fn test_parse_action() {
    let s = r#"push_coosys ID=t4-coosys-1 system=eq_FK4 equinox=B1900 
        @push_fieldref ref=RA1900 @@set_content Ref to the RA column @<
        @push_fieldref ref=DE1900 @@set_content Ref to the Declination column @<"#;
    let a = s.parse::<Action>();
    println!("{:?}", a);
    assert!(a.is_ok())
  }

  #[test]
  fn test_parse_action2() {
    let s = r#"push_post_info name=ps value=my post-scriptum @set_content My super post-scriptum"#;
    let a = s.parse::<Action>();
    println!("{:?}", a);
    assert!(a.is_ok())
  }
}
