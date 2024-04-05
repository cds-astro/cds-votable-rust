macro_rules! impl_builder_mandatory_string_attr {
  ($arg:ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.[<set_ $arg _by_ref>]($arg);
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$arg = $arg.into();
      }
    }
  };
  ($arg:ident, $alt:ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.[<set_ $alt _by_ref>]($arg);
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$arg = $arg.into();
      }
    }
  };
}

macro_rules! impl_builder_mandatory_string_attr_delegated {
  ($arg:ident, $del:ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$del.$arg = $arg.into();
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$del.$arg = $arg.into();
      }
    }
  };
  ($arg:ident, $alt:ident, $del:ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$del.$arg = $arg.into();
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$del.$arg = $arg.into();
      }
    }
  };
}

macro_rules! impl_builder_mandatory_attr {
  ($arg: ident, $t: ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.$arg = $arg;
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>](&mut self, $arg: $t) {
        self.$arg = $arg;
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.$arg = $arg;
        self
      }
     #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        self.$arg = $arg;
      }
    }
  };
}

macro_rules! impl_builder_mandatory_attr_delegated {
  ($arg: ident, $t: ident, $del:ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.$del.$arg = $arg;
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>](&mut self, $arg: $t) {
        self.$del.$arg = $arg;
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident, $del:ident) => {
    paste! {
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.$del.$arg = $arg;
        self
      }
      #[doc = concat!("Re-set the mandatory attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        self.$del.$arg = $arg;
      }
    }
  };
}

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
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$arg = Some($arg.into());
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$arg = Some($arg.into());
      }
    }
  };
  ($arg:ident, $alt:ident) => {
    paste! {
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$arg = Some($arg.into());
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$arg = Some($arg.into());
      }
    }
  };
}

macro_rules! impl_builder_opt_string_attr_delegated {
  ($arg:ident, $del:ident) => {
    paste! {
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$del.$arg = Some($arg.into());
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$del.$arg = Some($arg.into());
      }
    }
  };
  ($arg:ident, $alt:ident, $del:ident) => {
    paste! {
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>]<I: Into<String>>(mut self, $arg: I) -> Self {
        self.$del.$arg = Some($arg.into());
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>]<I: Into<String>>(&mut self, $arg: I) {
        self.$del.$arg = Some($arg.into());
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
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.$arg = Some($arg);
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>](&mut self, $arg: $t) {
        self.$arg = Some($arg);
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident) => {
    paste! {
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.$arg = Some($arg);
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        self.$arg = Some($arg);
      }
    }
  };
}

