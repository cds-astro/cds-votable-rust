//! Wrapper elements for easier dynamic build (but less efficient).

use votable::{
  Binary, Binary2, CooSys, Data, Definitions, Description, Field, FieldRef, Fits, Group,
  HasContent, Info, Link, Max, Min, Opt, Param, ParamRef, Resource, Stream, Table,
  TableDataContent, TableGroup, TimeSys, VOTable, VOTableElement, VOTableError, Values,
  VoidTableDataContent,
};

#[derive(Debug)]
pub enum VOTableWrappedElem<C: TableDataContent> {
  VOTable(VOTable<C>),
  Resource(Resource<C>),
  Table(Table<C>),
  Data(Data<C>),
  Binary(Binary<C>),
  Binary2(Binary2<C>),
  Fits(Fits),
  Stream(Stream<C>),
  FitsStream(Stream<VoidTableDataContent>),
  Description(Description),
  Definitions(Definitions),
  Info(Info),
  CooSys(CooSys),
  TimeSys(TimeSys),
  Field(Field),
  Link(Link),
  Param(Param),
  Group(Group),
  TableGroup(TableGroup),
  Values(Values),
  Option(Opt),
  Min(Min),
  Max(Max),
  FieldRef(FieldRef),
  ParamRef(ParamRef),
  // MIVOT related tags
  // Vodml(Vodml),
  // REPORT
  // MODEL
  // GLOBAL
  // TEMPLATE
  // ATTRIBUTE / COLLECTION / INSTANCE / REFERENCE
  // JOIN / WHERE / PRIMARY_KEY / FOREIGN_KEY
}

impl<C: TableDataContent> From<VOTable<C>> for VOTableWrappedElem<C> {
  fn from(value: VOTable<C>) -> Self {
    Self::VOTable(value)
  }
}
impl<C: TableDataContent> From<Resource<C>> for VOTableWrappedElem<C> {
  fn from(value: Resource<C>) -> Self {
    Self::Resource(value)
  }
}
impl<C: TableDataContent> From<Table<C>> for VOTableWrappedElem<C> {
  fn from(value: Table<C>) -> Self {
    Self::Table(value)
  }
}
impl<C: TableDataContent> From<Data<C>> for VOTableWrappedElem<C> {
  fn from(value: Data<C>) -> Self {
    Self::Data(value)
  }
}
impl<C: TableDataContent> From<Stream<C>> for VOTableWrappedElem<C> {
  fn from(value: Stream<C>) -> Self {
    Self::Stream(value)
  }
}
impl<C: TableDataContent> From<Fits> for VOTableWrappedElem<C> {
  fn from(value: Fits) -> Self {
    Self::Fits(value)
  }
}
impl<C: TableDataContent> From<Description> for VOTableWrappedElem<C> {
  fn from(value: Description) -> Self {
    Self::Description(value)
  }
}
impl<C: TableDataContent> From<Definitions> for VOTableWrappedElem<C> {
  fn from(value: Definitions) -> Self {
    Self::Definitions(value)
  }
}
impl<C: TableDataContent> From<Info> for VOTableWrappedElem<C> {
  fn from(value: Info) -> Self {
    Self::Info(value)
  }
}
impl<C: TableDataContent> From<CooSys> for VOTableWrappedElem<C> {
  fn from(value: CooSys) -> Self {
    Self::CooSys(value)
  }
}
impl<C: TableDataContent> From<TimeSys> for VOTableWrappedElem<C> {
  fn from(value: TimeSys) -> Self {
    Self::TimeSys(value)
  }
}
impl<C: TableDataContent> From<Field> for VOTableWrappedElem<C> {
  fn from(value: Field) -> Self {
    Self::Field(value)
  }
}
impl<C: TableDataContent> From<Link> for VOTableWrappedElem<C> {
  fn from(value: Link) -> Self {
    Self::Link(value)
  }
}
impl<C: TableDataContent> From<Param> for VOTableWrappedElem<C> {
  fn from(value: Param) -> Self {
    Self::Param(value)
  }
}
impl<C: TableDataContent> From<Group> for VOTableWrappedElem<C> {
  fn from(value: Group) -> Self {
    Self::Group(value)
  }
}
impl<C: TableDataContent> From<TableGroup> for VOTableWrappedElem<C> {
  fn from(value: TableGroup) -> Self {
    Self::TableGroup(value)
  }
}
impl<C: TableDataContent> From<Values> for VOTableWrappedElem<C> {
  fn from(value: Values) -> Self {
    Self::Values(value)
  }
}
impl<C: TableDataContent> From<Opt> for VOTableWrappedElem<C> {
  fn from(value: Opt) -> Self {
    Self::Option(value)
  }
}
impl<C: TableDataContent> From<Min> for VOTableWrappedElem<C> {
  fn from(value: Min) -> Self {
    Self::Min(value)
  }
}
impl<C: TableDataContent> From<Max> for VOTableWrappedElem<C> {
  fn from(value: Max) -> Self {
    Self::Max(value)
  }
}
impl<C: TableDataContent> From<FieldRef> for VOTableWrappedElem<C> {
  fn from(value: FieldRef) -> Self {
    Self::FieldRef(value)
  }
}
impl<C: TableDataContent> From<ParamRef> for VOTableWrappedElem<C> {
  fn from(value: ParamRef) -> Self {
    Self::ParamRef(value)
  }
}

