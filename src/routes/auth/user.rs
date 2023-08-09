use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{models::auth::user::User, AppState};

#[derive(Debug, serde::Deserialize)]
pub struct Placeholder;

#[get("/auth/user")]
async fn get_user(
    user: User,
    data: web::Data<AppState>,
    request_query: web::Query<RequestType<User, Placeholder, Placeholder>>,
) -> impl Responder {
    let pool = &data.db;

    let fetch_level = match request_query.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let user_id = match user {
        User::IdOnly(u) => u.id,
        User::Compact(u) => u.id,
        User::Default(u) => u.id,
        User::Detailed(u) => u.id,
    };

    let user = User::from_id(user_id, pool, Some(&fetch_level)).await;

    // HttpResponse::Ok().json(ResponseType::new(user, None))

    match user {
        Ok(user) => HttpResponse::Ok().json(ResponseType::new(
            user,
            Some(MetadataType::new(None::<PaginationType>)),
        )),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponseType::new(
            ErrorType {
                id: Uuid::new_v4().to_string(),
                code: 500,
                error_type: "internal_server_error".to_string(),
                detail: e.to_string(),
                source: format!("/auth/user"),
            },
            Some(MetadataType::new(None::<PaginationType>)),
        )),
    }
}
