use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType},
};
use uuid::Uuid;

use crate::{models::item::Item, AppState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryableItem;

#[get("/items/{item_id}")]
pub async fn item_detail(
    data: web::Data<AppState>,
    item_id: web::Path<Uuid>,
    request_query: web::Query<RequestType<Item, QueryableItem, QueryableItem>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let item_id = item_id.into_inner();

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request_query.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let item = Item::get_by_id(
        pool,
        item_id,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match item {
        Ok(item) => Ok(HttpResponse::Ok().json(item)),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/items/{item_id}".to_string(),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
