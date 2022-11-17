// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Messages](../pallet_messages/index.html)

#[cfg(feature = "std")]
use common_primitives::{messages::*, schema::*};
use poem_openapi::{
	param::Query,
	payload::Json,
	types::{ParseFromJSON, ToJSON, Type},
	ApiRequest, ApiResponse, OpenApi,
};
use serde::{Deserialize, Serialize};

#[derive(ApiRequest)]
enum MessagesApiRequest<T: ParseFromJSON + Send + Sync + Type + ToJSON + for<'b> Deserialize<'b>> {
	Json(Json<T>),
}

#[derive(ApiResponse)]
enum MessageApiResponse<T: ToJSON + Send + Sync + Serialize> {
	#[oai(status = 200, content_type = "application/json")]
	Json(Json<T>),
}

/// Frequency Messages OpenAPI Handler
struct MessagesOAIHandler;

#[OpenApi]
impl MessagesOAIHandler {
	/// Retrieve paginated messages by schema id
	#[oai(path = "/messages/getBySchemaId", method = "get")]
	async fn get_messages_by_schema_id(
		&self,
		_schema_id: Query<SchemaId>,
		_pagination: BlockPaginationRequest,
	) -> MessageApiResponse<BlockPaginationResponse<MessageResponse>> {
		MessageApiResponse::Json(Json(BlockPaginationResponse {
			content: vec![],
			has_next: false,
			next_block: Some(0),
			next_index: Some(0),
		}))
	}
}