impl VOTableWrappedElem<VoidTableDataContent> {
  pub fn from_attrs<T, K, V, I>(attrs: I) -> Result<Self, VOTableError>
  where
    T: VOTableElement + Into<VOTableWrappedElem<VoidTableDataContent>>,
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    T::from_attrs(attrs).map(|e| e.into())
  }
}

impl<C: TableDataContent> VOTableWrappedElem<C> {
  pub fn as_mut(&mut self) -> VOTableWrappedElemMut<C> {
    match self {
      Self::VOTable(e) => VOTableWrappedElemMut::VOTable(e),
      Self::Resource(e) => VOTableWrappedElemMut::Resource(e),
      Self::Table(e) => VOTableWrappedElemMut::Table(e),
      Self::Data(e) => VOTableWrappedElemMut::Data(e),
      Self::Binary(e) => VOTableWrappedElemMut::Binary(e),
      Self::Binary2(e) => VOTableWrappedElemMut::Binary2(e),
      Self::Stream(e) => VOTableWrappedElemMut::Stream(e),
      Self::FitsStream(e) => VOTableWrappedElemMut::FitsStream(e),
      Self::Fits(e) => VOTableWrappedElemMut::Fits(e),
      Self::Description(e) => VOTableWrappedElemMut::Description(e),
      Self::Definitions(e) => VOTableWrappedElemMut::Definitions(e),
      Self::Info(e) => VOTableWrappedElemMut::Info(e),
      Self::CooSys(e) => VOTableWrappedElemMut::CooSys(e),
      Self::TimeSys(e) => VOTableWrappedElemMut::TimeSys(e),
      Self::Field(e) => VOTableWrappedElemMut::Field(e),
      Self::Link(e) => VOTableWrappedElemMut::Link(e),
      Self::Param(e) => VOTableWrappedElemMut::Param(e),
      Self::Group(e) => VOTableWrappedElemMut::Group(e),
      Self::TableGroup(e) => VOTableWrappedElemMut::TableGroup(e),
      Self::Values(e) => VOTableWrappedElemMut::Values(e),
      Self::Option(e) => VOTableWrappedElemMut::Option(e),
      Self::Min(e) => VOTableWrappedElemMut::Min(e),
      Self::Max(e) => VOTableWrappedElemMut::Max(e),
      Self::FieldRef(e) => VOTableWrappedElemMut::FieldRef(e),
      Self::ParamRef(e) => VOTableWrappedElemMut::ParamRef(e),
    }
  }

