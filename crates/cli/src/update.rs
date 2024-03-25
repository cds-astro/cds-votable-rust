use std::str::FromStr;

use clap::Args;
use log::trace;

use votable::error::VOTableError;

use super::{
  input::Input,
  output::Output,
  visitors::{update::UpdateVisitor, Tag},
};

/// Update metadata from another VOTable of from a CSV file.
#[derive(Debug, Args)]
pub struct Update {
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
  ///   rm
  ///   set_attrs   KEY=null KEY=VAL ...  
  ///   set_content VAL
  ///   set_desc    VAL
  #[arg(short = 'e', long = "elem", verbatim_doc_comment)]
  elems: Vec<TagConditionAction>,
}

impl Update {
  pub fn exec(self) -> Result<(), VOTableError> {
    /*if self.input.is_streamable()? && self.output.is_streamable() {
    // For streaming mode:
    // get_iter
    // clone votable
    // mdofi cloned votable, read cloned votable
    // continue iterating (reading end)
    // clone the old, completed votable
    // modif the clone
    // write the end of the coned
    } else {
    }*/
    trace!("Elems: {:?}", &self.elems);
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

#[derive(Debug, Clone)]
pub struct TagConditionAction {
  pub tag: Tag,
  pub condition: Condition,
  pub action: Action,
}

impl FromStr for TagConditionAction {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.trim_start().split_once(|c: char| c.is_whitespace()) {
      Some((tag, rem)) => Tag::from_str(tag).and_then(|tag| {
        match rem.trim_start().split_once(|c: char| c.is_whitespace()) {
          Some((condition, action)) => Condition::from_str(condition).and_then(|condition| {
            Action::from_str(action.trim_start()).and_then(|action| {
              Ok(TagConditionAction {
                tag,
                condition,
                action,
              })
            })
          }),
          None => Err(format!("No whitespace to spit on in '{}'", &rem)),
        }
      }),
      None => Err(format!("No whitespace to spit on in '{}'", s)),
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
    trace!(
      "Test: {:?} on: vid={}; id:{:?}; name={:?} => {}",
      self,
      &vid,
      id,
      name,
      res
    );
    res
  }
}
impl FromStr for Condition {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.split_once('=') {
      Some((key, val)) => match key {
        "vid" => Ok(Self::VirtualIdEq(val.to_string())),
        "id" => Ok(Self::IdEq(val.to_string())),
        "name" => Ok(Self::NameEq(val.to_string())),
        _ => Err(format!(
          "Unrecognized condition. Actual: {}. Expected: one of [vid, id, name].",
          key
        )),
      },
      None => Err(format!("No '=' to spit on in '{}'", s)),
    }
  }
}

#[derive(Debug, Clone)]
pub enum Action {
  /// `rm`: remove
  Rm,
  /// `set_attrs KEY=VALUE KEY=VALUE`: set attributes like the name
  SetAttrs { attributes: Vec<(String, String)> },
  /// `set_desc DESC` set the description for `VOTABLE`, `RESOURCE`, `TABLE`, `FIELD` and `GROUP`.
  SetDesc { new_desc: String },
  /// `set_content CONTENT` set the content for `INFO`, `LINK`, `PARAMRef` and `FIELDRef`.
  SetContent { new_content: String },
}
impl Action {
  fn str2map(s: &str) -> Vec<(String, String)> {
    let mut kv: Vec<(String, String)> = Vec::new();
    let mut it = s.split('=');
    if let Some(first_key) = it.next() {
      let mut key = first_key;
      while let Some(rem) = it.next() {
        if let Some((val, next_key)) = rem.rsplit_once(' ') {
          kv.push((key.to_string(), val.to_string()));
          key = next_key;
        } else if !it.next().is_none() {
          panic!("Unable to parse value and key in {}", rem)
        } else {
          kv.push((key.to_string(), rem.to_string()));
        }
      }
    }
    kv
  }
}
impl FromStr for Action {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.trim_end() == "rm" {
      Ok(Self::Rm)
    } else {
      match s.split_once(|c: char| c.is_whitespace()) {
        Some((action, args)) => match action {
          "set_attrs" => Ok(Self::SetAttrs {
            attributes: Action::str2map(args.trim_end()),
          }),
          "set_content" => Ok(Self::SetContent {
            new_content: args.to_string(),
          }),
          "set_description" => Ok(Self::SetDesc {
            new_desc: args.to_string(),
          }),
          _ => Err(format!(
            "Unrecognized Action. Actual: {}. Expected: one of [rm, set_attrs, set_content, set_description].",
            action
          )),
        },
        None => Err(format!("No whitespace to spit on in '{}'", s)),
      }
    }
  }
}
