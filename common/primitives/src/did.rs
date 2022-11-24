#[cfg(feature = "std")]
use std::{
	default::Default,
	string::{String, ToString},
};

#[cfg(feature = "std")]
use serde::{
	Deserialize,
	ser::{Serializer, SerializeStruct},
	Serialize,
};
use serde_json::{self, Value};

const DIDPREFIX: &str = "did:dsnp:";

const CONTEXT: [&str; 4] = [
	"https://www.w3.org/ns/did/v1",
	"https://spec.dsnp.org/DSNP/Overview.html",
	"https://w3id.org/security/v2",
	"https://github.com/w3f/schnorrkel",
];

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum KeyType {
	Sr25519,
	Edd25519,
}

impl Default for KeyType {
	fn default() -> Self {
		Self::Sr25519
	}
}

#[cfg(feature = "std")]
#[derive(Copy, Clone, Debug, Default)]
pub struct Did {
	id: u32,
	// query: Option<HashMap<String, String>>
	// path: Vec<String>,
	// fragment: Option<String>,
}

impl Did {
	pub fn new(msa_id: u32) -> Self {
		Self { id: msa_id }
	}
}

impl ToString for Did {
	fn to_string(&self) -> String {
		DIDPREFIX.to_string() + &self.id.to_string()
	}
}

#[cfg(feature = "std")]
#[derive(Copy, Clone, Serialize, Debug, Default)]
pub struct VerificationMethod {
	#[serde(serialize_with = "Did::to_string")]
	pub id: Did,
	#[serde(serialize_with = "Did::to_string")]
	pub controller: Did,
	#[serde(rename = "type")]
	pub key_type: KeyType,
	#[serde(rename = "blockchainAccountId")]
	pub blockchain_account_id: u32,
	// pub blockchain_account_id_index: u32,
}

impl VerificationMethod {
	pub fn new(id: Did, controller: Did, blockchain_account_id: u32) -> Self {
		let key_type = KeyType::default();
		VerificationMethod { id, controller, key_type, blockchain_account_id }
	}
}

#[cfg(feature = "std")]
#[derive(Clone, Serialize, Debug, Default)]
pub struct DidDocument {
	#[serde(rename = "@context")]
	pub context: Vec<String>,
	#[serde(serialize_with = "Did::to_string")]
	pub id: Did,
	#[serde(serialize_with = "Did::to_string")]
	pub controller: Did,
	#[serde(rename = "verificationMethod", skip_serializing_if = "Vec::is_empty", default)]
	pub verification_method: Vec<VerificationMethod>,
	#[serde(rename = "capabilityDelegation", skip_serializing_if = "Vec::is_empty", default)]
	pub capability_delegation: Vec<VerificationMethod>,
}

impl DidDocument {
	pub fn new(id: Did, controller: Did) -> Self {
		let mut context = Vec::new();
		for c in CONTEXT {
			context.push(c.to_string());
		}
		DidDocument {
			context,
			id,
			controller,
			verification_method: Vec::new(),
			capability_delegation: Vec::new(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn can_construct_did() {
		let new_did: Did = Did::new(1234);
		assert_eq!("did:dsnp:1234", new_did.to_string().as_str());
	}

	#[test]
	fn can_construct_did_document() {
		let new_did_doc: DidDocument = DidDocument::new(Did::new(1234), Did::new(4321));
		assert_eq!("did:dsnp:1234", new_did_doc.id.to_string().as_str());
		assert_eq!("did:dsnp:4321", new_did_doc.controller.to_string().as_str());
		assert!(new_did_doc.verification_method.is_empty());
		assert!(new_did_doc.capability_delegation.is_empty());
		assert_eq!(
			Some(&"https://github.com/w3f/schnorrkel".to_string()),
			new_did_doc.context.get(3)
		);
	}

	#[test]
	fn can_construct_verification_method() {
		let new_verification_method = VerificationMethod::new(
			Did::new(3838),
			Did::new(3838),
			999999,
		);
		assert_eq!(KeyType::Sr25519, new_verification_method.key_type);
		assert_eq!("did:dsnp:3838", new_verification_method.id.to_string().as_str());
		assert_eq!("did:dsnp:3838", new_verification_method.controller.to_string().as_str());
		assert_eq!(999999, new_verification_method.blockchain_account_id);
	}

	#[test]
	fn did_document_serializes_correctly() {
		let mut new_did_doc: DidDocument = DidDocument::new(Did::new(1234), Did::new(1234));
		let account_keys: [u32; 1] = [31];
		let msa_id: u32 = 3343;
		for key in account_keys {
			new_did_doc.verification_method.push(VerificationMethod::new(
				Did::new(msa_id.clone()),
				Did::new(msa_id.clone()),
				key.clone())
			);
		}

		let providers: [(u32, u32); 2] = [(1, 32), (2, 42)];
		for (provider_msa_id, provider_key) in providers {
			new_did_doc.capability_delegation.push(
				VerificationMethod::new(
					Did::new(provider_msa_id.clone()),
					Did::new(provider_msa_id.clone()),
					provider_key.clone()
				)
			);
		}

		let expected_json_str = r#"{"@context":["https://www.w3.org/ns/did/v1","https://spec.dsnp.org/DSNP/Overview.html","https://w3id.org/security/v2","https://github.com/w3f/schnorrkel"],"id":"did:dsnp:1234","controller":"did:dsnp:1234","verificationMethod":[{"id":"did:dsnp:3343","controller":"did:dsnp:3343","type":"sr25519","blockchainAccountId":"1"},],"capabilityDelegation":[{"id": "did:dsnp:1","controller":"did:dsnp:1","type":"sr25519","blockchainAccountId":"32"},{"id":"did:dsnp:1","controller": "did:dsnp:1","type":"sr25519","blockchainAccountId":"32"}]}"#;
		let serialized = serde_json::to_string(&new_did_doc).unwrap();
		assert_eq!(expected_json_str, serialized);
	}
}
