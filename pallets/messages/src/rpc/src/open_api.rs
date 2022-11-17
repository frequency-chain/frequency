// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Messages](../pallet_messages/index.html)

#[cfg(feature = "std")]
use common_helpers::rpc::map_rpc_result;
use common_primitives::{messages::*, schema::*};
use frame_support::{ensure, fail};
use jsonrpsee::{
	core::{async_trait, error::Error as RpcError, RpcResult},
	proc_macros::rpc,
};
use pallet_messages_runtime_api::MessagesRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[cfg(test)]
mod tests;


#[derive(ApiRequest)]
enum MessagesApiRequest<T: ParseFromJSON + Send + Sync + Type + ToJSON + for<'b> Deserialize<'b>> {
    Json(Json<T>),
    Bcs(Bcs<T>),
}

impl<T: ParseFromJSON + Send + Sync + Type + ToJSON + for<'b> Deserialize<'b>> MyRequest<T> {
    fn unpack(self) -> T {
        let Self::Json(json) = self;
        json.0
    }
}

#[derive(ApiResponse)]
enum MessageApiResponse<T: ToJSON + Send + Sync + Serialize> {
    #[oai(status = 200, content_type = "application/json")]
    Json(Json<T>),
}
