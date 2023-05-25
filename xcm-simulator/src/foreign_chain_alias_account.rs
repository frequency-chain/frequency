use xcm_simulator::MultiLocation;
use frame_support::traits::Get;
use sp_std::{borrow::Borrow, marker::PhantomData};
// use xcm::opaque::latest::Junction::{AccountId32, Parachain, AccountKey20};
use sp_io::hashing::blake2_256;
use xcm::latest::prelude::*;
use xcm_executor::traits::Convert;
use codec::Encode;

/// Prefix for generating alias account for accounts coming
/// from chains that use 32 byte long representations.
pub const FOREIGN_CHAIN_PREFIX_PARA_32: [u8; 37] = *b"ForeignChainAliasAccountPrefix_Para32";

/// Prefix for generating alias account for accounts coming
/// from chains that use 20 byte long representations.
pub const FOREIGN_CHAIN_PREFIX_PARA_20: [u8; 37] = *b"ForeignChainAliasAccountPrefix_Para20";

/// Prefix for generating alias account for accounts coming
/// from the relay chain using 32 byte long representations.
pub const FOREIGN_CHAIN_PREFIX_RELAY: [u8; 36] = *b"ForeignChainAliasAccountPrefix_Relay";

pub struct ForeignChainAliasAccount<AccountId>(PhantomData<AccountId>);
impl<AccountId: From<[u8; 32]> + Clone> Convert<MultiLocation, AccountId>
	for ForeignChainAliasAccount<AccountId>
{
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		let entropy = match location.borrow() {
			// Used on the relay chain for sending paras that use 32 byte accounts
			MultiLocation {
				parents: 0,
				interior: X2(Parachain(para_id), AccountId32 { id, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_32(para_id, id, 0),

			// Used on the relay chain for sending paras that use 20 byte accounts
			MultiLocation {
				parents: 0,
				interior: X2(Parachain(para_id), AccountKey20 { key, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_20(para_id, key, 0),

			// Used on para-chain for sending paras that use 32 byte accounts
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), AccountId32 { id, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_32(para_id, id, 1),

			// Used on para-chain for sending paras that use 20 byte accounts
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(para_id), AccountKey20 { key, .. }),
			} => ForeignChainAliasAccount::<AccountId>::from_para_20(para_id, key, 1),

			// Used on para-chain for sending from the relay chain
			MultiLocation { parents: 1, interior: X1(AccountId32 { id, .. }) } =>
				ForeignChainAliasAccount::<AccountId>::from_relay_32(id, 1),

			// No other conversions provided
			_ => return Err(()),
		};

		Ok(entropy.into())
	}

	fn reverse_ref(_: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		Err(())
	}
}

impl<AccountId> ForeignChainAliasAccount<AccountId> {
	pub fn from_para_32(para_id: &u32, id: &[u8; 32], parents: u8) -> [u8; 32] {
		(FOREIGN_CHAIN_PREFIX_PARA_32, para_id, id, parents).using_encoded(blake2_256)
	}

	fn from_para_20(para_id: &u32, id: &[u8; 20], parents: u8) -> [u8; 32] {
		(FOREIGN_CHAIN_PREFIX_PARA_20, para_id, id, parents).using_encoded(blake2_256)
	}

	fn from_relay_32(id: &[u8; 32], parents: u8) -> [u8; 32] {
		(FOREIGN_CHAIN_PREFIX_RELAY, id, parents).using_encoded(blake2_256)
	}
}

pub struct Account32Hash<Network, AccountId>(PhantomData<(Network, AccountId)>);
impl<Network: Get<Option<NetworkId>>, AccountId: From<[u8; 32]> + Into<[u8; 32]> + Clone>
	Convert<MultiLocation, AccountId> for Account32Hash<Network, AccountId>
{
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		Ok(("multiloc", location.borrow()).using_encoded(blake2_256).into())
	}

	fn reverse_ref(_: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		Err(())
	}
}