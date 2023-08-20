use actix_web::{patch, web, HttpResponse, Responder};
use mysk_lib::models::common::requests::FetchLevel;

use crate::{
    models::order::{gbprimpay::GbPrimePayWebHookRequest, Order},
    utils::email::send_receipt_email,
    AppState,
};

#[patch("/orders/webhook")]
pub async fn upload_slip_payment(
    data: web::Data<AppState>,
    request: web::Json<GbPrimePayWebHookRequest>,
    // user: OptionalUser,
) -> Result<impl Responder, actix_web::Error> {
    let pool: &sqlx::Pool<sqlx::Postgres> = &data.db;
    let credential = &data.smtp_credential;

    let data = request.into_inner();

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
