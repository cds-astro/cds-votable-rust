
/// E.g. `impl_builder_opt_string_attr(id)` leads to
/// ```ignore
/// pub fn set_id<I: Into<String>>(mut self, id: I) -> Self {
///    self.id.insert(id.into());
///    self
/// }
/// ``` 
macro_rules! impl_builder_opt_string_attr {
  ($arg:ident) => {
    paste! {
      pub fn [<set_ $arg>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$arg = Some($arg.into());
        self
      }
    }
  };
  ($arg:ident, $alt:ident) => {
    paste! {
      pub fn [<set_ $alt>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$arg = Some($arg.into());
        self
      }
    }
  };
}

/// E.g. `impl_builder_opt_attr(description, Description)` leads to
/// ```ignore
/// pub fn set_description<I: Into<String>>(mut self, description: Description) -> Self {
///    self.description.insert(description);
///    self
/// }
/// ``` 
macro_rules! impl_builder_opt_attr {
  ($arg: ident, $t: ident) => {
    paste! {
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.$arg = Some($arg);
        self
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident) => {
    paste! {
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.$arg = Some($arg);
        self
      }
    }
  }
}

/// E.g. `impl_builder_push_elem(CooSys, ResourceElem)` leads to 
/// ```ignore
/// pub fn push_coosys(mut self, coosys: CooSys) -> Self {
///   self.elems.push(ResourceElem::CooSyst(coosys)); 
///   self
/// }
/// ```
macro_rules! impl_builder_push_elem {
  ($t: ident, $e: expr) => {
    paste! {
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.elems.push($e::$t([<$t:lower>])); 
        self
      }
      
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.elems.push($e::$t([<$t:lower>])); 
      }
    }
  }
}

/// E.g. `impl_builder_push(Info)` leads to 
/// ```ignore
/// pub fn push_info(mut self, info: Info) -> Self {
///   self.infos.push(info); 
///   self
/// }
/// ```
macro_rules! impl_builder_push {
  ($t: ident) => {
    paste! {
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<$t:lower s>].push([<$t:lower>]);
        self
      }
      
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.[<$t:lower s>].push([<$t:lower>]);
      }
    }
  };
    ($t: ident, $c: ident) => {
    paste! {
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t<$c>) -> Self {
        self.[<$t:lower s>].push([<$t:lower>]);
        self
      }
      
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t<$c>) {
        self.[<$t:lower s>].push([<$t:lower>]);
      }
    }
  }
}


/// Simply append the following method:
/// ```ignore
/// pub fn push_post_info(mut self, info: Info) -> Self {
///   self.post_infos.push(info);
///   self
/// }
/// ```
macro_rules! impl_builder_push_post_info {
  () => {
    pub fn push_post_info(mut self, info: Info) -> Self {
      self.post_infos.push(info);
      self
    }
  }
}

/// Simply append the following method:
/// ```ignore
/// pub fn insert_extra<S: Into<String>>(mut self, key: S, value: Value) -> Self {
///   self.extra.insert(key.into(), value);
///   self
/// }
/// ```
macro_rules! impl_builder_insert_extra {
  () => {
    pub fn insert_extra<S: Into<String>>(mut self, key: S, value: Value) -> Self {
      self.extra.insert(key.into(), value);
      self
    }
  }
}

macro_rules! read_content {
  ($Self:ident, $self:ident, $reader:ident, $reader_buff:ident) => {
    {
      /*let event: Event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      let content = match &event {
        Event::Text(e) => e.unescape_and_decode(&$reader).map_err(VOTableError::Read),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: Text. Actual: {:?}.", $Self::TAG, event))),
      }?;
      $self.content = Some(content);
      let event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      match &event {
        Event::End(e) if e.local_name() == $Self::TAG_BYTES => Ok($reader),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: End. Actual: {:?}.", $Self::TAG, event))),
      }*/
      let mut content = String::new();
      loop {
        let mut event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
          Event::Text(e) => content.push_str(e.unescape_and_decode(&$reader).map_err(VOTableError::Read)?.as_str()),
          Event::CData(e) => content.push_str(str::from_utf8(e.clone().into_inner().as_ref()).map_err(VOTableError::Utf8)?),
          Event::End(e) if e.local_name() == $Self::TAG_BYTES => { 
            $self.content = Some(content);
            return Ok($reader);
          },
          Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
          _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
          // _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: End or Text or CDATA. Actual: {:?}.", $Self::TAG, event))),
        }
      }
    }
  };
  ($Self:ident, $self:ident, $reader:ident, $reader_buff:ident, $content:tt) => {
    {
      /*let event: Event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      let content = match &event {
        Event::Text(e) => e.unescape_and_decode(&$reader).map_err(VOTableError::Read),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: Text. Actual: {:?}.", $Self::TAG, event))),
      }?;
      $self.$content = content;
      let event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      match &event {
        Event::End(e) if e.local_name() == $Self::TAG_BYTES => Ok($reader),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: End. Actual: {:?}.", $Self::TAG, event))),
      }*/
      let mut content = String::new();
      loop {
        let mut event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
        match &mut event {
          Event::Text(e) => content.push_str(e.unescape_and_decode(&$reader).map_err(VOTableError::Read)?.as_str()),
          Event::CData(e) => content.push_str(std::str::from_utf8(e.clone().into_inner().as_ref()).map_err(VOTableError::Utf8)?),
          Event::End(e) if e.local_name() == $Self::TAG_BYTES => { 
            $self.$content = content;
            return Ok($reader);
          },
          Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
          _ => eprintln!("Discarded event in {}: {:?}", Self::TAG, event),
          // _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: End or Text or CDATA. Actual: {:?}.", $Self::TAG, event))),
        }
      }
    }
  };
}

