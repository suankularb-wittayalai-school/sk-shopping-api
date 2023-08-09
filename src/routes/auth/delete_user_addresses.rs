use actix_web::{delete, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::RequestType,
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::{address::Address, auth::user::User},
    AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct Placeholder {}

#[delete("/auth/user/addresses")]
pub async fn delete_user_addresses(
    user: User,
    data: web::Data<AppState>,
    request: web::Json<RequestType<Vec<Uuid>, Placeholder, Placeholder>>,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;

    let user_id = match user {
        User::IdOnly(u) => u.id,
        User::Compact(u) => u.id,
        User::Default(u) => u.id,
        User::Detailed(u) => u.id,
    };

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "request body is empty".to_string(),
                    source: format!("/auth/user/addresses"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    for address_id in data {
        let res = sqlx::query(
            r#"
            SELECT COUNT(id) FROM addresses WHERE owner_id = $1 AND id = $2
            "#,
        )
        .bind(user_id)
        .bind(address_id)
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
                            "user {} is not a owner of address {}",
                            user_id, address_id
                        ),
                        source: format!("/auth/user/addresses"),
                    },
                    None::<MetadataType>,
                );

                return Ok(HttpResponse::Forbidden().json(response));
            }
        }
    }

    let res = Address::delete_by_ids(pool, data.to_vec()).await;

    match res {
        Ok(_) => {
            let response: ResponseType<Option<String>> =
                ResponseType::new(None, Some(MetadataType::new(None::<PaginationType>)));

            Ok(HttpResponse::NoContent().json(response))
        }
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: err.to_string(),
                    source: format!("/auth/user/addresses"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            Ok(HttpResponse::BadRequest().json(response))
        }
    }
}
