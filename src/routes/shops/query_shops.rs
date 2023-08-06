use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{
    models::shop::{
        request::{QueryableShop, SortableShop},
        Shop,
    },
    AppState,
};

#[get("/shops")]
pub async fn query_shops(
    data: web::Data<AppState>,
    request: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    let request_query = serde_qs::from_str::<RequestType<Shop, QueryableShop, SortableShop>>(
        request.query_string(),
    );

    let request_query = match request_query {
        Ok(request_query) => request_query,
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "invalid_request".to_string(),
                    detail: e.to_string(),
                    source: "/items".to_string(),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    let pool = &data.db;

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request_query.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    // dbg!(&request_query);

    let shops = Shop::query(
        pool,
        &request_query.filter,
        &request_query.sorting,
        &request_query.pagination,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match shops {
        Ok(shops) => Ok(HttpResponse::Ok().json(ResponseType::new(
            shops,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: e.to_string(),
                    source: "/items".to_string(),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}
