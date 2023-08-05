use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType},
};
use uuid::Uuid;

use crate::{models::shop::Shop, AppState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryableShop;

#[get("/shops/{shop_id}")]
pub async fn shop_detail(
    data: web::Data<AppState>,
    shop_id: web::Path<Uuid>,
    request_query: web::Query<RequestType<Shop, QueryableShop, QueryableShop>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let shop_id = shop_id.into_inner();

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request_query.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let shop = Shop::get_by_id(
        pool,
        shop_id,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match shop {
        Ok(shop) => Ok(HttpResponse::Ok().json(shop)),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/shops/{shop_id}".to_string(),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
