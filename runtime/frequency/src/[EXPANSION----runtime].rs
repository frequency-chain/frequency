// Recursive expansion of construct_runtime! macro
// ================================================

#[doc(hidden)]
mod sp_api_hidden_includes_construct_runtime {
  pub extern crate frame_support as hidden_include;
}const _:() = {
  #[allow(unused)]
  type __hidden_use_of_unchecked_extrinsic = UncheckedExtrinsic;
};
#[derive(Clone,Copy,PartialEq,Eq,self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::RuntimeDebug,self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::TypeInfo)]
pub struct Runtime;

impl self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::GetNodeBlockType for Runtime {
  type NodeBlock = opaque::Block;
}
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::GetRuntimeBlockType for Runtime {
  type RuntimeBlock = Block;
}
#[derive(Clone,PartialEq,Eq,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::Encode,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::Decode,self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::TypeInfo,self::sp_api_hidden_includes_construct_runtime::hidden_include::RuntimeDebug,)]
#[allow(non_camel_case_types)]
pub enum RuntimeEvent {
  #[codec(index = 0u8)]
  System(frame_system::Event<Runtime>), #[codec(index = 1u8)]
  ParachainSystem(cumulus_pallet_parachain_system::Event<Runtime>), #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
  #[codec(index = 4u8)]
  Sudo(pallet_sudo::Event<Runtime>), #[codec(index = 5u8)]
  Preimage(pallet_preimage::Event<Runtime>), #[codec(index = 6u8)]
  Democracy(pallet_democracy::Event<Runtime>), #[codec(index = 8u8)]
  Scheduler(pallet_scheduler::Event<Runtime>), #[codec(index = 9u8)]
  Utility(pallet_utility::Event), #[codec(index = 10u8)]
  Balances(pallet_balances::Event<Runtime>), #[codec(index = 11u8)]
  TransactionPayment(pallet_transaction_payment::Event<Runtime>), #[codec(index = 12u8)]
  Council(pallet_collective::Event<Runtime,pallet_collective::Instance1>), #[codec(index = 13u8)]
  TechnicalCommittee(pallet_collective::Event<Runtime,pallet_collective::Instance2>), #[codec(index = 14u8)]
  Treasury(pallet_treasury::Event<Runtime>), #[codec(index = 21u8)]
  CollatorSelection(pallet_collator_selection::Event<Runtime>), #[codec(index = 22u8)]
  Session(pallet_session::Event), #[codec(index = 30u8)]
  Multisig(pallet_multisig::Event<Runtime>), #[codec(index = 40u8)]
  TimeRelease(orml_vesting::Event<Runtime>), #[codec(index = 60u8)]
  Msa(pallet_msa::Event<Runtime>), #[codec(index = 61u8)]
  Messages(pallet_messages::Event<Runtime>), #[codec(index = 62u8)]
  Schemas(pallet_schemas::Event<Runtime>), #[codec(index = 63u8)]
  Capacity(pallet_capacity::Event<Runtime>), #[codec(index = 64u8)]
  FrequencyTxPayment(pallet_frequency_tx_payment::Event<Runtime>),
}
impl From<frame_system::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:frame_system::Event:: <Runtime>) -> Self {
    RuntimeEvent::System(x)
  }

  }
impl TryInto<frame_system::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<frame_system::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::System(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<cumulus_pallet_parachain_system::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:cumulus_pallet_parachain_system::Event:: <Runtime>) -> Self {
    RuntimeEvent::ParachainSystem(x)
  }

  }
impl TryInto<cumulus_pallet_parachain_system::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<cumulus_pallet_parachain_system::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::ParachainSystem(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
impl From<pallet_sudo::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_sudo::Event:: <Runtime>) -> Self {
    RuntimeEvent::Sudo(x)
  }

  }
#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
impl TryInto<pallet_sudo::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_sudo::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Sudo(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_preimage::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_preimage::Event:: <Runtime>) -> Self {
    RuntimeEvent::Preimage(x)
  }

  }
impl TryInto<pallet_preimage::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_preimage::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Preimage(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_democracy::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_democracy::Event:: <Runtime>) -> Self {
    RuntimeEvent::Democracy(x)
  }

  }
impl TryInto<pallet_democracy::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_democracy::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Democracy(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_scheduler::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_scheduler::Event:: <Runtime>) -> Self {
    RuntimeEvent::Scheduler(x)
  }

  }
