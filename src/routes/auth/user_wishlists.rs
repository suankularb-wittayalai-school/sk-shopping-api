use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        listing::{
            request::{QueryableListing, SortableListing},
            Listing,
        },
    },
    AppState,
};

#[get("/auth/user/wishlists")]
pub async fn get_user_wishlists(
    user: User,
    data: web::Data<AppState>,
    request_query: web::Query<RequestType<Listing, QueryableListing, SortableListing>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request_query.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let user_id = match user {
        User::IdOnly(u) => u.id,
        User::Compact(u) => u.id,
        User::Default(u) => u.id,
        User::Detailed(u) => u.id,
    };

    let wishlists = sqlx::query(
        r#"
        SELECT
            listing_id
        FROM
            user_wishlists
        WHERE
            user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await;

    let wishlists = match wishlists {
        Ok(wishlists) => wishlists,
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/auth/user/wishlists".to_string(),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::NotFound().json(response));
        }
    };

    let wishlists = wishlists
        .into_iter()
        .map(|wishlist| wishlist.get::<Uuid, _>("listing_id"))
        .collect();

    let items = Listing::get_by_ids(
        pool,
        wishlists,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match items {
        Ok(items) => Ok(HttpResponse::Ok().json(ResponseType::new(
            items,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/auth/user/wishlists".to_string(),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
