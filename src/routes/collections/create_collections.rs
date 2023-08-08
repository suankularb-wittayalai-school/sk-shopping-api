use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        collection::{
            request::{CreatableCollection, QueryableCollection, SortableCollection},
            Collection,
        },
    },
    AppState,
};

#[post("/collections")]
pub async fn create_collections(
    data: web::Data<AppState>,
    request: web::Json<
        RequestType<Vec<CreatableCollection>, QueryableCollection, SortableCollection>,
    >,
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
                    source: format!("/collections"),
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

    for collection in data {
        let res = sqlx::query(
            r#"
            SELECT COUNT(id) FROM shop_managers WHERE user_id = $1 AND shop_id = $2
            "#,
        )
        .bind(user_id)
        .bind(collection.shop_id)
        .fetch_one(pool)
        .await;

        if let Ok(res) = res {
            if res.get::<Option<i64>, _>("count").unwrap_or(0) == 0 {
                let response: ErrorResponseType = ErrorResponseType::new(
                    ErrorType {
                        id: Uuid::new_v4().to_string(),
                        code: 403,
                        error_type: "forbidden".to_string(),
                        detail: format!(
                            "user {} is not a manager of shop {}",
                            user_id, collection.shop_id
                        ),
                        source: format!("/collections"),
                    },
                    None::<MetadataType>,
                );

                return Ok(HttpResponse::Forbidden().json(response));
            }
        }
    }

    let collection_ids = CreatableCollection::bulk_insert(data.to_vec(), pool).await;

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

    let collection_ids = match collection_ids {
        Ok(collection_ids) => collection_ids,
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: err.to_string(),
                    source: format!("/collections"),
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

    let collections = Collection::get_by_ids(
        pool,
        collection_ids,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match collections {
        Ok(collections) => {
            let response: ResponseType<Vec<Collection>> =
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
                    source: format!("/collections"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}