  pub fn tag(&self) -> &'static str {
    match self {
      Self::VOTable(e) => e.tag(),
      Self::Resource(e) => e.tag(),
      Self::Table(e) => e.tag(),
      Self::Data(e) => e.tag(),
      Self::Binary(e) => e.tag(),
      Self::Binary2(e) => e.tag(),
      Self::Stream(e) => e.tag(),
      Self::FitsStream(e) => e.tag(),
      Self::Fits(e) => e.tag(),
      Self::Description(e) => e.tag(),
      Self::Definitions(e) => e.tag(),
      Self::Info(e) => e.tag(),
      Self::CooSys(e) => e.tag(),
      Self::TimeSys(e) => e.tag(),
      Self::Field(e) => e.tag(),
      Self::Link(e) => e.tag(),
      Self::Param(e) => e.tag(),
      Self::Group(e) => e.tag(),
      Self::TableGroup(e) => e.tag(),
      Self::Values(e) => e.tag(),
      Self::Option(e) => e.tag(),
      Self::Min(e) => e.tag(),
      Self::Max(e) => e.tag(),
      Self::FieldRef(e) => e.tag(),
      Self::ParamRef(e) => e.tag(),
    }
  }

  fn wrong_type(self, expected: &'static str) -> VOTableError {
    VOTableError::Custom(format!(
      "Wrong elem. Expected: {}. Actual: {}.",
      expected,
      self.tag()
    ))
  }
  pub fn votable(self) -> Result<VOTable<C>, VOTableError> {
    match self {
      Self::VOTable(x) => Ok(x),
      _ => Err(self.wrong_type(VOTable::<C>::TAG)),
    }
  }
  pub fn resource(self) -> Result<Resource<C>, VOTableError> {
    match self {
      Self::Resource(x) => Ok(x),
      _ => Err(self.wrong_type(Resource::<C>::TAG)),
    }
  }
  pub fn table(self) -> Result<Table<C>, VOTableError> {
    match self {
      Self::Table(x) => Ok(x),
      _ => Err(self.wrong_type(Table::<C>::TAG)),
    }
  }
  pub fn data(self) -> Result<Data<C>, VOTableError> {
    match self {
      Self::Data(x) => Ok(x),
      _ => Err(self.wrong_type(Data::<C>::TAG)),
    }
  }
  pub fn stream(self) -> Result<Stream<C>, VOTableError> {
    match self {
      Self::Stream(x) => Ok(x),
      _ => Err(self.wrong_type(Stream::<VoidTableDataContent>::TAG)),
    }
  }
  pub fn fits_stream(self) -> Result<Stream<VoidTableDataContent>, VOTableError> {
    match self {
      Self::FitsStream(x) => Ok(x),
      _ => Err(self.wrong_type(Stream::<VoidTableDataContent>::TAG)),
    }
  }
  pub fn fits(self) -> Result<Fits, VOTableError> {
    match self {
      Self::Fits(x) => Ok(x),
      _ => Err(self.wrong_type(Fits::TAG)),
    }
  }
  pub fn description(self) -> Result<Description, VOTableError> {
    match self {
      Self::Description(x) => Ok(x),
      _ => Err(self.wrong_type(Description::TAG)),
    }
  }
  pub fn definitions(self) -> Result<Definitions, VOTableError> {
    match self {
      Self::Definitions(x) => Ok(x),
      _ => Err(self.wrong_type(Definitions::TAG)),
    }
  }
  pub fn info(self) -> Result<Info, VOTableError> {
    match self {
      Self::Info(x) => Ok(x),
      _ => Err(self.wrong_type(Info::TAG)),
    }
  }
  pub fn coo_sys(self) -> Result<CooSys, VOTableError> {
    match self {
      Self::CooSys(x) => Ok(x),
      _ => Err(self.wrong_type(CooSys::TAG)),
    }
  }
  pub fn time_sys(self) -> Result<TimeSys, VOTableError> {
    match self {
      Self::TimeSys(x) => Ok(x),
      _ => Err(self.wrong_type(TimeSys::TAG)),
    }
  }
  pub fn field(self) -> Result<Field, VOTableError> {
    match self {
      Self::Field(x) => Ok(x),
      _ => Err(self.wrong_type(Field::TAG)),
    }
  }
  pub fn link(self) -> Result<Link, VOTableError> {
    match self {
      Self::Link(x) => Ok(x),
      _ => Err(self.wrong_type(Link::TAG)),
    }
  }
  pub fn param(self) -> Result<Param, VOTableError> {
    match self {
      Self::Param(x) => Ok(x),
      _ => Err(self.wrong_type(Param::TAG)),
    }
  }
  pub fn group(self) -> Result<Group, VOTableError> {
    match self {
      Self::Group(x) => Ok(x),
      _ => Err(self.wrong_type(Group::TAG)),
    }
  }
  pub fn table_group(self) -> Result<TableGroup, VOTableError> {
    match self {
      Self::TableGroup(x) => Ok(x),
      _ => Err(self.wrong_type(TableGroup::TAG)),
    }
  }
  pub fn values(self) -> Result<Values, VOTableError> {
    match self {
      Self::Values(x) => Ok(x),
      _ => Err(self.wrong_type(Values::TAG)),
    }
  }
  pub fn option(self) -> Result<Opt, VOTableError> {
    match self {
      Self::Option(x) => Ok(x),
      _ => Err(self.wrong_type(Opt::TAG)),
    }
  }
  pub fn min(self) -> Result<Min, VOTableError> {
    match self {
      Self::Min(x) => Ok(x),
      _ => Err(self.wrong_type(Min::TAG)),
    }
  }
  pub fn max(self) -> Result<Max, VOTableError> {
    match self {
      Self::Max(x) => Ok(x),
      _ => Err(self.wrong_type(Max::TAG)),
    }
  }
  pub fn field_ref(self) -> Result<FieldRef, VOTableError> {
    match self {
      Self::FieldRef(x) => Ok(x),
      _ => Err(self.wrong_type(FieldRef::TAG)),
    }
  }
  pub fn param_ref(self) -> Result<ParamRef, VOTableError> {
    match self {
      Self::ParamRef(x) => Ok(x),
      _ => Err(self.wrong_type(ParamRef::TAG)),
    }
  }

  pub fn set_attributes<K, V, I>(&mut self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    self.as_mut().set_attributes(attrs)
  }

  pub fn set_content<S: Into<String>>(&mut self, content: S) -> Result<(), VOTableError> {
    self.as_mut().set_content(content)
  }

  pub fn set_description(&mut self, description: Description) -> Result<(), VOTableError> {
    self.as_mut().set_description(description)
  }

  pub fn set_definitions(&mut self, definitions: Definitions) -> Result<(), VOTableError> {
    self.as_mut().set_definitions(definitions)
  }

  pub fn set_values(&mut self, values: Values) -> Result<(), VOTableError> {
    self.as_mut().set_values(values)
  }

  pub fn set_min(&mut self, min: Min) -> Result<(), VOTableError> {
    self.as_mut().set_min(min)
  }

  pub fn set_max(&mut self, max: Max) -> Result<(), VOTableError> {
    self.as_mut().set_max(max)
  }

  pub fn push_option(&mut self, opt: Opt) -> Result<(), VOTableError> {
    self.as_mut().push_option(opt)
  }

  pub fn push_info(&mut self, info: Info) -> Result<(), VOTableError> {
    self.as_mut().push_info(info)
  }

  pub fn push_post_info(&mut self, info: Info) -> Result<(), VOTableError> {
    self.as_mut().push_post_info(info)
  }

  pub fn push_coosys(&mut self, coosys: CooSys) -> Result<(), VOTableError> {
    self.as_mut().push_coosys(coosys)
  }

  pub fn push_timesys(&mut self, timesys: TimeSys) -> Result<(), VOTableError> {
    self.as_mut().push_timesys(timesys)
  }

  pub fn push_link(&mut self, link: Link) -> Result<(), VOTableError> {
    self.as_mut().push_link(link)
  }

  pub fn push_field(&mut self, field: Field) -> Result<(), VOTableError> {
    self.as_mut().push_field(field)
  }

  pub fn push_param(&mut self, param: Param) -> Result<(), VOTableError> {
    self.as_mut().push_param(param)
  }

  pub fn push_fieldref(&mut self, fieldref: FieldRef) -> Result<(), VOTableError> {
    self.as_mut().push_fieldref(fieldref)
  }

  pub fn push_paramref(&mut self, paramref: ParamRef) -> Result<(), VOTableError> {
    self.as_mut().push_paramref(paramref)
  }

  pub fn push_group(&mut self, group: Group) -> Result<(), VOTableError> {
    self.as_mut().push_group(group)
  }

  pub fn push_tablegroup(&mut self, group: TableGroup) -> Result<(), VOTableError> {
    self.as_mut().push_tablegroup(group)
  }

  pub fn push_resource(&mut self, resource: Resource<C>) -> Result<(), VOTableError> {
    self.as_mut().push_resource(resource)
  }

  pub fn prepend_resource(&mut self, resource: Resource<C>) -> Result<(), VOTableError> {
    self.as_mut().prepend_resource(resource)
  }

  pub fn push_table(&mut self, table: Table<C>) -> Result<(), VOTableError> {
    self.as_mut().push_table(table)
  }

  pub fn set_data(&mut self, data: Data<C>) -> Result<(), VOTableError> {
    self.as_mut().set_data(data)
  }

  pub fn set_binary(&mut self, binary: Binary<C>) -> Result<(), VOTableError> {
    self.as_mut().set_binary(binary)
  }

  pub fn set_binary2(&mut self, binary2: Binary2<C>) -> Result<(), VOTableError> {
    self.as_mut().set_binary2(binary2)
  }

  pub fn set_fits(&mut self, fits: Fits) -> Result<(), VOTableError> {
    self.as_mut().set_fits(fits)
  }

  pub fn set_stream(&mut self, stream: Stream<C>) -> Result<(), VOTableError> {
    self.as_mut().set_stream(stream)
  }

  pub fn set_fits_stream(
    &mut self,
    stream: Stream<VoidTableDataContent>,
  ) -> Result<(), VOTableError> {
    self.as_mut().set_fits_stream(stream)
  }

  pub fn push(&mut self, elem: VOTableWrappedElem<C>) -> Result<(), VOTableError> {
    self.as_mut().push(elem)
  }
}

