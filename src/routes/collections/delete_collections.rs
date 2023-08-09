use actix_web::{delete, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::RequestType,
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        collection::{
            db::CollectionTable,
            request::{QueryableCollection, SortableCollection},
        },
    },
    AppState,
};

#[delete("/collections")]
pub async fn delete_collections(
    data: web::Data<AppState>,
    request: web::Json<RequestType<Vec<Uuid>, QueryableCollection, SortableCollection>>,
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

    for collection_id in data {
        let res = sqlx::query(
            r#"
            SELECT COUNT(id) FROM shop_managers INNER JOIN collections ON collections.shop_id = shop_managers.shop_id WHERE user_id = $1 AND collections.id = $2
            "#,
        )
        .bind(user_id)
        .bind(collection_id)
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
                            user_id, collection_id
                        ),
                        source: format!("/collections"),
                    },
                    None::<MetadataType>,
                );

                return Ok(HttpResponse::Forbidden().json(response));
            }
        }
    }

    let res = CollectionTable::bulk_delete(pool, data.to_vec()).await;

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

    match res {
        Ok(_) => Ok(HttpResponse::NoContent().json(ResponseType::new(
            None::<bool>,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
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

            Ok(HttpResponse::BadRequest().json(response))
        }
    }
}
