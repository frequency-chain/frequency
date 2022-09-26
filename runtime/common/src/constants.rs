use frame_support::parameter_types;

pub const FREQUENCY_ROCOCO_TOKEN: &str = "XRQCY";
pub const FREQUENCY_TOKEN: &str = "FRQCY";

parameter_types! {
	/// Clone + Debug + Eq  implementation for u32 types
	pub const MaxDataSize: u32 = 30;
}

impl Clone for MaxDataSize {
	fn clone(&self) -> Self {
		MaxDataSize {}
	}
}

impl Eq for MaxDataSize {
	fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MaxDataSize {
	fn eq(&self, other: &Self) -> bool {
		self == other
	}
}

impl sp_std::fmt::Debug for MaxDataSize {
	#[cfg(feature = "std")]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}
