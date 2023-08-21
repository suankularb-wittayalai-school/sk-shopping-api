use std::fs::OpenOptions;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use futures::StreamExt;
use mysk_lib::models::common::requests::FetchLevel;

use crate::{
    models::order::{
        gbprimpay::{GbPrimePayWebHookRequest, ResultCode},
        Order,
    },
    utils::email::send_receipt_email,
    AppState,
};

// const MAX_SIZE: usize = 262_144;

#[post("/orders/webhook")]
pub async fn update_order_webhook(
    data: web::Data<AppState>,
    request: web::Json<GbPrimePayWebHookRequest>,
    // request: HttpRequest, // user: OptionalUser,
    body: web::Bytes,
) -> Result<impl Responder, actix_web::Error> {
    let pool: &sqlx::Pool<sqlx::Postgres> = &data.db;
    let credential = &data.smtp_credential;

    let json_string = std::str::from_utf8(&body).unwrap();

    dbg!(&json_string);

    dbg!(serde_json::from_str::<GbPrimePayWebHookRequest>(
        &json_string
    ));

    // dbg!(&request);

    // Ok(HttpResponse::Ok().finish())

    let data = request.into_inner();

    if data.result_code != ResultCode::Success {
        return Ok(HttpResponse::Ok().finish());
    }

    let res = data.update_order_status(pool).await;

    match res {
        Err(e) => {
            println!("Error: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
        Ok(order_id) => {
            let order = Order::get_by_id(
                pool,
                order_id,
                Some(&FetchLevel::Default),
                Some(&FetchLevel::Compact),
            )
            .await;

            match order {
                Err(e) => {
                    println!("Error: {}", e);
                    Ok(HttpResponse::InternalServerError().finish())
                }
                Ok(order) => {
                    let res = send_receipt_email(credential, order);

                    match res {
                        Err(e) => {
                            println!("Error: {}", e);
                            Ok(HttpResponse::InternalServerError().finish())
                        }
                        Ok(_) => Ok(HttpResponse::Ok().finish()),
                    }
                }
            }
        }
    }
}
