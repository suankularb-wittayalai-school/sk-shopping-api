use actix_web::{post, web, HttpResponse, Responder};
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
            request::{CreatableItem, QueryableItem, SortableItem},
            Item,
        },
    },
    AppState,
};

#[post("/items")]
pub async fn create_items(
    data: web::Data<AppState>,
    request: web::Json<RequestType<Vec<CreatableItem>, QueryableItem, SortableItem>>,
    user: User,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "request body is empty".to_string(),
                    source: format!("/items"),
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

    for item in data {
        let res = match item.listing_id {
            Some(listing_id) => {
                sqlx::query(
                    r#"
                    SELECT COUNT(id) FROM listings WHERE id = $1 AND shop_id IN (SELECT shop_id FROM shop_managers WHERE user_id = $2)
                    "#,
                )
                .bind(listing_id)
                .bind(user_id)
                .fetch_one(pool)
                .await
            }
            None => {
                let shop_id = match item.shop_id {
                    Some(shop_id) => shop_id,
                    None => {
                        let response: ErrorResponseType = ErrorResponseType::new(
                            ErrorType {
                                id: Uuid::new_v4().to_string(),
                                code: 400,
                                error_type: "bad_request".to_string(),
                                detail: "missing shop_id".to_string(),
                                source: format!("/items"),
                            },
                            None::<MetadataType>,
                        );

                        return Ok(HttpResponse::BadRequest().json(response));
                    }
                };

                sqlx::query(
                    r#"
                    SELECT COUNT(id) FROM shop_managers WHERE user_id = $1 AND shop_id = $2
                    "#,
                )
                .bind(user_id)
                .bind(shop_id)
                .fetch_one(pool)
                .await
            }
        };

        if let Ok(res) = res {
            if res.get::<Option<i64>, _>("count").unwrap_or(0) == 0 {
                let response: ErrorResponseType = ErrorResponseType::new(
                    ErrorType {
                        id: Uuid::new_v4().to_string(),
                        code: 403,
                        error_type: "forbidden".to_string(),
                        detail: format!("user {} is not a manager of a shop", user_id),
                        source: format!("/items"),
                    },
                    None::<MetadataType>,
                );

                return Ok(HttpResponse::Forbidden().json(response));
            }
        }
    }

    let item_ids = CreatableItem::bulk_insert(data.to_vec(), pool).await;

    // if collection_ids.is_err() {
    //     let response: ErrorResponseType = ErrorResponseType::new(
    //         ErrorType {
    //             id: Uuid::new_v4().to_string(),
    //             code: 400,
    //             error_type: "bad_request".to_string(),
    //             detail: collection_ids.err().unwrap().to_string(),
    //             source: format!("/collections"),
    //         },
    //         Some(MetadataType::new(None::<PaginationType>)),
    //     );

    //     return Ok(HttpResponse::BadRequest().json(response));
    // };

    let item_ids = match item_ids {
        Ok(item_ids) => item_ids,
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: err.to_string(),
                    source: format!("/items"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    let fetch_level = match request.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let collections = Item::get_by_ids(
        pool,
        item_ids,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match collections {
        Ok(collections) => {
            let response: ResponseType<Vec<Item>> =
                ResponseType::new(collections, Some(MetadataType::new(None::<PaginationType>)));

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: err.to_string(),
                    source: format!("/items"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}
