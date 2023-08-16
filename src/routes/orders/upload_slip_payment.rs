use actix_web::{patch, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    models::order::{
        request::{QueryableOrder, SortableOrder},
        Order,
    },
    utils::email::send_receipt_email,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct UpdateOrder {
    payment_slip_url: String,
}

#[patch("/orders/{order_id}/slip")]
pub async fn upload_slip_payment(
    data: web::Data<AppState>,
    order_id: web::Path<Uuid>,
    request: web::Json<RequestType<UpdateOrder, QueryableOrder, SortableOrder>>,
    // user: OptionalUser,
) -> Result<impl Responder, actix_web::Error> {
    let pool: &sqlx::Pool<sqlx::Postgres> = &data.db;
    let credential = &data.smtp_credential;
    let order_id = order_id.into_inner();

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "request body is empty".to_string(),
                    source: format!("/orders/{order_id}"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    // let user_id = match user.0 {
    //     Some(user) => match user {
    //         User::IdOnly(user) => Some(user.id),
    //         User::Compact(user) => Some(user.id),
    //         User::Default(user) => Some(user.id),
    //         User::Detailed(user) => Some(user.id),
    //     },
    //     None => None,
    // };

    let res = sqlx::query(
        r#"
        UPDATE orders
        SET payment_slip_url = $1, is_paid = true
        WHERE id = $2
        RETURNING id
        "#,
    )
    .bind(&data.payment_slip_url)
    .bind(order_id)
    .fetch_one(pool)
    .await;

    let res = match res {
        Ok(res) => res,
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: err.to_string(),
                    source: format!("/orders/{order_id}"),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::InternalServerError().json(response));
        }
    };

    let order_id = res.get::<Uuid, _>("id");

    let order = Order::get_by_id(
        pool,
        order_id,
        Some(&FetchLevel::Default),
        Some(&FetchLevel::Compact),
    )
    .await;

    let order = match order {
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: err.to_string(),
                    source: format!("/orders/{order_id}"),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::InternalServerError().json(response));
        }
        Ok(order) => order,
    };

    let res = send_receipt_email(credential, order);

    if let Err(err) = res {
        let response: ErrorResponseType = ErrorResponseType::new(
            ErrorType {
                id: Uuid::new_v4().to_string(),
                code: 500,
                error_type: "internal_server_error".to_string(),
                detail: err.to_string(),
                source: format!("/orders/{order_id}"),
            },
            None::<MetadataType>,
        );

        return Ok(HttpResponse::InternalServerError().json(response));
    }

    let fetch_level = match request.fetch_level.clone() {
        Some(fetch_level) => fetch_level,
        None => FetchLevel::Default,
    };

    let descendant_fetch_level = match request.descendant_fetch_level.clone() {
        Some(descendant_fetch_level) => descendant_fetch_level,
        None => FetchLevel::IdOnly,
    };

    let item = Order::get_by_id(
        pool,
        order_id,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match item {
        Ok(collection) => {
            let response: ResponseType<Order> = ResponseType::new(collection, None::<MetadataType>);

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "not_found".to_string(),
                    detail: err.to_string(),
                    source: format!("/orders/{order_id}"),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