pub enum VOTableWrappedElemMut<'a, C: TableDataContent> {
  VOTable(&'a mut VOTable<C>),
  Resource(&'a mut Resource<C>),
  Table(&'a mut Table<C>),
  Data(&'a mut Data<C>),
  Binary(&'a mut Binary<C>),
  Binary2(&'a mut Binary2<C>),
  Fits(&'a mut Fits),
  Stream(&'a mut Stream<C>),
  FitsStream(&'a mut Stream<VoidTableDataContent>),
  Description(&'a mut Description),
  Definitions(&'a mut Definitions),
  Info(&'a mut Info),
  CooSys(&'a mut CooSys),
  TimeSys(&'a mut TimeSys),
  Field(&'a mut Field),
  Link(&'a mut Link),
  Param(&'a mut Param),
  Group(&'a mut Group),
  TableGroup(&'a mut TableGroup),
  Values(&'a mut Values),
  Option(&'a mut Opt),
  Min(&'a mut Min),
  Max(&'a mut Max),
  FieldRef(&'a mut FieldRef),
  ParamRef(&'a mut ParamRef),
  // MIVOT related tags
  // Vodml(Vodml),
  // REPORT
  // MODEL
  // GLOBAL
  // TEMPLATE
  // ATTRIBUTE / COLLECTION / INSTANCE / REFERENCE
  // JOIN / WHERE / PRIMARY_KEY / FOREIGN_KEY
}

impl<'a, C: TableDataContent> From<&'a mut VOTable<C>> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut VOTable<C>) -> Self {
    Self::VOTable(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Resource<C>> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Resource<C>) -> Self {
    Self::Resource(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Table<C>> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Table<C>) -> Self {
    Self::Table(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Data<C>> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Data<C>) -> Self {
    Self::Data(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Stream<C>> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Stream<C>) -> Self {
    Self::Stream(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Fits> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Fits) -> Self {
    Self::Fits(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Description> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Description) -> Self {
    Self::Description(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Definitions> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Definitions) -> Self {
    Self::Definitions(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Info> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Info) -> Self {
    Self::Info(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut CooSys> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut CooSys) -> Self {
    Self::CooSys(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut TimeSys> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut TimeSys) -> Self {
    Self::TimeSys(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Field> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Field) -> Self {
    Self::Field(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Link> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Link) -> Self {
    Self::Link(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Param> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Param) -> Self {
    Self::Param(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Group> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Group) -> Self {
    Self::Group(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut TableGroup> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut TableGroup) -> Self {
    Self::TableGroup(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Values> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Values) -> Self {
    Self::Values(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Opt> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Opt) -> Self {
    Self::Option(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Min> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Min) -> Self {
    Self::Min(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut Max> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut Max) -> Self {
    Self::Max(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut FieldRef> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut FieldRef) -> Self {
    Self::FieldRef(value)
  }
}
impl<'a, C: TableDataContent> From<&'a mut ParamRef> for VOTableWrappedElemMut<'a, C> {
  fn from(value: &'a mut ParamRef) -> Self {
    Self::ParamRef(value)
  }
}

impl<'a, C: TableDataContent> VOTableWrappedElemMut<'a, C> {
  pub fn tag(self) -> &'static str {
    match self {
      Self::VOTable(e) => e.tag(),
      Self::Resource(e) => e.tag(),
      Self::Table(e) => e.tag(),
      Self::Data(e) => e.tag(),
      Self::Binary(e) => e.tag(),
      Self::Binary2(e) => e.tag(),
      Self::Stream(e) => e.tag(),
      Self::FitsStream(e) => e.tag(),
      Self::Fits(e) => e.tag(),
      Self::Description(e) => e.tag(),
      Self::Definitions(e) => e.tag(),
      Self::Info(e) => e.tag(),
      Self::CooSys(e) => e.tag(),
      Self::TimeSys(e) => e.tag(),
      Self::Field(e) => e.tag(),
      Self::Link(e) => e.tag(),
      Self::Param(e) => e.tag(),
      Self::Group(e) => e.tag(),
      Self::TableGroup(e) => e.tag(),
      Self::Values(e) => e.tag(),
      Self::Option(e) => e.tag(),
      Self::Min(e) => e.tag(),
      Self::Max(e) => e.tag(),
      Self::FieldRef(e) => e.tag(),
      Self::ParamRef(e) => e.tag(),
    }
  }

  pub fn set_attributes<K, V, I>(self, attrs: I) -> Result<(), VOTableError>
  where
    K: AsRef<str> + Into<String>,
    V: AsRef<str> + Into<String>,
    I: Iterator<Item = (K, V)>,
  {
    match self {
      Self::VOTable(e) => e.set_attrs_by_ref(attrs),
      Self::Resource(e) => e.set_attrs_by_ref(attrs),
      Self::Table(e) => e.set_attrs_by_ref(attrs),
      Self::Data(e) => e.set_attrs_by_ref(attrs),
      Self::Binary(e) => e.set_attrs_by_ref(attrs),
      Self::Binary2(e) => e.set_attrs_by_ref(attrs),
      Self::Stream(e) => e.set_attrs_by_ref(attrs),
      Self::FitsStream(e) => e.set_attrs_by_ref(attrs),
      Self::Fits(e) => e.set_attrs_by_ref(attrs),
      Self::Description(e) => e.set_attrs_by_ref(attrs),
      Self::Definitions(e) => e.set_attrs_by_ref(attrs),
      Self::Info(e) => e.set_attrs_by_ref(attrs),
      Self::CooSys(e) => e.set_attrs_by_ref(attrs),
      Self::TimeSys(e) => e.set_attrs_by_ref(attrs),
      Self::Field(e) => e.set_attrs_by_ref(attrs),
      Self::Link(e) => e.set_attrs_by_ref(attrs),
      Self::Param(e) => e.set_attrs_by_ref(attrs),
      Self::Group(e) => e.set_attrs_by_ref(attrs),
      Self::TableGroup(e) => e.set_attrs_by_ref(attrs),
      Self::Values(e) => e.set_attrs_by_ref(attrs),
      Self::Option(e) => e.set_attrs_by_ref(attrs),
      Self::Min(e) => e.set_attrs_by_ref(attrs),
      Self::Max(e) => e.set_attrs_by_ref(attrs),
      Self::FieldRef(e) => e.set_attrs_by_ref(attrs),
      Self::ParamRef(e) => e.set_attrs_by_ref(attrs),
    }
  }

  pub fn set_content<S: Into<String>>(self, content: S) -> Result<(), VOTableError> {
    match self {
      Self::Description(e) => Ok(e.set_content_by_ref(content)),
      Self::Info(e) => Ok(e.set_content_by_ref(content)),
      Self::Link(e) => Ok(e.set_content_by_ref(content)),
      Self::ParamRef(e) => Ok(e.set_content_by_ref(content)),
      Self::FieldRef(e) => Ok(e.set_content_by_ref(content)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot have a content.",
        self.tag(),
      ))),
    }
  }

  pub fn set_description(self, description: Description) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.reset_description_by_ref(description)),
      Self::Resource(e) => Ok(e.reset_description_by_ref(description)),
      Self::Table(e) => Ok(e.reset_description_by_ref(description)),
      Self::Field(e) => Ok(e.reset_description_by_ref(description)),
      Self::Param(e) => Ok(e.reset_description_by_ref(description)),
      Self::Group(e) => Ok(e.reset_description_by_ref(description)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        description.tag()
      ))),
    }
  }

  pub fn set_definitions(self, definitions: Definitions) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.reset_definitions_by_ref(definitions)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        definitions.tag()
      ))),
    }
  }

  pub fn set_values(self, values: Values) -> Result<(), VOTableError> {
    match self {
      Self::Field(e) => Ok(e.reset_values_by_ref(values)),
      Self::Param(e) => Ok(e.reset_values_by_ref(values)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        values.tag()
      ))),
    }
  }

  pub fn set_min(self, min: Min) -> Result<(), VOTableError> {
    match self {
      Self::Values(e) => Ok(e.reset_min_by_ref(min)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        min.tag()
      ))),
    }
  }

  pub fn set_max(self, max: Max) -> Result<(), VOTableError> {
    match self {
      Self::Values(e) => Ok(e.reset_max_by_ref(max)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        max.tag()
      ))),
    }
  }

  pub fn push_option(self, opt: Opt) -> Result<(), VOTableError> {
    match self {
      Self::Values(e) => Ok(e.push_opt_by_ref(opt)),
      Self::Option(e) => Ok(e.push_opt_by_ref(opt)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        opt.tag()
      ))),
    }
  }

  pub fn push_info(self, info: Info) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_info_by_ref(info)),
      Self::Resource(e) => Ok(e.push_info_by_ref(info)),
      Self::Table(e) => Ok(e.push_info_by_ref(info)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        info.tag()
      ))),
    }
  }

  pub fn push_post_info(self, info: Info) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_post_info_by_ref(info)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a post-{} tag.",
        self.tag(),
        info.tag()
      ))),
    }
  }

  pub fn push_coosys(self, coosys: CooSys) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_coosys_by_ref(coosys)),
      Self::Resource(e) => Ok(e.push_coosys_by_ref(coosys)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {} tag.",
        self.tag(),
        coosys.tag()
      ))),
    }
  }

  pub fn push_timesys(self, timesys: TimeSys) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_timesys_by_ref(timesys)),
      Self::Resource(e) => Ok(e.push_timesys_by_ref(timesys)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        timesys.tag()
      ))),
    }
  }

  pub fn push_link(self, link: Link) -> Result<(), VOTableError> {
    match self {
      Self::Resource(e) => e.push_link_by_ref(link),
      Self::Table(e) => Ok(e.push_link_by_ref(link)),
      Self::Field(e) => Ok(e.push_link_by_ref(link)),
      Self::Param(e) => Ok(e.push_link_by_ref(link)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        link.tag()
      ))),
    }
  }

  pub fn push_field(self, field: Field) -> Result<(), VOTableError> {
    match self {
      Self::Table(e) => Ok(e.push_field_by_ref(field)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        field.tag()
      ))),
    }
  }

  pub fn push_param(self, param: Param) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_param_by_ref(param)),
      Self::Resource(e) => Ok(e.push_param_by_ref(param)),
      Self::Table(e) => Ok(e.push_param_by_ref(param)),
      Self::Group(e) => Ok(e.push_param_by_ref(param)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        param.tag()
      ))),
    }
  }

  pub fn push_fieldref(self, fieldref: FieldRef) -> Result<(), VOTableError> {
    match self {
      Self::CooSys(e) => Ok(e.push_fieldref_by_ref(fieldref)),
      Self::TableGroup(e) => Ok(e.push_fieldref_by_ref(fieldref)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        fieldref.tag()
      ))),
    }
  }

  pub fn push_paramref(self, paramref: ParamRef) -> Result<(), VOTableError> {
    match self {
      Self::CooSys(e) => Ok(e.push_paramref_by_ref(paramref)),
      Self::Group(e) => Ok(e.push_paramref_by_ref(paramref)),
      Self::TableGroup(e) => Ok(e.push_paramref_by_ref(paramref)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        paramref.tag()
      ))),
    }
  }

  pub fn push_group(self, group: Group) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_group_by_ref(group)),
      Self::Resource(e) => Ok(e.push_group_by_ref(group)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        group.tag()
      ))),
    }
  }

  pub fn push_tablegroup(self, group: TableGroup) -> Result<(), VOTableError> {
    match self {
      Self::Table(e) => Ok(e.push_tablegroup_by_ref(group)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        group.tag()
      ))),
    }
  }

  pub fn push_resource(self, resource: Resource<C>) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.push_resource_by_ref(resource)),
      Self::Resource(e) => Ok(e.push_resource_by_ref(resource)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        resource.tag()
      ))),
    }
  }

  pub fn prepend_resource(self, resource: Resource<C>) -> Result<(), VOTableError> {
    match self {
      Self::VOTable(e) => Ok(e.prepend_resource_by_ref(resource)),
      Self::Resource(e) => Ok(e.prepend_resource_by_ref(resource)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        resource.tag()
      ))),
    }
  }

  pub fn push_table(self, table: Table<C>) -> Result<(), VOTableError> {
    match self {
      Self::Resource(e) => Ok(e.push_table_by_ref(table)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        table.tag()
      ))),
    }
  }

  pub fn set_data(self, data: Data<C>) -> Result<(), VOTableError> {
    match self {
      Self::Table(e) => Ok(e.set_data_by_ref(data)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        data.tag()
      ))),
    }
  }

  pub fn set_binary(self, binary: Binary<C>) -> Result<(), VOTableError> {
    match self {
      Self::Data(e) => Ok(e.set_binary_by_ref(binary)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        binary.tag()
      ))),
    }
  }

  pub fn set_binary2(self, binary: Binary2<C>) -> Result<(), VOTableError> {
    match self {
      Self::Data(e) => Ok(e.set_binary2_by_ref(binary)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        binary.tag()
      ))),
    }
  }

  pub fn set_fits(self, fits: Fits) -> Result<(), VOTableError> {
    match self {
      Self::Data(e) => Ok(e.set_fits_by_ref(fits)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        fits.tag()
      ))),
    }
  }

  pub fn set_stream(self, stream: Stream<C>) -> Result<(), VOTableError> {
    match self {
      Self::Binary(e) => Ok(e.set_stream_by_ref(stream)),
      Self::Binary2(e) => Ok(e.set_stream_by_ref(stream)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        stream.tag()
      ))),
    }
  }

  pub fn set_fits_stream(self, stream: Stream<VoidTableDataContent>) -> Result<(), VOTableError> {
    match self {
      Self::Fits(e) => Ok(e.set_stream_by_ref(stream)),
      _ => Err(VOTableError::Custom(format!(
        "Tag '{}' cannot contain a {}.",
        self.tag(),
        stream.tag()
      ))),
    }
  }

  pub fn push(self, elem: VOTableWrappedElem<C>) -> Result<(), VOTableError> {
    match elem {
      VOTableWrappedElem::<C>::VOTable(_) => Err(VOTableError::Custom(
        "A VOTable cannot be pushed into another element.".into(),
      )),
      VOTableWrappedElem::<C>::Resource(e) => self.push_resource(e),
      VOTableWrappedElem::<C>::Table(e) => self.push_table(e),
      VOTableWrappedElem::<C>::Data(e) => self.set_data(e),
      VOTableWrappedElem::<C>::Binary(e) => self.set_binary(e),
      VOTableWrappedElem::<C>::Binary2(e) => self.set_binary2(e),
      VOTableWrappedElem::<C>::Stream(e) => self.set_stream(e),
      VOTableWrappedElem::<C>::FitsStream(e) => self.set_fits_stream(e),
      VOTableWrappedElem::<C>::Fits(e) => self.set_fits(e),
      VOTableWrappedElem::<C>::Description(e) => self.set_description(e),
      VOTableWrappedElem::<C>::Definitions(e) => self.set_definitions(e),
      VOTableWrappedElem::<C>::Info(e) => self.push_info(e),
      VOTableWrappedElem::<C>::CooSys(e) => self.push_coosys(e),
      VOTableWrappedElem::<C>::TimeSys(e) => self.push_timesys(e),
      VOTableWrappedElem::<C>::Field(e) => self.push_field(e),
      VOTableWrappedElem::<C>::Link(e) => self.push_link(e),
      VOTableWrappedElem::<C>::Param(e) => self.push_param(e),
      VOTableWrappedElem::<C>::Group(e) => self.push_group(e),
      VOTableWrappedElem::<C>::TableGroup(e) => self.push_tablegroup(e),
      VOTableWrappedElem::<C>::Values(e) => self.set_values(e),
      VOTableWrappedElem::<C>::Option(e) => self.push_option(e),
      VOTableWrappedElem::<C>::Min(e) => self.set_min(e),
      VOTableWrappedElem::<C>::Max(e) => self.set_max(e),
      VOTableWrappedElem::<C>::FieldRef(e) => self.push_fieldref(e),
      VOTableWrappedElem::<C>::ParamRef(e) => self.push_paramref(e),
    }
  }
}
