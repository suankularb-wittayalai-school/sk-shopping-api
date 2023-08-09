use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        item::{
            request::{QueryableItem, SortableItem},
            CartItem,
        },
    },
    AppState,
};

#[get("/auth/user/carts")]
pub async fn get_user_cart_items(
    user: User,
    data: web::Data<AppState>,
    request_query: web::Query<RequestType<CartItem, QueryableItem, SortableItem>>,
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

    let items = CartItem::get_by_user_id(
        pool,
        user_id,
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
