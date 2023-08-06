use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{models::collection::Collection, AppState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryableCollection;

#[get("/collections/{collection_id}")]
pub async fn collection_detail(
    data: web::Data<AppState>,
    collection_id: web::Path<Uuid>,
    request_query: web::Query<RequestType<Collection, QueryableCollection, QueryableCollection>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let collection_id = collection_id.into_inner();

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request_query.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let collection = Collection::get_by_id(
        pool,
        collection_id,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match collection {
        Ok(collection) => Ok(HttpResponse::Ok().json(ResponseType::new(
            collection,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/collections/{collection_id}".to_string(),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
