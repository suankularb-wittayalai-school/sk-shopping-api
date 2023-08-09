use actix_web::{patch, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        item::{
            request::{QueryableItem, SortableItem, UpdatableItem},
            Item,
        },
    },
    AppState,
};

#[patch("/items/{item_id}")]
pub async fn update_item_by_id(
    data: web::Data<AppState>,
    item_id: web::Path<Uuid>,
    request: web::Json<RequestType<UpdatableItem, QueryableItem, SortableItem>>,
    user: User,
) -> Result<impl Responder, actix_web::Error> {
    let pool: &sqlx::Pool<sqlx::Postgres> = &data.db;
    let item_id = item_id.into_inner();

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "request body is empty".to_string(),
                    source: format!("/items/{item_id}"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    let user_id = match user {
        User::IdOnly(user) => user.id,
        User::Compact(user) => user.id,
        User::Default(user) => user.id,
        User::Detailed(user) => user.id,
    };

    let res = sqlx::query(
        r#"
        SELECT COUNT(id) FROM shop_managers WHERE user_id = $1 AND shop_id IN (SELECT shop_id FROM listings WHERE id IN (SELECT listing_id FROM items WHERE id = $2)))
        "#,
    )
    .bind(user_id)
    .bind(item_id)
    .fetch_one(pool)
    .await;

    if let Ok(res) = res {
        if res.get::<Option<i64>, _>("count").unwrap_or(0) == 0 {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 403,
                    error_type: "forbidden".to_string(),
                    detail: format!("user {} is not a manager of shop {}", user_id, item_id),
                    source: format!("/items/{item_id}"),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::Forbidden().json(response));
        }
    }

    let res = data.commit_changes(pool, item_id).await;

    if res.is_err() {
        let response: ErrorResponseType = ErrorResponseType::new(
            ErrorType {
                id: Uuid::new_v4().to_string(),
                code: 400,
                error_type: "bad_request".to_string(),
                detail: res.err().unwrap().to_string(),
                source: format!("/items/{item_id}"),
            },
            Some(MetadataType::new(None::<PaginationType>)),
        );

        return Ok(HttpResponse::BadRequest().json(response));
    };

    let fetch_level = match request.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request.descendant_fetch_level.clone() {
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
        Ok(collection) => {
            let response: ResponseType<Item> = ResponseType::new(collection, None::<MetadataType>);

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "not_found".to_string(),
                    detail: err.to_string(),
                    source: format!("/items/{item_id}"),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