macro_rules! impl_builder_opt_attr_delegated {
  ($arg: ident, $t: ident, $del:ident) => {
    paste! {
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.[<set_ $arg _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>](&mut self, $arg: $t) {
        self.$del.$arg = Some($arg);
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident, $del:ident) => {
    paste! {
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.[<set_ $alt _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional attribute `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        self.$del.$arg = Some($arg);
      }
    }
  };
}

macro_rules! impl_builder_opt_subelem {
  ($arg: ident, $t: ident) => {
    paste! {
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.[<set_ $arg _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>](&mut self, $arg: $t) {
        if self.$arg.replace($arg).is_some() {
          warn!(concat!("Multiple occurrence of ", stringify!($arg), ". All but the last one are discarded."));
        }
      }
      #[doc = concat!("Reset the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<reset_ $arg>](mut self, $arg: $t) -> Self {
        self.[<reset_ $arg _by_ref>]($arg);
        self
      }
      #[doc = concat!("Reset the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<reset_ $arg _by_ref>](&mut self, $arg: $t) {
        let _ = self.$arg.replace($arg);
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident) => {
    paste! {
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.[<set_ $alt _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        if self.$arg.replace($arg).is_some() {
          warn!(concat!("Multiple occurrence of ", stringify!($arg), ". All but the last one are discarded."));
        }
      }
      #[doc = concat!("Reset the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<reset_ $alt>](mut self, $arg: $t) -> Self {
        self.[<reset_ $alt _by_ref>]($arg);
        self
      }
      #[doc = concat!("Reset the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<reset_ $alt _by_ref>](&mut self, $arg: $t) {
        let _ = self.$arg.replace($arg);
      }
    }
  };
}

macro_rules!  impl_builder_opt_subelem_delegated {
  ($arg: ident, $t: ident, $del:ident) => {
    paste! {
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $arg>](mut self, $arg: $t) -> Self {
        self.[<set_ $arg _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $arg _by_ref>](&mut self, $arg: $t) {
        if self.$del.$arg.replace($arg).is_some() {
          warn!(concat!("Multiple occurrence of ", stringify!($arg), ". All but the last one are discarded."));
        }
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<reset_ $arg>](mut self, $arg: $t) -> Self {
        self.[<reset_ $arg _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<reset_ $arg _by_ref>](&mut self, $arg: $t) {
        let _ = self.$del.$arg.replace($arg);
      }
    }
  };
  ($arg: ident, $alt:ident, $t: ident, $del:ident) => {
    paste! {
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<set_ $alt>](mut self, $arg: $t) -> Self {
        self.[<set_ $alt _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        if self.$del.$arg.replace($arg).is_some() {
          warn!(concat!("Multiple occurrence of ", stringify!($arg), ". All but the last one are discarded."));
        }
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` taking the ownership and returning itself.")]
      pub fn [<reset_ $alt>](mut self, $arg: $t) -> Self {
        self.[<reset_ $alt _by_ref>]($arg);
        self
      }
      #[doc = concat!("Set the optional sub-element `", stringify!($arg), "` by mutable ref.")]
      pub fn [<set_ $alt _by_ref>](&mut self, $arg: $t) {
        let _ = self.$del.$arg.replace($arg);
      }
    }
  };
}

macro_rules! impl_has_content {
  ($tag:ident) => {
    paste! {
      impl HasContent for $tag {
        fn get_content(&self) -> Option<&str> {
          self.content.as_deref()
        }
        fn set_content<S: Into<String>>(mut self, content: S) -> Self {
          self.content = Some(content.into());
          self
        }
        fn set_content_by_ref<S: Into<String>>(&mut self, content: S) {
          self.content = Some(content.into());
        }
      }
    }
  };
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
      #[doc = concat!("Add the given `", stringify!($e), "` to the element list.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given `", stringify!($e), "` to the element list, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.elems.push($e::$t([<$t:lower>]));
      }
    }
  };
  ($t: ident, $e: expr, $a: ident) => {
    paste! {
      #[doc = concat!("Add the given `", stringify!($a), "` to the element list.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $a) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given `", stringify!($a), "` to the element list, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $a) {
        self.elems.push($e::$t([<$t:lower>]));
      }
    }
  };
}

/*
macro_rules! impl_builder_prepend_elem {
  ($t: ident, $e: expr) => {
    paste! {
      #[doc = concat!("Add the given `", stringify!($e), "` to the element list.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given `", stringify!($e), "` to the element list, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.elems.insert(0, $e::$t([<$t:lower>]));
      }
    }
  };
  ($t: ident, $e: expr, $a: ident) => {
    paste! {
      #[doc = concat!("Add the given `", stringify!($a), "` to the element list.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $a) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given `", stringify!($a), "` to the element list, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $a) {
        self.elems.insert(0, $e::$t([<$t:lower>]));
      }
    }
  };
}*/

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
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.[<$t:lower s>].push([<$t:lower>]);
      }
    }
  };
  ($t: ident, $c: ident) => {
    paste! {
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t<$c>) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t<$c>) {
        self.[<$t:lower s>].push([<$t:lower>]);
      }
    }
  };
}

