use actix_web::{post, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::RequestType,
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{
    models::{address::Address, auth::user::User},
    AppState,
};

#[derive(Debug, serde::Deserialize)]
pub struct Placeholder {}

#[post("/auth/user/addresses")]
pub async fn create_user_addresses(
    user: User,
    data: web::Data<AppState>,
    request: web::Json<RequestType<Vec<Address>, Placeholder, Placeholder>>,
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

    let mut addresses = Vec::new();

    for address in data {
        let res = address.create(pool, user_id).await;

        match res {
            Ok(address) => addresses.push(address),
            Err(e) => {
                let response: ErrorResponseType = ErrorResponseType::new(
                    ErrorType {
                        id: Uuid::new_v4().to_string(),
                        code: 500,
                        error_type: "internal_server_error".to_string(),
                        detail: e.to_string(),
                        source: format!("/auth/user/addresses"),
                    },
                    Some(MetadataType::new(None::<PaginationType>)),
                );

                return Ok(HttpResponse::InternalServerError().json(response));
            }
        }
    }

    let response: ResponseType<Vec<Address>> =
        ResponseType::new(addresses, Some(MetadataType::new(None::<PaginationType>)));

    Ok(HttpResponse::Ok().json(response))
}
