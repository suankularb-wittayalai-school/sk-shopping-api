use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{
    models::listing::{
        request::{QueryableListing, SortableListing},
        Listing,
    },
    AppState,
};

#[get("/listings")]
pub async fn query_listings(
    data: web::Data<AppState>,
    request: HttpRequest,
) -> Result<impl Responder, actix_web::Error> {
    let request_query = serde_qs::from_str::<RequestType<Listing, QueryableListing, SortableListing>>(
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
                    source: "/listings".to_string(),
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

    dbg!(&request_query);

    let listings = Listing::query(
        pool,
        &request_query.filter,
        &request_query.sorting,
        &request_query.pagination,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match listings {
        Ok(listings) => Ok(HttpResponse::Ok().json(ResponseType::new(
            listings,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: e.to_string(),
                    source: "/listings".to_string(),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}
