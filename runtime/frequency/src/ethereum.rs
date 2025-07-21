use common_primitives::signatures::{AccountAddressMapper, EthereumAddressMapper};
use core::{fmt::Debug, marker::PhantomData};
use parity_scale_codec::Codec;
use scale_info::StaticTypeInfo;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{
	traits::{LookupError, StaticLookup},
	MultiAddress,
};

/// A lookup implementation returning the `AccountId` from a `MultiAddress`.
pub struct EthereumCompatibleAccountIdLookup<AccountId, AccountIndex>(
	PhantomData<(AccountId, AccountIndex)>,
);
impl<AccountId, AccountIndex> StaticLookup
	for EthereumCompatibleAccountIdLookup<AccountId, AccountIndex>
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
				log::debug!(target: "ETHEREUM", "lookup 0x{:?}", HexDisplay::from(&acc20));
				let account_id_bytes = EthereumAddressMapper::to_bytes32(&acc20);
				let decoded =
					Self::Target::decode(&mut &account_id_bytes[..]).map_err(|_| LookupError)?;
				Ok(decoded)
			},
			_ => Err(LookupError),
		}
	}
	fn unlookup(x: Self::Target) -> Self::Source {
		// We are not converting back to 20 bytes since everywhere we are using Id
		MultiAddress::Id(x)
	}
}

#[cfg(test)]
mod tests {
	use crate::ethereum::EthereumCompatibleAccountIdLookup;
	use sp_core::{bytes::from_hex, crypto::AccountId32};
	use sp_runtime::{traits::StaticLookup, MultiAddress};

	#[test]
	fn address20_should_get_decoded_correctly() {
		let lookup =
			EthereumCompatibleAccountIdLookup::<AccountId32, ()>::lookup(MultiAddress::Address20(
				from_hex("0x19a701d23f0ee1748b5d5f883cb833943096c6c4")
					.expect("should convert")
					.try_into()
					.expect("invalid size"),
			));
		assert!(lookup.is_ok());

		let converted = lookup.unwrap();
		let expected = AccountId32::new(
			from_hex("0x19a701d23f0ee1748b5d5f883cb833943096c6c4eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		assert_eq!(converted, expected)
	}
}
