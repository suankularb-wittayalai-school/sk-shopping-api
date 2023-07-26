use actix_web::{get, HttpResponse, Responder};
use mysk_lib::models::common::response::ResponseType;

use crate::models::auth::user::User;

#[get("/auth/user")]
async fn get_user(user: User) -> impl Responder {
    HttpResponse::Ok().json(ResponseType::new(user, None))
}