macro_rules! read_content_by_ref {
  ($Self:ident, $self:ident, $reader:ident, $reader_buff:ident) => {
    {
      let event: Event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      let content = match &event {
        Event::Text(e) => e.unescape_and_decode(&$reader).map_err(VOTableError::Read),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: Text. Actual: {:?}.", $Self::TAG, event))),
      }?;
      $self.content = Some(content);
      let event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      match &event {
        Event::End(e) if e.local_name() == $Self::TAG_BYTES => Ok(()),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: End. Actual: {:?}.", $Self::TAG, event))),
      }
    }
  };
  ($Self:ident, $self:ident, $reader:ident, $reader_buff:ident, $content:tt) => {
    {
      let event: Event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      let content = match &event {
        Event::Text(e) => e.unescape_and_decode(&$reader).map_err(VOTableError::Read),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: Text. Actual: {:?}.", $Self::TAG, event))),
      }?;
      $self.$content = content;
      let event = $reader.read_event($reader_buff).map_err(VOTableError::Read)?;
      match &event {
        Event::End(e) if e.local_name() == $Self::TAG_BYTES => Ok(()),
        _ => Err(VOTableError::Custom(format!("Unexpected {} event. Expected: End. Actual: {:?}.", $Self::TAG, event))),
      }
    }
  };
}



/// E.g. `write_opt_string_attr(self, elem_writer, ID)` leads to
/// ```ignore
/// if let Some(id) = self.id.as_ref() {
///     elem_writer.with_attribute(("ID", id));
/// }
/// ```
macro_rules! write_opt_string_attr {
  ($self:ident, $elem_writer:ident, $arg:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = $self.[<$arg:lower>].as_ref() {
        $elem_writer = $elem_writer.with_attribute((stringify!($arg), [<$arg:lower>].as_str()));
      }
    }
  };
  ($self:ident, $elem_writer:ident, $arg:ident, $arg_str:literal) => {
    paste! {
      if let Some([<$arg:lower>]) = $self.[<$arg:lower>].as_ref() {
        $elem_writer = $elem_writer.with_attribute(($arg_str, [<$arg:lower>].as_str()));
      }
    }
  };
}

macro_rules! write_opt_tostring_attr {
  ($self:ident, $elem_writer:ident, $arg:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = $self.[<$arg:lower>].as_ref() {
        $elem_writer = $elem_writer.with_attribute((stringify!($arg), [<$arg:lower>].to_string().as_str()));
      }
    }
  };
  ($self:ident, $elem_writer:ident, $arg:ident, $arg_str:literal) => {
    paste! {
      if let Some([<$arg:lower>]) = $self.[<$arg:lower>].as_ref() {
        $elem_writer = $elem_writer.with_attribute(($arg_str, [<$arg:lower>].to_string().as_str()));
      }
    }
  };
}

macro_rules! write_opt_into_attr {
  ($self:ident, $elem_writer:ident, $arg:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = $self.[<$arg:lower>].as_ref() {
        $elem_writer = $elem_writer.with_attribute((stringify!($arg), [<$arg:lower>].into()));
      }
    }
  };
  ($self:ident, $elem_writer:ident, $arg:ident, $arg_str:literal) => {
    paste! {
      if let Some([<$arg:lower>]) = $self.[<$arg:lower>].as_ref() {
        $elem_writer = $elem_writer.with_attribute(($arg_str, [<$arg:lower>].into()));
      }
    }
  };
}

macro_rules! write_extra {
  ($self:ident, $elem_writer:ident) => {
    for (key, val) in &$self.extra {
       $elem_writer = $elem_writer.with_attribute((key.as_str(), val.to_string().as_str()));
    }
  }
}


macro_rules! push2write_opt_string_attr {
  ($self:ident, $tag:ident, $arg:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = &$self.[<$arg:lower>] {
        $tag.push_attribute((stringify!($arg), [<$arg:lower>].as_str()));
      }
    }
  };
  ($self:ident, $tag:ident, $arg:ident, $arg_str:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = &$self.[<$arg:lower>] {
        $tag.push_attribute((stringify!($arg_str), [<$arg:lower>].as_str()));
      }
    }
  };
}

macro_rules! push2write_opt_tostring_attr {
  ($self:ident, $tag:ident, $arg:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = &$self.[<$arg:lower>] {
        $tag.push_attribute((stringify!($arg), [<$arg:lower>].to_string().as_str()));
      }
    }
  };
  ($self:ident, $tag:ident, $arg:ident, $arg_str:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = &$self.[<$arg:lower>] {
        $tag.push_attribute((stringify!($arg_str), [<$arg:lower>].to_string().as_str()));
      }
    }
  };
}

