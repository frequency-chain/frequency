use parity_scale_codec::Codec;
use scale_info::StaticTypeInfo;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{
	traits::{LookupError, StaticLookup},
	MultiAddress,
};
use sp_std::{fmt::Debug, marker::PhantomData};

/// A lookup implementation returning the `AccountId` from a `MultiAddress`.
pub struct EthCompatibleAccountIdLookup<AccountId, AccountIndex>(
	PhantomData<(AccountId, AccountIndex)>,
);
impl<AccountId, AccountIndex> StaticLookup for EthCompatibleAccountIdLookup<AccountId, AccountIndex>
where
	AccountId: Codec + Clone + PartialEq + Debug,
	AccountIndex: Codec + Clone + PartialEq + Debug,
	MultiAddress<AccountId, AccountIndex>: Codec + StaticTypeInfo,
{
	type Source = MultiAddress<AccountId, AccountIndex>;
	type Target = AccountId;
	fn lookup(x: Self::Source) -> Result<Self::Target, LookupError> {
		match x {
			MultiAddress::Id(i) => Ok(i),
			MultiAddress::Address20(acc20) => {
				log::info!(target: "ETHEREUM", "lookup 0x{:?}", HexDisplay::from(&acc20));
				let mut buffer = [0u8; 32];
				buffer[12..].copy_from_slice(&acc20);
				let decoded = Self::Target::decode(&mut &buffer[..]).map_err(|_| LookupError)?;
				Ok(decoded)
			},
			_ => Err(LookupError),
		}
	}
	fn unlookup(x: Self::Target) -> Self::Source {
		MultiAddress::Id(x)
		// This should probably leave commented out since we are always dealing with 32 byte accounts
		// let encoded = x.encode();
		// match encoded[..12].eq(&[0u8; 12]) {
		// 	true => {
		// 		log::info!(target: "ETHEREUM", "unlookup before 0x{:?}", HexDisplay::from(&encoded));
		// 		let mut address20 = [0u8; 20];
		// 		address20[..].copy_from_slice(&encoded[12..]);
		// 		log::info!(target: "ETHEREUM", "unlookup after 0x{:?}", HexDisplay::from(&address20));
		// 		MultiAddress::Address20(address20)
		// 	},
		// 	false => MultiAddress::Id(x),
		// }
	}
}
