use actix_web::{post, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::RequestType,
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    models::{
        auth::user::User,
        item::{
            request::{QueryableItem, SortableItem},
            CartItem, Item,
        },
    },
    AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToCartRequest {
    pub amount: i64,
}

#[post("/items/{item_id}/add-to-cart")]
pub async fn add_to_cart(
    data: web::Data<AppState>,
    item_id: web::Path<Uuid>,
    request: web::Json<RequestType<AddToCartRequest, QueryableItem, SortableItem>>,
    user: User,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let item_id = item_id.into_inner();

    let user_id = match user {
        User::IdOnly(user) => user.id,
        User::Compact(user) => user.id,
        User::Default(user) => user.id,
        User::Detailed(user) => user.id,
    };

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "missing request body".to_string(),
                    source: format!("/items/{item_id}/add-to-cart"),
                },
                None::<MetadataType>,
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    let cart_item = CartItem {
        item: Item::IdOnly(crate::models::item::IdOnlyItem { id: item_id }),
        amount: data.amount,
    };

    let res = cart_item.add_to_user_cart(user_id, pool).await;

    match res {
        Ok(_) => Ok(HttpResponse::NoContent().json(ResponseType::new(
            None::<bool>,
            Some(MetadataType::new(None::<PaginationType>)),
        ))),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: e.to_string(),
                    source: format!("/items/{item_id}/add-to-cart"),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}