macro_rules! push2write_opt_into_attr {
  ($self:ident, $tag:ident, $arg:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = &$self.[<$arg:lower>] {
        $tag.push_attribute((stringify!($arg), [<$arg:lower>].into()));
      }
    }
  };
  ($self:ident, $tag:ident, $arg:ident, $arg_str:ident) => {
    paste! {
      if let Some([<$arg:lower>]) = &$self.[<$arg:lower>] {
        $tag.push_attribute((stringify!($arg_str), [<$arg:lower>].into()));
      }
    }
  };
}

macro_rules! push2write_extra {
  ($self:ident, $tag:ident) => {
    for (key, val) in &$self.extra {
      match val {
        Value::Null => $tag.push_attribute((key.as_str(), "")),
        Value::Bool(val) => $tag.push_attribute((key.as_str(), val.to_string().as_str())),
        Value::Number(val) => $tag.push_attribute((key.as_str(), val.to_string().as_str())),
        Value::String(val) => $tag.push_attribute((key.as_str(), val.to_string().as_str())),
        Value::Array(_) => $tag.push_attribute((key.as_str(), val.to_string().as_str())),
        Value::Object(_) => $tag.push_attribute((key.as_str(), val.to_string().as_str())),
      }
    }
  }
}


macro_rules! write_content {
  ($self:ident, $elem_writer:ident) => {
    if let Some(content) = &$self.content {
      $elem_writer.write_text_content(
        BytesText::from_plain_str(content.as_str())
      )
    } else {
      $elem_writer.write_empty()
    }.map_err(VOTableError::Write)?;
  }
}

macro_rules! write_elem {
  ($self:ident, $elem:ident, $writer:ident, $context:ident) => {
    if let Some(elem) = &mut $self.$elem {
      elem.write($writer, $context)?;
    }
  }
}

macro_rules! write_elem_vec {
  ($self:ident, $elems:ident, $writer:ident, $context:ident) => {
    for elem in &mut $self.$elems {
      elem.write($writer, $context)?;
    }
  }
}

/*macro_rules! write_elem_no_context {
  ($self:ident, $elem:ident, $writer:ident) => {
    if let Some(elem) = &mut $self.$elem {
      elem.write($writer)?;
    }
  }
}*/

macro_rules! write_elem_vec_no_context {
  ($self:ident, $elems:ident, $writer:ident) => {
    for elem in &mut $self.$elems {
      elem.write($writer)?;
    }
  }
}

/*macro_rules! write_elem_empty_context {
  ($self:ident, $elem:ident, $writer:ident) => {
    if let Some(elem) = &mut $self.$elem {
      elem.write($writer, &())?;
    }
  }
}*/

macro_rules! write_elem_vec_empty_context {
  ($self:ident, $elems:ident, $writer:ident) => {
    for elem in &mut $self.$elems {
      elem.write($writer, &())?;
    }
  }
}


macro_rules! from_event_start {
  ($elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    {
      let mut elem = $elem::from_attributes($e.attributes())?;
      $reader = elem.read_sub_elements_and_clean($reader, &mut $reader_buff, &())?;
      elem
    }
  };
  ($elem:ident, $reader:ident, $reader_buff:ident, $e:ident, $context:expr) => {
    {
      let mut elem = $elem::from_attributes($e.attributes())?;
      $reader = elem.read_sub_elements_and_clean($reader, &mut $reader_buff, &$context)?;
      elem
    }
  };
}

macro_rules! from_event_start_by_ref {
  ($elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    {
      let mut elem = $elem::from_attributes($e.attributes())?;
      elem.read_sub_elements_and_clean_by_ref(&mut $reader, &mut $reader_buff, &())?;
      elem
    }
  };
  ($elem:ident, $reader:ident, $reader_buff:ident, $e:ident, $context:expr) => {
    {
      let mut elem = $elem::from_attributes($e.attributes())?;
      elem.read_sub_elements_and_clean_by_ref(&mut $reader, &mut $reader_buff, &$context)?;
      elem
    }
  };
}


macro_rules! from_event_start_desc {
  ($self:ident, $elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    {
      let mut desc = $elem::from_attributes($e.attributes())?;
      $reader = desc.read_sub_elements_and_clean($reader, &mut $reader_buff, &())?;
      if $self.description.replace(desc).is_some() {
        eprintln!("WARNING: multiple occurrence of DESCRIPTION in VOTable. All but the last one are discarded.");
      }
    }
  };
}

macro_rules! from_event_start_desc_by_ref {
  ($self:ident, $elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    {
      let mut desc = $elem::from_attributes($e.attributes())?;
      desc.read_sub_elements_and_clean_by_ref(&mut $reader, &mut $reader_buff, &())?;
      if $self.description.replace(desc).is_some() {
        eprintln!("WARNING: multiple occurrence of DESCRIPTION in VOTable. All but the last one are discarded.");
      }
    }
  };
}
