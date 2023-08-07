use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{models::order::Order, AppState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlaceholderOrder;

#[get("/orders/{order_id}")]
pub async fn order_detail(
    data: web::Data<AppState>,
    order_id: web::Path<Uuid>,
    request_query: web::Query<RequestType<Order, PlaceholderOrder, PlaceholderOrder>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let order_id = order_id.into_inner();

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request_query.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let order = Order::get_by_id(
        pool,
        order_id,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match order {
        Ok(order) => Ok(HttpResponse::Ok().json(ResponseType::new(
            order,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/orders/{order_id}".to_string(),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
