/// MSA Offchain Storage Data
use codec::{Decode, Encode};
use common_primitives::msa::MessageSourceId;
use sp_std::collections::btree_map::BTreeMap;

/// MSA Public Key Data, stored in offchain storage
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct MSAPublicKeyData(pub BTreeMap<MessageSourceId, Vec<u8>>);