macro_rules! impl_builder_prepend {
  ($t: ident) => {
    paste! {
      #[doc = concat!("Prepend the given object to the list of `", stringify!($t), "`.")]
      pub fn [<prepend_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<prepend_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Prepend the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<prepend_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.[<$t:lower s>].insert(0, [<$t:lower>]);
      }
    }
  };
  ($t: ident, $c: ident) => {
    paste! {
      #[doc = concat!("Prepend the given object to the list of `", stringify!($t), "`.")]
      pub fn [<prepend_ $t:lower>](mut self, [<$t:lower>]: $t<$c>) -> Self {
        self.[<prepend_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Prepend the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<prepend_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t<$c>) {
        self.[<$t:lower s>].insert(0, [<$t:lower>]);
      }
    }
  };
}

macro_rules! impl_builder_push_delegated {
  ($t: ident, $del: ident) => {
    paste! {
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.$del.[<$t:lower s>].push([<$t:lower>]);
      }
    }
  };
  ($t: ident, $c: ident, $del: ident) => {
    paste! {
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t<$c>) -> Self {
        self.[<push_ $t:lower _by_ref>]([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t<$c>) {
        self.$del.[<$t:lower s>].push([<$t:lower>]);
      }
    }
  };
}

/// Like macro `impl_builder_push` but without adding a 's' to the vector attribute name.
#[cfg(feature = "mivot")]
macro_rules! impl_builder_push_no_s {
  ($t: ident) => {
    paste! {
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t) -> Self {
        self.[<$t:lower>].push([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t) {
        self.[<$t:lower>].push([<$t:lower>]);
      }
    }
  };
  ($t: ident, $c: ident) => {
    paste! {
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`.")]
      pub fn [<push_ $t:lower>](mut self, [<$t:lower>]: $t<$c>) -> Self {
        self.[<$t:lower>].push([<$t:lower>]);
        self
      }
      #[doc = concat!("Add the given object to the list of `", stringify!($t), "`, by mutable ref.")]
      pub fn [<push_ $t:lower _by_ref>](&mut self, [<$t:lower>]: $t<$c>) {
        self.[<$t:lower>].push([<$t:lower>]);
      }
    }
  };
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
    #[doc = concat!("Add the given info to the list of post infos.")]
    pub fn push_post_info(mut self, info: Info) -> Self {
      self.push_post_info_by_ref(info);
      self
    }
    #[doc = concat!("Add the given info to the list of post infos, by mutable ref.")]
    pub fn push_post_info_by_ref(&mut self, info: Info) {
      self.post_infos.push(info);
    }
  };
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
    /// Insert a **non-standard** attribute, taking the ownership and returning itself.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra<S: Into<String>>(mut self, key: S, value: Value) -> Self {
      self.insert_extra_by_ref(key, value);
      self
    }
    /// Insert a **non-standard** attribute by mutable reference.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra_by_ref<S: Into<String>>(&mut self, key: S, value: Value) {
      let mut key = key.into();
      if !key.as_str().contains(':') {
        key = format!("extra:{}", &key);
      }
      self.extra.insert(key, value);
    }
    /// Insert a **non-standard** string attribute, taking the ownership and returning itself.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra_str<S: Into<String>, T: Into<String>>(mut self, key: S, value: T) -> Self {
      self.insert_extra_by_ref(key, Value::String(value.into()));
      self
    }
    /// Insert a **non-standard** string attribute by mutable reference.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra_str_by_ref<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) {
      self.insert_extra_by_ref(key, Value::String(value.into()))
    }
  };
}

macro_rules! impl_builder_insert_extra_delegated {
  ($del:ident) => {
    /// Insert a **non-standard** attribute, taking the ownership and returning itself.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra<S: Into<String>>(mut self, key: S, value: Value) -> Self {
      self.insert_extra_by_ref(key, value);
      self
    }
    /// Insert a **non-standard** attribute by mutable reference.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra_by_ref<S: Into<String>>(&mut self, key: S, value: Value) {
      let mut key = key.into();
      if !key.as_str().contains(':') {
        key = format!("extra:{}", &key);
      }
      self.$del.extra.insert(key, value);
    }
    /// Insert a **non-standard** string attribute, taking the ownership and returning itself.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra_str<S: Into<String>, T: Into<String>>(mut self, key: S, value: T) -> Self {
      self.insert_extra_by_ref(key, Value::String(value.into()));
      self
    }
    /// Insert a **non-standard** string attribute by mutable reference.
    /// # Info
    /// The attribute name must contains ':', else the  `extra:` prefix is added.
    pub fn insert_extra_str_by_ref<S: Into<String>, T: Into<String>>(&mut self, key: S, value: T) {
      self.insert_extra_by_ref(key, Value::String(value.into()))
    }
  };
}

/*
/// Read a tag content (without sub-elements)
macro_rules! read_content_by_ref {
  ($Self:ident, $self:ident, $reader:ident, $reader_buff:ident) => {{
    let mut content = String::new();
    loop {
      let mut event = $reader
        .read_event($reader_buff)
        .map_err(VOTableError::Read)?;
      match &mut event {
        Event::Text(e) => content.push_str(
          e.unescape_and_decode(&$reader)
            .map_err(VOTableError::Read)?
            .as_str(),
        ),
        Event::CData(e) => content
          .push_str(str::from_utf8(e.clone().into_inner().as_ref()).map_err(VOTableError::Utf8)?),
        Event::End(e) if e.local_name() == $Self::TAG_BYTES => {
          $self.content = Some(content);
          return Ok(());
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, &$reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }};
  ($Self:ident, $self:ident, $reader:ident, $reader_buff:ident, $content:tt) => {{
    let mut content = String::new();
    loop {
      let mut event = $reader
        .read_event($reader_buff)
        .map_err(VOTableError::Read)?;
      match &mut event {
        Event::Text(e) => content.push_str(
          e.unescape_and_decode(&$reader)
            .map_err(VOTableError::Read)?
            .as_str(),
        ),
        Event::CData(e) => content.push_str(
          std::str::from_utf8(e.clone().into_inner().as_ref()).map_err(VOTableError::Utf8)?,
        ),
        Event::End(e) if e.local_name() == $Self::TAG_BYTES => {
          $self.$content = content;
          return Ok(());
        }
        Event::Eof => return Err(VOTableError::PrematureEOF(Self::TAG)),
        Event::Comment(e) => discard_comment(e, &$reader, Self::TAG),
        _ => discard_event(event, Self::TAG),
      }
    }
  }};
}*/

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

macro_rules! for_each_extra_attribute {
  ($self:ident, $f:ident) => {
    for (k, v) in &$self.extra {
      match v {
        Value::Null => $f(k.as_str(), ""),
        Value::Bool(v) => $f(k.as_str(), v.to_string().as_str()),
        Value::Number(v) => $f(k.as_str(), v.to_string().as_str()),
        Value::String(v) => $f(k.as_str(), v.to_string().as_str()),
        Value::Array(_) => $f(k.as_str(), v.to_string().as_str()),
        Value::Object(_) => $f(k.as_str(), v.to_string().as_str()),
      }
    }
  };
}

macro_rules! for_each_extra_attribute_delegated {
  ($self:ident, $del:ident, $f:ident) => {
    for (k, v) in &$self.$del.extra {
      match v {
        Value::Null => $f(k.as_str(), ""),
        Value::Bool(v) => $f(k.as_str(), v.to_string().as_str()),
        Value::Number(v) => $f(k.as_str(), v.to_string().as_str()),
        Value::String(v) => $f(k.as_str(), v.to_string().as_str()),
        Value::Array(_) => $f(k.as_str(), v.to_string().as_str()),
        Value::Object(_) => $f(k.as_str(), v.to_string().as_str()),
      }
    }
  };
}

/*
macro_rules! impl_read_write_content_only {
  () => {
    fn read_sub_elements_by_ref<R: BufRead>(
      &mut self,
      reader: &mut Reader<R>,
      reader_buff: &mut Vec<u8>,
      _context: &Self::Context,
    ) -> Result<(), VOTableError> {
      read_content_by_ref!(Self, self, reader, reader_buff)
    }

    fn write_sub_elements_by_ref<W: Write>(
      &mut self,
      _writer: &mut Writer<W>,
      _context: &Self::Context,
    ) -> Result<(), VOTableError> {
      unreachable!()
    }
  };
}*/

macro_rules! write_elem {
  ($self:ident, $elem:ident, $writer:ident, $context:ident) => {
    if let Some(elem) = &mut $self.$elem {
      elem.write($writer, $context)?;
    }
  };
}

macro_rules! write_elem_vec {
  ($self:ident, $elems:ident, $writer:ident, $context:ident) => {
    for elem in &mut $self.$elems {
      elem.write($writer, $context)?;
    }
  };
}

macro_rules! write_elem_vec_no_context {
  ($self:ident, $elems:ident, $writer:ident) => {
    for elem in &mut $self.$elems {
      elem.write($writer)?;
    }
  };
}

macro_rules! write_elem_vec_empty_context {
  ($self:ident, $elems:ident, $writer:ident) => {
    for elem in &mut $self.$elems {
      elem.write($writer, &())?;
    }
  };
}

// We use a macro because of mutable borrowing issue with 'reader_buff' if we put this in a function.
macro_rules! from_event_start_by_ref {
  ($elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {{
    $elem::from_event_start($e)
      .and_then(|elem| elem.read_content(&mut $reader, &mut $reader_buff, &()))?
  }};
  ($elem:ident, $reader:ident, $reader_buff:ident, $e:ident, $context:expr) => {{
    $elem::from_event_start($e)
      .and_then(|elem| elem.read_content(&mut $reader, &mut $reader_buff, &$context))?
  }};
}

macro_rules! set_from_event_start {
  ($self:ident, $elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    paste! {
      $elem::from_event_start($e)
        .and_then(|elem| elem.read_content($reader, $reader_buff, &()))
        .map(|elem| $self.[<set_ $elem:lower _by_ref>](elem))?
    }
  };
  ($self:ident, $elem:ident, $reader:ident, $reader_buff:ident, $e:ident, $context:expr) => {
    paste! {
      $elem::from_event_start($e)
        .and_then(|elem| elem.read_content($reader, $reader_buff, &$context))
        .map(|elem| $self.[<set_ $elem:lower _by_ref>](elem))?
    }
  };
}

macro_rules! set_from_event_empty {
  ($self:ident, $elem:ident, $e:ident) => {
    paste! {
      $elem::from_event_empty($e)
        .map(|elem| $self.[<set_ $elem:lower _by_ref>](elem))?
    }
  };
}

macro_rules! set_desc_from_event_start {
  ($self:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    set_from_event_start!($self, Description, $reader, $reader_buff, $e)
  };
}

macro_rules! push_from_event_start {
  ($self:ident, $elem:ident, $reader:ident, $reader_buff:ident, $e:ident) => {
    paste! {
      $elem::from_event_start($e)
      .and_then(|elem| elem.read_content(&mut $reader, &mut $reader_buff, &()))
      .map(|elem| $self.[<push_ $elem:lower _by_ref>](elem))?
    }
  };
  ($self:ident, $elem:ident, $reader:ident, $reader_buff:ident, $e:ident, $context:expr) => {
    paste! {
      $elem::from_event_start($e)
      .and_then(|elem| elem.read_content(&mut $reader, &mut $reader_buff, &$context))
      .map(|elem| $self.[<push_ $elem:lower _by_ref>](elem))?
    }
  };
}

macro_rules! push_from_event_empty {
  ($self:ident, $elem:ident, $e:ident) => {
    paste! {
      $elem::from_event_empty($e)
      .map(|elem| $self.[<push_ $elem:lower _by_ref>](elem))?
    }
  };
}