impl TryInto<pallet_scheduler::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_scheduler::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Scheduler(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_utility::Event>for RuntimeEvent {
  fn from(x:pallet_utility::Event) -> Self {
    RuntimeEvent::Utility(x)
  }

  }
impl TryInto<pallet_utility::Event>for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_utility::Event,Self::Error>{
    match self {
      Self::Utility(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_balances::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_balances::Event:: <Runtime>) -> Self {
    RuntimeEvent::Balances(x)
  }

  }
impl TryInto<pallet_balances::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_balances::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Balances(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_transaction_payment::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_transaction_payment::Event:: <Runtime>) -> Self {
    RuntimeEvent::TransactionPayment(x)
  }

  }
impl TryInto<pallet_transaction_payment::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_transaction_payment::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::TransactionPayment(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_collective::Event:: <Runtime,pallet_collective::Instance1> >for RuntimeEvent {
  fn from(x:pallet_collective::Event:: <Runtime,pallet_collective::Instance1>) -> Self {
    RuntimeEvent::Council(x)
  }

  }
impl TryInto<pallet_collective::Event:: <Runtime,pallet_collective::Instance1> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collective::Event:: <Runtime,pallet_collective::Instance1> ,Self::Error>{
    match self {
      Self::Council(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_collective::Event:: <Runtime,pallet_collective::Instance2> >for RuntimeEvent {
  fn from(x:pallet_collective::Event:: <Runtime,pallet_collective::Instance2>) -> Self {
    RuntimeEvent::TechnicalCommittee(x)
  }

  }
impl TryInto<pallet_collective::Event:: <Runtime,pallet_collective::Instance2> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collective::Event:: <Runtime,pallet_collective::Instance2> ,Self::Error>{
    match self {
      Self::TechnicalCommittee(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_treasury::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_treasury::Event:: <Runtime>) -> Self {
    RuntimeEvent::Treasury(x)
  }

  }
impl TryInto<pallet_treasury::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_treasury::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Treasury(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_collator_selection::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_collator_selection::Event:: <Runtime>) -> Self {
    RuntimeEvent::CollatorSelection(x)
  }

  }
impl TryInto<pallet_collator_selection::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collator_selection::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::CollatorSelection(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_session::Event>for RuntimeEvent {
  fn from(x:pallet_session::Event) -> Self {
    RuntimeEvent::Session(x)
  }

  }
impl TryInto<pallet_session::Event>for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_session::Event,Self::Error>{
    match self {
      Self::Session(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_multisig::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_multisig::Event:: <Runtime>) -> Self {
    RuntimeEvent::Multisig(x)
  }

  }
impl TryInto<pallet_multisig::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_multisig::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Multisig(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<orml_vesting::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:orml_vesting::Event:: <Runtime>) -> Self {
    RuntimeEvent::TimeRelease(x)
  }

  }
impl TryInto<orml_vesting::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<orml_vesting::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::TimeRelease(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_msa::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_msa::Event:: <Runtime>) -> Self {
    RuntimeEvent::Msa(x)
  }

  }
impl TryInto<pallet_msa::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_msa::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Msa(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_messages::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_messages::Event:: <Runtime>) -> Self {
    RuntimeEvent::Messages(x)
  }

  }
impl TryInto<pallet_messages::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_messages::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Messages(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_schemas::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_schemas::Event:: <Runtime>) -> Self {
    RuntimeEvent::Schemas(x)
  }

  }
impl TryInto<pallet_schemas::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_schemas::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Schemas(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_capacity::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_capacity::Event:: <Runtime>) -> Self {
    RuntimeEvent::Capacity(x)
  }

  }
impl TryInto<pallet_capacity::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_capacity::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::Capacity(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
impl From<pallet_frequency_tx_payment::Event:: <Runtime> >for RuntimeEvent {
  fn from(x:pallet_frequency_tx_payment::Event:: <Runtime>) -> Self {
    RuntimeEvent::FrequencyTxPayment(x)
  }

  }
impl TryInto<pallet_frequency_tx_payment::Event:: <Runtime> >for RuntimeEvent {
  type Error = ();
  fn try_into(self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_frequency_tx_payment::Event:: <Runtime> ,Self::Error>{
    match self {
      Self::FrequencyTxPayment(evt) => Ok(evt),
      _ => Err(()),
    
      }
  }

  }
#[doc = r" The runtime origin type representing the origin of a call."]
#[doc = r""]
#[doc = " Origin is always created with the base filter configured in [`frame_system::Config::BaseCallFilter`]."]
#[derive(Clone)]
pub struct RuntimeOrigin {
  caller:OriginCaller,
  filter:self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::rc::Rc<Box<dyn Fn(& <Runtime as frame_system::Config> ::RuntimeCall) -> bool>> ,
}
#[cfg(not(feature = "std"))]
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::fmt::Debug for RuntimeOrigin {
  fn fmt(&self,fmt: &mut self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::fmt::Formatter,) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<(),self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::fmt::Error>{
    fmt.write_str("<wasm:stripped>")
  }

  }
#[cfg(feature = "std")]
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::fmt::Debug for RuntimeOrigin {
  fn fmt(&self,fmt: &mut self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::fmt::Formatter,) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<(),self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::fmt::Error>{
    fmt.debug_struct("Origin").field("caller", &self.caller).field("filter", &"[function ptr]").finish()
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait for RuntimeOrigin {
  type Call =  <Runtime as frame_system::Config> ::RuntimeCall;
  type PalletsOrigin = OriginCaller;
  type AccountId =  <Runtime as frame_system::Config> ::AccountId;
  fn add_filter(&mut self,filter:impl Fn(&Self::Call) -> bool+'static){
    let f = self.filter.clone();
    self.filter = self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::rc::Rc::new(Box::new(move|call|{
      f(call)&&filter(call)
    }));
  }
  fn reset_filter(&mut self){
    let filter =  < <Runtime as frame_system::Config> ::BaseCallFilter as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::Contains<<Runtime as frame_system::Config> ::RuntimeCall> > ::contains;
    self.filter = self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::rc::Rc::new(Box::new(filter));
  }
  fn set_caller_from(&mut self,other:impl Into<Self>){
    self.caller = other.into().caller;
  }
  fn filter_call(&self,call: &Self::Call) -> bool {
    match self.caller {
      OriginCaller::system(frame_system::Origin:: <Runtime> ::Root) => true,
      _ => (self.filter)(call),
    
      }
  }
  fn caller(&self) ->  &Self::PalletsOrigin {
    &self.caller
  }
  fn into_caller(self) -> Self::PalletsOrigin {
    self.caller
  }
  fn try_with_caller<R>(mut self,f:impl FnOnce(Self::PalletsOrigin) -> Result<R,Self::PalletsOrigin> ,) -> Result<R,Self>{
    match f(self.caller){
      Ok(r) => Ok(r),
      Err(caller) => {
        self.caller = caller;
        Err(self)
      }
    
      }
  }
  fn none() -> Self {
    frame_system::RawOrigin::None.into()
  }
  fn root() -> Self {
    frame_system::RawOrigin::Root.into()
  }
  fn signed(by:Self::AccountId) -> Self {
    frame_system::RawOrigin::Signed(by).into()
  }

  }
#[derive(Clone,PartialEq,Eq,self::sp_api_hidden_includes_construct_runtime::hidden_include::RuntimeDebug,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::Encode,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::Decode,self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::TypeInfo,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::MaxEncodedLen,)]
#[allow(non_camel_case_types)]
pub enum OriginCaller {
  #[codec(index = 0u8)]
  system(frame_system::Origin<Runtime>), #[codec(index = 12u8)]
  Council(pallet_collective::Origin<Runtime,pallet_collective::Instance1>), #[codec(index = 13u8)]
  TechnicalCommittee(pallet_collective::Origin<Runtime,pallet_collective::Instance2>), #[allow(dead_code)]
  Void(self::sp_api_hidden_includes_construct_runtime::hidden_include::Void)
}
#[allow(dead_code)]
impl RuntimeOrigin {
  #[doc = " Create with system none origin and [`frame_system::Config::BaseCallFilter`]."]
  pub fn none() -> Self {
    <RuntimeOrigin as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait> ::none()
  }
  #[doc = " Create with system root origin and [`frame_system::Config::BaseCallFilter`]."]
  pub fn root() -> Self {
    <RuntimeOrigin as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait> ::root()
  }
  #[doc = " Create with system signed origin and [`frame_system::Config::BaseCallFilter`]."]
  pub fn signed(by: <Runtime as frame_system::Config> ::AccountId) -> Self {
    <RuntimeOrigin as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait> ::signed(by)
  }

  }
impl From<frame_system::Origin<Runtime>>for OriginCaller {
  fn from(x:frame_system::Origin<Runtime>) -> Self {
    OriginCaller::system(x)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::CallerTrait<<Runtime as frame_system::Config> ::AccountId>for OriginCaller {
  fn into_system(self) -> Option<frame_system::RawOrigin<<Runtime as frame_system::Config> ::AccountId>>{
    match self {
      OriginCaller::system(x) => Some(x),
      _ => None,
    
      }
  }
  fn as_system_ref(&self) -> Option< &frame_system::RawOrigin<<Runtime as frame_system::Config> ::AccountId>>{
    match&self {
      OriginCaller::system(o) => Some(o),
      _ => None,
    
      }
  }

  }
impl TryFrom<OriginCaller>for frame_system::Origin<Runtime>{
  type Error = OriginCaller;
  fn try_from(x:OriginCaller) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<frame_system::Origin<Runtime> ,OriginCaller>{
    if let OriginCaller::system(l) = x {
      Ok(l)
    }else {
      Err(x)
    }
  }

  }
impl From<frame_system::Origin<Runtime>>for RuntimeOrigin {
  #[doc = " Convert to runtime origin, using as filter: [`frame_system::Config::BaseCallFilter`]."]
  fn from(x:frame_system::Origin<Runtime>) -> Self {
    let o:OriginCaller = x.into();
    o.into()
  }

  }
impl From<OriginCaller>for RuntimeOrigin {
  fn from(x:OriginCaller) -> Self {
    let mut o = RuntimeOrigin {
      caller:x,filter:self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::rc::Rc::new(Box::new(|_|true)),
    };
    self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait::reset_filter(&mut o);
    o
  }

  }
impl From<RuntimeOrigin>for self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<frame_system::Origin<Runtime> ,RuntimeOrigin>{
  #[doc = r" NOTE: converting to pallet origin loses the origin filter information."]
  fn from(val:RuntimeOrigin) -> Self {
    if let OriginCaller::system(l) = val.caller {
      Ok(l)
    }else {
      Err(val)
    }
  }

  }
impl From<Option<<Runtime as frame_system::Config> ::AccountId>>for RuntimeOrigin {
  #[doc = " Convert to runtime origin with caller being system signed or none and use filter [`frame_system::Config::BaseCallFilter`]."]
  fn from(x:Option<<Runtime as frame_system::Config> ::AccountId>) -> Self {
    <frame_system::Origin<Runtime>> ::from(x).into()
  }

  }
impl From<pallet_collective::Origin<Runtime,pallet_collective::Instance1> >for OriginCaller {
  fn from(x:pallet_collective::Origin<Runtime,pallet_collective::Instance1>) -> Self {
    OriginCaller::Council(x)
  }

  }
impl From<pallet_collective::Origin<Runtime,pallet_collective::Instance1> >for RuntimeOrigin {
  #[doc = "  Convert to runtime origin using [`pallet_collective::Config::BaseCallFilter`]."]
  fn from(x:pallet_collective::Origin<Runtime,pallet_collective::Instance1>) -> Self {
    let x:OriginCaller = x.into();
    x.into()
  }

  }
impl From<RuntimeOrigin>for self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collective::Origin<Runtime,pallet_collective::Instance1> ,RuntimeOrigin>{
  #[doc = r" NOTE: converting to pallet origin loses the origin filter information."]
  fn from(val:RuntimeOrigin) -> Self {
    if let OriginCaller::Council(l) = val.caller {
      Ok(l)
    }else {
      Err(val)
    }
  }

  }
impl TryFrom<OriginCaller>for pallet_collective::Origin<Runtime,pallet_collective::Instance1>{
  type Error = OriginCaller;
  fn try_from(x:OriginCaller,) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collective::Origin<Runtime,pallet_collective::Instance1> ,OriginCaller>{
    if let OriginCaller::Council(l) = x {
      Ok(l)
    }else {
      Err(x)
    }
  }

  }
impl From<pallet_collective::Origin<Runtime,pallet_collective::Instance2> >for OriginCaller {
  fn from(x:pallet_collective::Origin<Runtime,pallet_collective::Instance2>) -> Self {
    OriginCaller::TechnicalCommittee(x)
  }

  }
impl From<pallet_collective::Origin<Runtime,pallet_collective::Instance2> >for RuntimeOrigin {
  #[doc = "  Convert to runtime origin using [`pallet_collective::Config::BaseCallFilter`]."]
  fn from(x:pallet_collective::Origin<Runtime,pallet_collective::Instance2>) -> Self {
    let x:OriginCaller = x.into();
    x.into()
  }

  }
impl From<RuntimeOrigin>for self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collective::Origin<Runtime,pallet_collective::Instance2> ,RuntimeOrigin>{
  #[doc = r" NOTE: converting to pallet origin loses the origin filter information."]
  fn from(val:RuntimeOrigin) -> Self {
    if let OriginCaller::TechnicalCommittee(l) = val.caller {
      Ok(l)
    }else {
      Err(val)
    }
  }

  }
impl TryFrom<OriginCaller>for pallet_collective::Origin<Runtime,pallet_collective::Instance2>{
  type Error = OriginCaller;
  fn try_from(x:OriginCaller,) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result<pallet_collective::Origin<Runtime,pallet_collective::Instance2> ,OriginCaller>{
    if let OriginCaller::TechnicalCommittee(l) = x {
      Ok(l)
    }else {
      Err(x)
    }
  }

  }
pub type System = frame_system::Pallet<Runtime> ;
pub type ParachainSystem = cumulus_pallet_parachain_system::Pallet<Runtime> ;
pub type Timestamp = pallet_timestamp::Pallet<Runtime> ;
pub type ParachainInfo = parachain_info::Pallet<Runtime> ;
#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
pub type Sudo = pallet_sudo::Pallet<Runtime> ;
pub type Preimage = pallet_preimage::Pallet<Runtime> ;
pub type Democracy = pallet_democracy::Pallet<Runtime> ;
pub type Scheduler = pallet_scheduler::Pallet<Runtime> ;
pub type Utility = pallet_utility::Pallet<Runtime> ;
pub type Balances = pallet_balances::Pallet<Runtime> ;
pub type TransactionPayment = pallet_transaction_payment::Pallet<Runtime> ;
pub type Council = pallet_collective::Pallet<Runtime,pallet_collective::Instance1> ;
pub type TechnicalCommittee = pallet_collective::Pallet<Runtime,pallet_collective::Instance2> ;
pub type Treasury = pallet_treasury::Pallet<Runtime> ;
pub type Authorship = pallet_authorship::Pallet<Runtime> ;
pub type CollatorSelection = pallet_collator_selection::Pallet<Runtime> ;
pub type Session = pallet_session::Pallet<Runtime> ;
pub type Aura = pallet_aura::Pallet<Runtime> ;
pub type AuraExt = cumulus_pallet_aura_ext::Pallet<Runtime> ;
pub type Multisig = pallet_multisig::Pallet<Runtime> ;
pub type Vesting = orml_vesting::Pallet<Runtime> ;
pub type Msa = pallet_msa::Pallet<Runtime> ;
pub type Messages = pallet_messages::Pallet<Runtime> ;
pub type Schemas = pallet_schemas::Pallet<Runtime> ;
pub type Capacity = pallet_capacity::Pallet<Runtime> ;
pub type FrequencyTxPayment = pallet_frequency_tx_payment::Pallet<Runtime> ;
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
#[deprecated(note = "The type definition has changed from representing all pallets \
			excluding system, in reversed order to become the representation of all pallets \
			including system pallet in regular order. For this reason it is encouraged to use \
			explicitly one of `AllPalletsWithSystem`, `AllPalletsWithoutSystem`, \
			`AllPalletsWithSystemReversed`, `AllPalletsWithoutSystemReversed`. \
			Note that the type `frame_executive::Executive` expects one of `AllPalletsWithSystem` \
			, `AllPalletsWithSystemReversed`, `AllPalletsReversedWithSystemFirst`. More details in \
			https://github.com/paritytech/substrate/pull/10043")]
pub type AllPallets = AllPalletsWithSystem;
#[cfg(all(not(feature = "all-frequency-features"),not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
pub type AllPalletsWithSystem = (System,ParachainSystem,Timestamp,ParachainInfo,Sudo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(feature = "all-frequency-features",not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
pub type AllPalletsWithSystem = (System,ParachainSystem,Timestamp,ParachainInfo,Sudo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(feature = "frequency",not(feature = "all-frequency-features")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
pub type AllPalletsWithSystem = (System,ParachainSystem,Timestamp,ParachainInfo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(feature = "all-frequency-features",feature = "frequency",))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
pub type AllPalletsWithSystem = (System,ParachainSystem,Timestamp,ParachainInfo,Sudo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(not(feature = "all-frequency-features"),not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
#[doc = r" Excludes the System pallet."]
pub type AllPalletsWithoutSystem = (ParachainSystem,Timestamp,ParachainInfo,Sudo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(feature = "all-frequency-features",not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
#[doc = r" Excludes the System pallet."]
pub type AllPalletsWithoutSystem = (ParachainSystem,Timestamp,ParachainInfo,Sudo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(feature = "frequency",not(feature = "all-frequency-features")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
#[doc = r" Excludes the System pallet."]
pub type AllPalletsWithoutSystem = (ParachainSystem,Timestamp,ParachainInfo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(feature = "all-frequency-features",feature = "frequency",))]
#[doc = r" All pallets included in the runtime as a nested tuple of types."]
#[doc = r" Excludes the System pallet."]
pub type AllPalletsWithoutSystem = (ParachainSystem,Timestamp,ParachainInfo,Sudo,Preimage,Democracy,Scheduler,Utility,Balances,TransactionPayment,Council,TechnicalCommittee,Treasury,Authorship,CollatorSelection,Session,Aura,AuraExt,Multisig,Vesting,Msa,Messages,Schemas,Capacity,FrequencyTxPayment,);
#[cfg(all(not(feature = "all-frequency-features"),not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,System,);
#[cfg(all(feature = "all-frequency-features",not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,System,);
#[cfg(all(feature = "frequency",not(feature = "all-frequency-features")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,ParachainInfo,Timestamp,ParachainSystem,System,);
#[cfg(all(feature = "all-frequency-features",feature = "frequency",))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,System,);
#[cfg(all(not(feature = "all-frequency-features"),not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" Excludes the System pallet."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithoutSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(feature = "all-frequency-features",not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" Excludes the System pallet."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithoutSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(feature = "frequency",not(feature = "all-frequency-features")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" Excludes the System pallet."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithoutSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(feature = "all-frequency-features",feature = "frequency",))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" Excludes the System pallet."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsWithoutSystemReversed = (FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(not(feature = "all-frequency-features"),not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" With the system pallet first."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsReversedWithSystemFirst = (System,FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(feature = "all-frequency-features",not(feature = "frequency")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" With the system pallet first."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsReversedWithSystemFirst = (System,FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(feature = "frequency",not(feature = "all-frequency-features")))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" With the system pallet first."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsReversedWithSystemFirst = (System,FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,ParachainInfo,Timestamp,ParachainSystem,);
#[cfg(all(feature = "all-frequency-features",feature = "frequency",))]
#[doc = r" All pallets included in the runtime as a nested tuple of types in reversed order."]
#[doc = r" With the system pallet first."]
#[deprecated(note = "Using reverse pallet orders is deprecated. use only \
			`AllPalletsWithSystem or AllPalletsWithoutSystem`")]
pub type AllPalletsReversedWithSystemFirst = (System,FrequencyTxPayment,Capacity,Schemas,Messages,Msa,Vesting,Multisig,AuraExt,Aura,Session,CollatorSelection,Authorship,Treasury,TechnicalCommittee,Council,TransactionPayment,Balances,Utility,Scheduler,Democracy,Preimage,Sudo,ParachainInfo,Timestamp,ParachainSystem,);
#[doc = r" Provides an implementation of `PalletInfo` to provide information"]
#[doc = r" about the pallet setup in the runtime."]
pub struct PalletInfo;

impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfo for PalletInfo {
  fn index<P:'static>() -> Option<usize>{
    let type_id = self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <P>();
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <System>(){
      return Some(0usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainSystem>(){
      return Some(1usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Timestamp>(){
      return Some(2usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainInfo>(){
      return Some(3usize)
    }#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Sudo>(){
      return Some(4usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Preimage>(){
      return Some(5usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Democracy>(){
      return Some(6usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Scheduler>(){
      return Some(8usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Utility>(){
      return Some(9usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Balances>(){
      return Some(10usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TransactionPayment>(){
      return Some(11usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Council>(){
      return Some(12usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TechnicalCommittee>(){
      return Some(13usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Treasury>(){
      return Some(14usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Authorship>(){
      return Some(20usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <CollatorSelection>(){
      return Some(21usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Session>(){
      return Some(22usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Aura>(){
      return Some(23usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <AuraExt>(){
      return Some(24usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Multisig>(){
      return Some(30usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Vesting>(){
      return Some(40usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Msa>(){
      return Some(60usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Messages>(){
      return Some(61usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Schemas>(){
      return Some(62usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Capacity>(){
      return Some(63usize)
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <FrequencyTxPayment>(){
      return Some(64usize)
    }None
  }
  fn name<P:'static>() -> Option< &'static str>{
    let type_id = self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <P>();
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <System>(){
      return Some("System")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainSystem>(){
      return Some("ParachainSystem")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Timestamp>(){
      return Some("Timestamp")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainInfo>(){
      return Some("ParachainInfo")
    }#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Sudo>(){
      return Some("Sudo")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Preimage>(){
      return Some("Preimage")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Democracy>(){
      return Some("Democracy")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Scheduler>(){
      return Some("Scheduler")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Utility>(){
      return Some("Utility")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Balances>(){
      return Some("Balances")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TransactionPayment>(){
      return Some("TransactionPayment")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Council>(){
      return Some("Council")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TechnicalCommittee>(){
      return Some("TechnicalCommittee")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Treasury>(){
      return Some("Treasury")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Authorship>(){
      return Some("Authorship")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <CollatorSelection>(){
      return Some("CollatorSelection")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Session>(){
      return Some("Session")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Aura>(){
      return Some("Aura")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <AuraExt>(){
      return Some("AuraExt")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Multisig>(){
      return Some("Multisig")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Vesting>(){
      return Some("Vesting")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Msa>(){
      return Some("Msa")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Messages>(){
      return Some("Messages")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Schemas>(){
      return Some("Schemas")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Capacity>(){
      return Some("Capacity")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <FrequencyTxPayment>(){
      return Some("FrequencyTxPayment")
    }None
  }
  fn module_name<P:'static>() -> Option< &'static str>{
    let type_id = self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <P>();
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <System>(){
      return Some("frame_system")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainSystem>(){
      return Some("cumulus_pallet_parachain_system")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Timestamp>(){
      return Some("pallet_timestamp")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainInfo>(){
      return Some("parachain_info")
    }#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Sudo>(){
      return Some("pallet_sudo")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Preimage>(){
      return Some("pallet_preimage")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Democracy>(){
      return Some("pallet_democracy")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Scheduler>(){
      return Some("pallet_scheduler")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Utility>(){
      return Some("pallet_utility")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Balances>(){
      return Some("pallet_balances")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TransactionPayment>(){
      return Some("pallet_transaction_payment")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Council>(){
      return Some("pallet_collective")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TechnicalCommittee>(){
      return Some("pallet_collective")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Treasury>(){
      return Some("pallet_treasury")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Authorship>(){
      return Some("pallet_authorship")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <CollatorSelection>(){
      return Some("pallet_collator_selection")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Session>(){
      return Some("pallet_session")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Aura>(){
      return Some("pallet_aura")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <AuraExt>(){
      return Some("cumulus_pallet_aura_ext")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Multisig>(){
      return Some("pallet_multisig")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Vesting>(){
      return Some("orml_vesting")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Msa>(){
      return Some("pallet_msa")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Messages>(){
      return Some("pallet_messages")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Schemas>(){
      return Some("pallet_schemas")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Capacity>(){
      return Some("pallet_capacity")
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <FrequencyTxPayment>(){
      return Some("pallet_frequency_tx_payment")
    }None
  }
  fn crate_version<P:'static>() -> Option<self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::CrateVersion>{
    let type_id = self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <P>();
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <System>(){
      return Some(<frame_system::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainSystem>(){
      return Some(<cumulus_pallet_parachain_system::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Timestamp>(){
      return Some(<pallet_timestamp::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <ParachainInfo>(){
      return Some(<parachain_info::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Sudo>(){
      return Some(<pallet_sudo::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Preimage>(){
      return Some(<pallet_preimage::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Democracy>(){
      return Some(<pallet_democracy::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Scheduler>(){
      return Some(<pallet_scheduler::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Utility>(){
      return Some(<pallet_utility::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Balances>(){
      return Some(<pallet_balances::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TransactionPayment>(){
      return Some(<pallet_transaction_payment::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Council>(){
      return Some(<pallet_collective::Pallet<Runtime,pallet_collective::Instance1>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <TechnicalCommittee>(){
      return Some(<pallet_collective::Pallet<Runtime,pallet_collective::Instance2>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Treasury>(){
      return Some(<pallet_treasury::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Authorship>(){
      return Some(<pallet_authorship::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <CollatorSelection>(){
      return Some(<pallet_collator_selection::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Session>(){
      return Some(<pallet_session::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Aura>(){
      return Some(<pallet_aura::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <AuraExt>(){
      return Some(<cumulus_pallet_aura_ext::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Multisig>(){
      return Some(<pallet_multisig::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Vesting>(){
      return Some(<orml_vesting::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Msa>(){
      return Some(<pallet_msa::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Messages>(){
      return Some(<pallet_messages::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Schemas>(){
      return Some(<pallet_schemas::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <Capacity>(){
      return Some(<pallet_capacity::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }if type_id==self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::any::TypeId::of:: <FrequencyTxPayment>(){
      return Some(<pallet_frequency_tx_payment::Pallet<Runtime>as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::PalletInfoAccess> ::crate_version())
    }None
  }

  }
#[derive(Clone,PartialEq,Eq,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::Encode,self::sp_api_hidden_includes_construct_runtime::hidden_include::codec::Decode,self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::TypeInfo,self::sp_api_hidden_includes_construct_runtime::hidden_include::RuntimeDebug,)]
pub enum RuntimeCall {
  #[codec(index = 0u8)]
  System(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<System,Runtime>), #[codec(index = 1u8)]
  ParachainSystem(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<ParachainSystem,Runtime>), #[codec(index = 2u8)]
  Timestamp(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Timestamp,Runtime>), #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
  #[codec(index = 4u8)]
  Sudo(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Sudo,Runtime>), #[codec(index = 5u8)]
  Preimage(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Preimage,Runtime>), #[codec(index = 6u8)]
  Democracy(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Democracy,Runtime>), #[codec(index = 8u8)]
  Scheduler(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Scheduler,Runtime>), #[codec(index = 9u8)]
  Utility(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Utility,Runtime>), #[codec(index = 10u8)]
  Balances(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Balances,Runtime>), #[codec(index = 12u8)]
  Council(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Council,Runtime>), #[codec(index = 13u8)]
  TechnicalCommittee(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<TechnicalCommittee,Runtime>), #[codec(index = 14u8)]
  Treasury(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Treasury,Runtime>), #[codec(index = 20u8)]
  Authorship(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Authorship,Runtime>), #[codec(index = 21u8)]
  CollatorSelection(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<CollatorSelection,Runtime>), #[codec(index = 22u8)]
  Session(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Session,Runtime>), #[codec(index = 30u8)]
  Multisig(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Multisig,Runtime>), #[codec(index = 40u8)]
  Vesting(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Vesting,Runtime>), #[codec(index = 60u8)]
  Msa(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Msa,Runtime>), #[codec(index = 61u8)]
  Messages(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Messages,Runtime>), #[codec(index = 62u8)]
  Schemas(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Schemas,Runtime>), #[codec(index = 63u8)]
  Capacity(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Capacity,Runtime>), #[codec(index = 64u8)]
  FrequencyTxPayment(self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<FrequencyTxPayment,Runtime>),
}
#[cfg(test)]
impl RuntimeCall {
  #[doc = r" Return a list of the module names together with their size in memory."]
  pub const fn sizes() ->  &'static[(&'static str,usize)]{
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::Callable;
    use core::mem::size_of;
    &[("System",size_of:: < <System as Callable<Runtime>> ::RuntimeCall>(),),("ParachainSystem",size_of:: < <ParachainSystem as Callable<Runtime>> ::RuntimeCall>(),),("Timestamp",size_of:: < <Timestamp as Callable<Runtime>> ::RuntimeCall>(),), #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    ("Sudo",size_of:: < <Sudo as Callable<Runtime>> ::RuntimeCall>(),),("Preimage",size_of:: < <Preimage as Callable<Runtime>> ::RuntimeCall>(),),("Democracy",size_of:: < <Democracy as Callable<Runtime>> ::RuntimeCall>(),),("Scheduler",size_of:: < <Scheduler as Callable<Runtime>> ::RuntimeCall>(),),("Utility",size_of:: < <Utility as Callable<Runtime>> ::RuntimeCall>(),),("Balances",size_of:: < <Balances as Callable<Runtime>> ::RuntimeCall>(),),("Council",size_of:: < <Council as Callable<Runtime>> ::RuntimeCall>(),),("TechnicalCommittee",size_of:: < <TechnicalCommittee as Callable<Runtime>> ::RuntimeCall>(),),("Treasury",size_of:: < <Treasury as Callable<Runtime>> ::RuntimeCall>(),),("Authorship",size_of:: < <Authorship as Callable<Runtime>> ::RuntimeCall>(),),("CollatorSelection",size_of:: < <CollatorSelection as Callable<Runtime>> ::RuntimeCall>(),),("Session",size_of:: < <Session as Callable<Runtime>> ::RuntimeCall>(),),("Multisig",size_of:: < <Multisig as Callable<Runtime>> ::RuntimeCall>(),),("Vesting",size_of:: < <Vesting as Callable<Runtime>> ::RuntimeCall>(),),("Msa",size_of:: < <Msa as Callable<Runtime>> ::RuntimeCall>(),),("Messages",size_of:: < <Messages as Callable<Runtime>> ::RuntimeCall>(),),("Schemas",size_of:: < <Schemas as Callable<Runtime>> ::RuntimeCall>(),),("Capacity",size_of:: < <Capacity as Callable<Runtime>> ::RuntimeCall>(),),("FrequencyTxPayment",size_of:: < <FrequencyTxPayment as Callable<Runtime>> ::RuntimeCall>(),),]
  }
  #[doc = r" Panics with diagnostic information if the size is greater than the given `limit`."]
  pub fn assert_size_under(limit:usize){
    let size = core::mem::size_of:: <Self>();
    let call_oversize = size>limit;
    if call_oversize {
      {
        $crate::io::_print($crate::fmt::Arguments::new_v1(&[], &[$crate::fmt::ArgumentV1::new(&(size),$crate::fmt::Display::fmt),$crate::fmt::ArgumentV1::new(&(limit),$crate::fmt::Display::fmt),]));
      };
      let mut sizes = Self::sizes().to_vec();
      sizes.sort_by_key(|x| -(x.1 as isize));
      for(i, &(name,size))in sizes.iter().enumerate().take(5){
        {
          $crate::io::_print($crate::fmt::Arguments::new_v1(&[], &[$crate::fmt::ArgumentV1::new(&(i+1),$crate::fmt::Display::fmt),$crate::fmt::ArgumentV1::new(&(name),$crate::fmt::Display::fmt),$crate::fmt::ArgumentV1::new(&(size),$crate::fmt::Display::fmt),]));
        };
      }if let Some((_,next_size)) = sizes.get(5){
        {
          $crate::io::_print($crate::fmt::Arguments::new_v1(&[], &[$crate::fmt::ArgumentV1::new(&(sizes.len()-5),$crate::fmt::Display::fmt),$crate::fmt::ArgumentV1::new(&(next_size),$crate::fmt::Display::fmt),]));
        };
      }$crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
    }
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::GetDispatchInfo for RuntimeCall {
  fn get_dispatch_info(&self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::DispatchInfo {
    match self {
      RuntimeCall::System(call) => call.get_dispatch_info(),
      RuntimeCall::ParachainSystem(call) => call.get_dispatch_info(),
      RuntimeCall::Timestamp(call) => call.get_dispatch_info(), 
      #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
      RuntimeCall::Sudo(call) => call.get_dispatch_info(),
      RuntimeCall::Preimage(call) => call.get_dispatch_info(),
      RuntimeCall::Democracy(call) => call.get_dispatch_info(),
      RuntimeCall::Scheduler(call) => call.get_dispatch_info(),
      RuntimeCall::Utility(call) => call.get_dispatch_info(),
      RuntimeCall::Balances(call) => call.get_dispatch_info(),
      RuntimeCall::Council(call) => call.get_dispatch_info(),
      RuntimeCall::TechnicalCommittee(call) => call.get_dispatch_info(),
      RuntimeCall::Treasury(call) => call.get_dispatch_info(),
      RuntimeCall::Authorship(call) => call.get_dispatch_info(),
      RuntimeCall::CollatorSelection(call) => call.get_dispatch_info(),
      RuntimeCall::Session(call) => call.get_dispatch_info(),
      RuntimeCall::Multisig(call) => call.get_dispatch_info(),
      RuntimeCall::Vesting(call) => call.get_dispatch_info(),
      RuntimeCall::Msa(call) => call.get_dispatch_info(),
      RuntimeCall::Messages(call) => call.get_dispatch_info(),
      RuntimeCall::Schemas(call) => call.get_dispatch_info(),
      RuntimeCall::Capacity(call) => call.get_dispatch_info(),
      RuntimeCall::FrequencyTxPayment(call) => call.get_dispatch_info(),
    
      }
  }

  }
#[allow(deprecated)]
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::weights::GetDispatchInfo for RuntimeCall{}

impl self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::GetCallMetadata for RuntimeCall {
  fn get_call_metadata(&self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::GetCallName;
    match self {
      RuntimeCall::System(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "System";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::ParachainSystem(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "ParachainSystem";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Timestamp(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Timestamp";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
      RuntimeCall::Sudo(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Sudo";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Preimage(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Preimage";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Democracy(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Democracy";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Scheduler(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Scheduler";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Utility(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Utility";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Balances(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Balances";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Council(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Council";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::TechnicalCommittee(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "TechnicalCommittee";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Treasury(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Treasury";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Authorship(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Authorship";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::CollatorSelection(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "CollatorSelection";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Session(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Session";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Multisig(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Multisig";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Vesting(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Vesting";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Msa(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Msa";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Messages(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Messages";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Schemas(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Schemas";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::Capacity(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "Capacity";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
      RuntimeCall::FrequencyTxPayment(call) => {
        let function_name = call.get_call_name();
        let pallet_name = "FrequencyTxPayment";
        self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallMetadata {
          function_name,pallet_name
        }
      }
    
      }
  }
  fn get_module_names() ->  &'static[&'static str]{
    &["System","ParachainSystem","Timestamp", #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    "Sudo","Preimage","Democracy","Scheduler","Utility","Balances","Council","TechnicalCommittee","Treasury","Authorship","CollatorSelection","Session","Multisig","Vesting","Msa","Messages","Schemas","Capacity","FrequencyTxPayment",]
  }
  fn get_call_names(module: &str) ->  &'static[&'static str]{
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::{
      Callable,GetCallName
    };
    match module {
      "System" =>  <<System as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "ParachainSystem" =>  <<ParachainSystem as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Timestamp" =>  <<Timestamp as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(), 
      #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
      "Sudo" =>  <<Sudo as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Preimage" =>  <<Preimage as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Democracy" =>  <<Democracy as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Scheduler" =>  <<Scheduler as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Utility" =>  <<Utility as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Balances" =>  <<Balances as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Council" =>  <<Council as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "TechnicalCommittee" =>  <<TechnicalCommittee as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Treasury" =>  <<Treasury as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Authorship" =>  <<Authorship as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "CollatorSelection" =>  <<CollatorSelection as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Session" =>  <<Session as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Multisig" =>  <<Multisig as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Vesting" =>  <<Vesting as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Msa" =>  <<Msa as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Messages" =>  <<Messages as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Schemas" =>  <<Schemas as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "Capacity" =>  <<Capacity as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      "FrequencyTxPayment" =>  <<FrequencyTxPayment as Callable<Runtime>> ::RuntimeCall as GetCallName> ::get_call_names(),
      _ => $crate::panicking::panic("internal error: entered unreachable code"),
    
      }
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::Dispatchable for RuntimeCall {
  type RuntimeOrigin = RuntimeOrigin;
  type Config = RuntimeCall;
  type Info = self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::DispatchInfo;
  type PostInfo = self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::PostDispatchInfo;
  fn dispatch(self,origin:RuntimeOrigin) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::DispatchResultWithPostInfo {
    if! <Self::RuntimeOrigin as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OriginTrait> ::filter_call(&origin, &self){
      return self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_std::result::Result::Err(frame_system::Error:: <Runtime> ::CallFiltered.into());
    }self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(self,origin)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable for RuntimeCall {
  type RuntimeOrigin = RuntimeOrigin;
  fn dispatch_bypass_filter(self,origin:RuntimeOrigin) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::DispatchResultWithPostInfo {
    match self {
      RuntimeCall::System(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::ParachainSystem(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Timestamp(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin), 
      #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
      RuntimeCall::Sudo(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Preimage(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Democracy(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Scheduler(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Utility(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Balances(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Council(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::TechnicalCommittee(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Treasury(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Authorship(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::CollatorSelection(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Session(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Multisig(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Vesting(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Msa(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Messages(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Schemas(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::Capacity(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
      RuntimeCall::FrequencyTxPayment(call) => self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::UnfilteredDispatchable::dispatch_bypass_filter(call,origin),
    
      }
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<System,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<System,Runtime>>{
    match self {
      RuntimeCall::System(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<System,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<System,Runtime>) -> Self {
    RuntimeCall::System(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<ParachainSystem,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<ParachainSystem,Runtime>>{
    match self {
      RuntimeCall::ParachainSystem(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<ParachainSystem,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<ParachainSystem,Runtime>) -> Self {
    RuntimeCall::ParachainSystem(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Timestamp,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Timestamp,Runtime>>{
    match self {
      RuntimeCall::Timestamp(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Timestamp,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Timestamp,Runtime>) -> Self {
    RuntimeCall::Timestamp(call)
  }

  }
#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Sudo,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Sudo,Runtime>>{
    match self {
      RuntimeCall::Sudo(call) => Some(call),
      _ => None,
    
      }
  }

  }
#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Sudo,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Sudo,Runtime>) -> Self {
    RuntimeCall::Sudo(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Preimage,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Preimage,Runtime>>{
    match self {
      RuntimeCall::Preimage(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Preimage,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Preimage,Runtime>) -> Self {
    RuntimeCall::Preimage(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Democracy,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Democracy,Runtime>>{
    match self {
      RuntimeCall::Democracy(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Democracy,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Democracy,Runtime>) -> Self {
    RuntimeCall::Democracy(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Scheduler,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Scheduler,Runtime>>{
    match self {
      RuntimeCall::Scheduler(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Scheduler,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Scheduler,Runtime>) -> Self {
    RuntimeCall::Scheduler(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Utility,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Utility,Runtime>>{
    match self {
      RuntimeCall::Utility(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Utility,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Utility,Runtime>) -> Self {
    RuntimeCall::Utility(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Balances,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Balances,Runtime>>{
    match self {
      RuntimeCall::Balances(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Balances,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Balances,Runtime>) -> Self {
    RuntimeCall::Balances(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Council,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Council,Runtime>>{
    match self {
      RuntimeCall::Council(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Council,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Council,Runtime>) -> Self {
    RuntimeCall::Council(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<TechnicalCommittee,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<TechnicalCommittee,Runtime>>{
    match self {
      RuntimeCall::TechnicalCommittee(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<TechnicalCommittee,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<TechnicalCommittee,Runtime>) -> Self {
    RuntimeCall::TechnicalCommittee(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Treasury,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Treasury,Runtime>>{
    match self {
      RuntimeCall::Treasury(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Treasury,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Treasury,Runtime>) -> Self {
    RuntimeCall::Treasury(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Authorship,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Authorship,Runtime>>{
    match self {
      RuntimeCall::Authorship(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Authorship,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Authorship,Runtime>) -> Self {
    RuntimeCall::Authorship(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<CollatorSelection,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<CollatorSelection,Runtime>>{
    match self {
      RuntimeCall::CollatorSelection(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<CollatorSelection,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<CollatorSelection,Runtime>) -> Self {
    RuntimeCall::CollatorSelection(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Session,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Session,Runtime>>{
    match self {
      RuntimeCall::Session(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Session,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Session,Runtime>) -> Self {
    RuntimeCall::Session(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Multisig,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Multisig,Runtime>>{
    match self {
      RuntimeCall::Multisig(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Multisig,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Multisig,Runtime>) -> Self {
    RuntimeCall::Multisig(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Vesting,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Vesting,Runtime>>{
    match self {
      RuntimeCall::Vesting(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Vesting,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Vesting,Runtime>) -> Self {
    RuntimeCall::Vesting(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Msa,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Msa,Runtime>>{
    match self {
      RuntimeCall::Msa(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Msa,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Msa,Runtime>) -> Self {
    RuntimeCall::Msa(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Messages,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Messages,Runtime>>{
    match self {
      RuntimeCall::Messages(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Messages,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Messages,Runtime>) -> Self {
    RuntimeCall::Messages(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Schemas,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Schemas,Runtime>>{
    match self {
      RuntimeCall::Schemas(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Schemas,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Schemas,Runtime>) -> Self {
    RuntimeCall::Schemas(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Capacity,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Capacity,Runtime>>{
    match self {
      RuntimeCall::Capacity(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Capacity,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<Capacity,Runtime>) -> Self {
    RuntimeCall::Capacity(call)
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IsSubType<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<FrequencyTxPayment,Runtime>>for RuntimeCall {
  #[allow(unreachable_patterns)]
  fn is_sub_type(&self) -> Option< &self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<FrequencyTxPayment,Runtime>>{
    match self {
      RuntimeCall::FrequencyTxPayment(call) => Some(call),
      _ => None,
    
      }
  }

  }
impl From<self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<FrequencyTxPayment,Runtime>>for RuntimeCall {
  fn from(call:self::sp_api_hidden_includes_construct_runtime::hidden_include::dispatch::CallableCallFor<FrequencyTxPayment,Runtime>) -> Self {
    RuntimeCall::FrequencyTxPayment(call)
  }

  }
impl Runtime {
  pub fn metadata() -> self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::RuntimeMetadataPrefixed {
    self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::RuntimeMetadataLastVersion::new((<[_]>::into_vec(#[rustc_box]
    $crate::boxed::Box::new([(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"System",index:0u8,storage:Some(frame_system::Pallet:: <Runtime> ::storage_metadata()),calls:Some(frame_system::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <frame_system::Event:: <Runtime> >()
      }),constants:frame_system::Pallet:: <Runtime> ::pallet_constants_metadata(),error:frame_system::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"ParachainSystem",index:1u8,storage:Some(cumulus_pallet_parachain_system::Pallet:: <Runtime> ::storage_metadata()),calls:Some(cumulus_pallet_parachain_system::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <cumulus_pallet_parachain_system::Event:: <Runtime> >()
      }),constants:cumulus_pallet_parachain_system::Pallet:: <Runtime> ::pallet_constants_metadata(),error:cumulus_pallet_parachain_system::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Timestamp",index:2u8,storage:Some(pallet_timestamp::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_timestamp::Pallet:: <Runtime> ::call_functions()),event:None,constants:pallet_timestamp::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_timestamp::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"ParachainInfo",index:3u8,storage:Some(parachain_info::Pallet:: <Runtime> ::storage_metadata()),calls:None,event:None,constants:parachain_info::Pallet:: <Runtime> ::pallet_constants_metadata(),error:parachain_info::Pallet:: <Runtime> ::error_metadata(),
    }),(#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Sudo",index:4u8,storage:Some(pallet_sudo::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_sudo::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_sudo::Event:: <Runtime> >()
      }),constants:pallet_sudo::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_sudo::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Preimage",index:5u8,storage:Some(pallet_preimage::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_preimage::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_preimage::Event:: <Runtime> >()
      }),constants:pallet_preimage::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_preimage::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Democracy",index:6u8,storage:Some(pallet_democracy::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_democracy::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_democracy::Event:: <Runtime> >()
      }),constants:pallet_democracy::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_democracy::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Scheduler",index:8u8,storage:Some(pallet_scheduler::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_scheduler::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_scheduler::Event:: <Runtime> >()
      }),constants:pallet_scheduler::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_scheduler::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Utility",index:9u8,storage:None,calls:Some(pallet_utility::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_utility::Event>()
      }),constants:pallet_utility::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_utility::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Balances",index:10u8,storage:Some(pallet_balances::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_balances::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_balances::Event:: <Runtime> >()
      }),constants:pallet_balances::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_balances::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"TransactionPayment",index:11u8,storage:Some(pallet_transaction_payment::Pallet:: <Runtime> ::storage_metadata()),calls:None,event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_transaction_payment::Event:: <Runtime> >()
      }),constants:pallet_transaction_payment::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_transaction_payment::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Council",index:12u8,storage:Some(pallet_collective::Pallet:: <Runtime,pallet_collective::Instance1> ::storage_metadata()),calls:Some(pallet_collective::Pallet:: <Runtime,pallet_collective::Instance1> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_collective::Event:: <Runtime,pallet_collective::Instance1> >()
      }),constants:pallet_collective::Pallet:: <Runtime,pallet_collective::Instance1> ::pallet_constants_metadata(),error:pallet_collective::Pallet:: <Runtime,pallet_collective::Instance1> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"TechnicalCommittee",index:13u8,storage:Some(pallet_collective::Pallet:: <Runtime,pallet_collective::Instance2> ::storage_metadata()),calls:Some(pallet_collective::Pallet:: <Runtime,pallet_collective::Instance2> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_collective::Event:: <Runtime,pallet_collective::Instance2> >()
      }),constants:pallet_collective::Pallet:: <Runtime,pallet_collective::Instance2> ::pallet_constants_metadata(),error:pallet_collective::Pallet:: <Runtime,pallet_collective::Instance2> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Treasury",index:14u8,storage:Some(pallet_treasury::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_treasury::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_treasury::Event:: <Runtime> >()
      }),constants:pallet_treasury::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_treasury::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Authorship",index:20u8,storage:Some(pallet_authorship::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_authorship::Pallet:: <Runtime> ::call_functions()),event:None,constants:pallet_authorship::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_authorship::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"CollatorSelection",index:21u8,storage:Some(pallet_collator_selection::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_collator_selection::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_collator_selection::Event:: <Runtime> >()
      }),constants:pallet_collator_selection::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_collator_selection::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Session",index:22u8,storage:Some(pallet_session::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_session::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_session::Event>()
      }),constants:pallet_session::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_session::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Aura",index:23u8,storage:Some(pallet_aura::Pallet:: <Runtime> ::storage_metadata()),calls:None,event:None,constants:pallet_aura::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_aura::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"AuraExt",index:24u8,storage:Some(cumulus_pallet_aura_ext::Pallet:: <Runtime> ::storage_metadata()),calls:None,event:None,constants:cumulus_pallet_aura_ext::Pallet:: <Runtime> ::pallet_constants_metadata(),error:cumulus_pallet_aura_ext::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Multisig",index:30u8,storage:Some(pallet_multisig::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_multisig::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_multisig::Event:: <Runtime> >()
      }),constants:pallet_multisig::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_multisig::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Vesting",index:40u8,storage:Some(orml_vesting::Pallet:: <Runtime> ::storage_metadata()),calls:Some(orml_vesting::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <orml_vesting::Event:: <Runtime> >()
      }),constants:orml_vesting::Pallet:: <Runtime> ::pallet_constants_metadata(),error:orml_vesting::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Msa",index:60u8,storage:Some(pallet_msa::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_msa::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_msa::Event:: <Runtime> >()
      }),constants:pallet_msa::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_msa::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Messages",index:61u8,storage:Some(pallet_messages::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_messages::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_messages::Event:: <Runtime> >()
      }),constants:pallet_messages::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_messages::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Schemas",index:62u8,storage:Some(pallet_schemas::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_schemas::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_schemas::Event:: <Runtime> >()
      }),constants:pallet_schemas::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_schemas::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"Capacity",index:63u8,storage:Some(pallet_capacity::Pallet:: <Runtime> ::storage_metadata()),calls:Some(pallet_capacity::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_capacity::Event:: <Runtime> >()
      }),constants:pallet_capacity::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_capacity::Pallet:: <Runtime> ::error_metadata(),
    }),(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletMetadata {
      name:"FrequencyTxPayment",index:64u8,storage:None,calls:Some(pallet_frequency_tx_payment::Pallet:: <Runtime> ::call_functions()),event:Some(self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::PalletEventMetadata {
        ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <pallet_frequency_tx_payment::Event:: <Runtime> >()
      }),constants:pallet_frequency_tx_payment::Pallet:: <Runtime> ::pallet_constants_metadata(),error:pallet_frequency_tx_payment::Pallet:: <Runtime> ::error_metadata(),
    })]))),self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::ExtrinsicMetadata {
      ty:self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <UncheckedExtrinsic>(),version: <UncheckedExtrinsic as self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::ExtrinsicMetadata> ::VERSION,signed_extensions: < <UncheckedExtrinsic as self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::ExtrinsicMetadata> ::SignedExtensions as self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::SignedExtension> ::metadata().into_iter().map(|meta|self::sp_api_hidden_includes_construct_runtime::hidden_include::metadata::SignedExtensionMetadata {
        identifier:meta.identifier,ty:meta.ty,additional_signed:meta.additional_signed,
      }).collect(),
    },self::sp_api_hidden_includes_construct_runtime::hidden_include::scale_info::meta_type:: <Runtime>()).into()
  }

  }
#[cfg(any(feature = "std",test))]
pub type SystemConfig = frame_system::GenesisConfig;
#[cfg(any(feature = "std",test))]
pub type ParachainSystemConfig = cumulus_pallet_parachain_system::GenesisConfig;
#[cfg(any(feature = "std",test))]
pub type ParachainInfoConfig = parachain_info::GenesisConfig;
#[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
#[cfg(any(feature = "std",test))]
pub type SudoConfig = pallet_sudo::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type DemocracyConfig = pallet_democracy::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type BalancesConfig = pallet_balances::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type CouncilConfig = pallet_collective::GenesisConfig<Runtime,pallet_collective::Instance1> ;
#[cfg(any(feature = "std",test))]
pub type TechnicalCommitteeConfig = pallet_collective::GenesisConfig<Runtime,pallet_collective::Instance2> ;
#[cfg(any(feature = "std",test))]
pub type TreasuryConfig = pallet_treasury::GenesisConfig;
#[cfg(any(feature = "std",test))]
pub type CollatorSelectionConfig = pallet_collator_selection::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type SessionConfig = pallet_session::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type AuraConfig = pallet_aura::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type AuraExtConfig = cumulus_pallet_aura_ext::GenesisConfig;
#[cfg(any(feature = "std",test))]
pub type VestingConfig = orml_vesting::GenesisConfig<Runtime> ;
#[cfg(any(feature = "std",test))]
pub type SchemasConfig = pallet_schemas::GenesisConfig;
#[cfg(any(feature = "std",test))]
use self::sp_api_hidden_includes_construct_runtime::hidden_include::serde as __genesis_config_serde_import__;
#[cfg(any(feature = "std",test))]
#[derive(self::sp_api_hidden_includes_construct_runtime::hidden_include::serde::Serialize,self::sp_api_hidden_includes_construct_runtime::hidden_include::serde::Deserialize,Default)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[serde(crate = "__genesis_config_serde_import__")]
pub struct GenesisConfig {
  pub system:SystemConfig,pub parachain_system:ParachainSystemConfig,pub parachain_info:ParachainInfoConfig, #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
  pub sudo:SudoConfig,pub democracy:DemocracyConfig,pub balances:BalancesConfig,pub council:CouncilConfig,pub technical_committee:TechnicalCommitteeConfig,pub treasury:TreasuryConfig,pub collator_selection:CollatorSelectionConfig,pub session:SessionConfig,pub aura:AuraConfig,pub aura_ext:AuraExtConfig,pub vesting:VestingConfig,pub schemas:SchemasConfig,
}
#[cfg(any(feature = "std",test))]
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildStorage for GenesisConfig {
  fn assimilate_storage(&self,storage: &mut self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::Storage,) -> std::result::Result<(),String>{
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,frame_system::__InherentHiddenInstance> ::build_module_genesis_storage(&self.system,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,cumulus_pallet_parachain_system::__InherentHiddenInstance> ::build_module_genesis_storage(&self.parachain_system,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,parachain_info::__InherentHiddenInstance> ::build_module_genesis_storage(&self.parachain_info,storage)? ;
    #[cfg(any(not(feature = "frequency"),feature = "all-frequency-features"))]
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_sudo::__InherentHiddenInstance> ::build_module_genesis_storage(&self.sudo,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_democracy::__InherentHiddenInstance> ::build_module_genesis_storage(&self.democracy,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_balances::__InherentHiddenInstance> ::build_module_genesis_storage(&self.balances,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_collective::Instance1> ::build_module_genesis_storage(&self.council,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_collective::Instance2> ::build_module_genesis_storage(&self.technical_committee,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_treasury::__InherentHiddenInstance> ::build_module_genesis_storage(&self.treasury,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_collator_selection::__InherentHiddenInstance> ::build_module_genesis_storage(&self.collator_selection,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_session::__InherentHiddenInstance> ::build_module_genesis_storage(&self.session,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_aura::__InherentHiddenInstance> ::build_module_genesis_storage(&self.aura,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,cumulus_pallet_aura_ext::__InherentHiddenInstance> ::build_module_genesis_storage(&self.aura_ext,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,orml_vesting::__InherentHiddenInstance> ::build_module_genesis_storage(&self.vesting,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::BuildModuleGenesisStorage:: <Runtime,pallet_schemas::__InherentHiddenInstance> ::build_module_genesis_storage(&self.schemas,storage)? ;
    self::sp_api_hidden_includes_construct_runtime::hidden_include::BasicExternalities::execute_with_storage(storage, ||{
      <AllPalletsWithSystem as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::OnGenesis> ::on_genesis();
    });
    Ok(())
  }

  }
trait InherentDataExt {
  fn create_extrinsics(&self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Vec<<Block as self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::BlockT> ::Extrinsic> ;
  
  fn check_extrinsics(&self,block: &Block) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::CheckInherentsResult;

  }impl InherentDataExt for self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::InherentData {
  fn create_extrinsics(&self) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Vec<<Block as self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::BlockT> ::Extrinsic>{
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::ProvideInherent;
    let mut inherents = Vec::new();
    if let Some(inherent) = ParachainSystem::create_inherent(self){
      let inherent =  <UncheckedExtrinsic as self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Extrinsic> ::new(inherent.into(),None,).expect("Runtime UncheckedExtrinsic is not Opaque, so it has to return \
							`Some`; qed");
      inherents.push(inherent);
    }if let Some(inherent) = Timestamp::create_inherent(self){
      let inherent =  <UncheckedExtrinsic as self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Extrinsic> ::new(inherent.into(),None,).expect("Runtime UncheckedExtrinsic is not Opaque, so it has to return \
							`Some`; qed");
      inherents.push(inherent);
    }inherents
  }
  fn check_extrinsics(&self,block: &Block) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::CheckInherentsResult {
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::{
      ProvideInherent,IsFatalError
    };
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::{
      IsSubType,ExtrinsicCall
    };
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::Block as _;
    let mut result = self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::CheckInherentsResult::new();
    for xt in block.extrinsics(){
      if self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Extrinsic::is_signed(xt).unwrap_or(false){
        break
      }let mut is_inherent = false;
      {
        let call =  <UncheckedExtrinsic as ExtrinsicCall> ::call(xt);
        if let Some(call) = IsSubType:: <_> ::is_sub_type(call){
          if ParachainSystem::is_inherent(call){
            is_inherent = true;
            if let Err(e) = ParachainSystem::check_inherent(call,self){
              result.put_error(ParachainSystem::INHERENT_IDENTIFIER, &e).expect("There is only one fatal error; qed");
              if e.is_fatal_error(){
                return result;
              }
            }
          }
        }
      }{
        let call =  <UncheckedExtrinsic as ExtrinsicCall> ::call(xt);
        if let Some(call) = IsSubType:: <_> ::is_sub_type(call){
          if Timestamp::is_inherent(call){
            is_inherent = true;
            if let Err(e) = Timestamp::check_inherent(call,self){
              result.put_error(Timestamp::INHERENT_IDENTIFIER, &e).expect("There is only one fatal error; qed");
              if e.is_fatal_error(){
                return result;
              }
            }
          }
        }
      }if!is_inherent {
        break
      }
    }match ParachainSystem::is_inherent_required(self){
      Ok(Some(e)) => {
        let found = block.extrinsics().iter().any(|xt|{
          let is_signed = self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Extrinsic::is_signed(xt).unwrap_or(false);
          if!is_signed {
            let call =  <UncheckedExtrinsic as ExtrinsicCall> ::call(xt);
            if let Some(call) = IsSubType:: <_> ::is_sub_type(call){
              ParachainSystem::is_inherent(&call)
            }else {
              false
            }
          }else {
            false
          }
        });
        if!found {
          result.put_error(ParachainSystem::INHERENT_IDENTIFIER, &e).expect("There is only one fatal error; qed");
          if e.is_fatal_error(){
            return result;
          }
        }
      },
      Ok(None) => (),
      Err(e) => {
        result.put_error(ParachainSystem::INHERENT_IDENTIFIER, &e).expect("There is only one fatal error; qed");
        if e.is_fatal_error(){
          return result;
        }
      },
    
      }match Timestamp::is_inherent_required(self){
      Ok(Some(e)) => {
        let found = block.extrinsics().iter().any(|xt|{
          let is_signed = self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Extrinsic::is_signed(xt).unwrap_or(false);
          if!is_signed {
            let call =  <UncheckedExtrinsic as ExtrinsicCall> ::call(xt);
            if let Some(call) = IsSubType:: <_> ::is_sub_type(call){
              Timestamp::is_inherent(&call)
            }else {
              false
            }
          }else {
            false
          }
        });
        if!found {
          result.put_error(Timestamp::INHERENT_IDENTIFIER, &e).expect("There is only one fatal error; qed");
          if e.is_fatal_error(){
            return result;
          }
        }
      },
      Ok(None) => (),
      Err(e) => {
        result.put_error(Timestamp::INHERENT_IDENTIFIER, &e).expect("There is only one fatal error; qed");
        if e.is_fatal_error(){
          return result;
        }
      },
    
      }result
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::EnsureInherentsAreFirst<Block>for Runtime {
  fn ensure_inherents_are_first(block: &Block) -> Result<(),u32>{
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::ProvideInherent;
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::{
      IsSubType,ExtrinsicCall
    };
    use self::sp_api_hidden_includes_construct_runtime::hidden_include::sp_runtime::traits::Block as _;
    let mut first_signed_observed = false;
    for(i,xt)in block.extrinsics().iter().enumerate(){
      let is_signed = self::sp_api_hidden_includes_construct_runtime::hidden_include::inherent::Extrinsic::is_signed(xt).unwrap_or(false);
      let is_inherent = if is_signed {
        false
      }else {
        let mut is_inherent = false;
        {
          let call =  <UncheckedExtrinsic as ExtrinsicCall> ::call(xt);
          if let Some(call) = IsSubType:: <_> ::is_sub_type(call){
            if ParachainSystem::is_inherent(&call){
              is_inherent = true;
            }
          }
        }{
          let call =  <UncheckedExtrinsic as ExtrinsicCall> ::call(xt);
          if let Some(call) = IsSubType:: <_> ::is_sub_type(call){
            if Timestamp::is_inherent(&call){
              is_inherent = true;
            }
          }
        }is_inherent
      };
      if!is_inherent {
        first_signed_observed = true;
      }if first_signed_observed&&is_inherent {
        return Err(i as u32)
      }
    }Ok(())
  }

  }
impl self::sp_api_hidden_includes_construct_runtime::hidden_include::unsigned::ValidateUnsigned for Runtime {
  type Call = RuntimeCall;
  fn pre_dispatch(call: &Self::Call) -> Result<(),self::sp_api_hidden_includes_construct_runtime::hidden_include::unsigned::TransactionValidityError>{
    #[allow(unreachable_patterns)]
    match call {
      RuntimeCall::ParachainSystem(inner_call) => ParachainSystem::pre_dispatch(inner_call),
      _ => Ok(()),
    
      }
  }
  fn validate_unsigned(#[allow(unused_variables)]
  source:self::sp_api_hidden_includes_construct_runtime::hidden_include::unsigned::TransactionSource,call: &Self::Call,) -> self::sp_api_hidden_includes_construct_runtime::hidden_include::unsigned::TransactionValidity {
    #[allow(unreachable_patterns)]
    match call {
      RuntimeCall::ParachainSystem(inner_call) => ParachainSystem::validate_unsigned(source,inner_call),
      _ => self::sp_api_hidden_includes_construct_runtime::hidden_include::unsigned::UnknownTransaction::NoUnsignedValidator.into(),
    
      }
  }

  }
#[cfg(test)]
mod __construct_runtime_integrity_test {
  use super:: * ;
  #[test]
  pub fn runtime_integrity_tests(){
    <AllPalletsWithSystem as self::sp_api_hidden_includes_construct_runtime::hidden_include::traits::IntegrityTest> ::integrity_test();
  }

  }const _:() = {
  if!<frame_system::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<cumulus_pallet_parachain_system::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_sudo::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_preimage::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_democracy::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_scheduler::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_utility::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_balances::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_collective::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_collective::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_treasury::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_authorship::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_collator_selection::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_session::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_multisig::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<orml_vesting::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_msa::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_messages::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_schemas::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
const _:() = {
  if!<pallet_capacity::Error<Runtime>as $crate::traits::PalletError>::MAX_ENCODED_SIZE<=$crate::MAX_MODULE_ERROR_ENCODED_SIZE {
    $crate::panicking::panic_fmt($crate::fmt::Arguments::new_v1(&[], &[]));
  }
};
