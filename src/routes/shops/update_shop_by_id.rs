use actix_web::{patch, web, HttpRequest, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        shop::{
            request::{QueryableShop, SortableShop, UpdatableShop},
            Shop,
        },
    },
    AppState,
};

#[patch("/shops/{shop_id}")]
pub async fn update_shop_by_id(
    data: web::Data<AppState>,
    shop_id: web::Path<Uuid>,
    request: web::Json<RequestType<UpdatableShop, QueryableShop, SortableShop>>,
    user: User,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let shop_id = shop_id.into_inner();

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "request body is empty".to_string(),
                    source: format!("/shops/{shop_id}"),
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
        SELECT COUNT(id) FROM shop_managers WHERE user_id = $1 AND shop_id = $2
        "#,
    )
    .bind(user_id)
    .bind(shop_id)
    .fetch_one(pool)
    .await;

    if let Ok(res) = res {
        if res.get::<Option<i64>, _>("count").unwrap_or(0) == 0 {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 403,
                    error_type: "forbidden".to_string(),
                    detail: "the user is not the shop manager".to_string(),
                    source: format!("/shops/{shop_id}"),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::Forbidden().json(response));
        }
    }

    // dbg!(data);

    let res = data.commit_changes(pool, shop_id).await;

    if res.is_err() {
        let response: ErrorResponseType = ErrorResponseType::new(
            ErrorType {
                id: Uuid::new_v4().to_string(),
                code: 400,
                error_type: "bad_request".to_string(),
                detail: res.err().unwrap().to_string(),
                source: format!("/shops/{shop_id}"),
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

    let shop = Shop::get_by_id(
        pool,
        shop_id,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match shop {
        Ok(shop) => Ok(HttpResponse::Ok().json(ResponseType::new(
            shop,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
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
