use actix_web::{post, web, HttpResponse, Responder};
use mysk_lib::models::common::{
    requests::{FetchLevel, RequestType},
    response::{ErrorResponseType, ErrorType, MetadataType, PaginationType, ResponseType},
};
use uuid::Uuid;

use crate::{
    models::{
        auth::user::{OptionalUser, User},
        order::{
            request::{CreatableOrder, QueryableOrder, SortableOrder},
            Order,
        },
    },
    AppState,
};

#[post("/orders")]
pub async fn create_orders(
    data: web::Data<AppState>,
    request: web::Json<RequestType<Vec<CreatableOrder>, QueryableOrder, SortableOrder>>,
    user: OptionalUser,
) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;
    let omise_secret = &data.env.omise_secret_key;

    let data = match &request.data {
        Some(data) => data,
        None => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: "request body is empty".to_string(),
                    source: format!("/orders"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            return Ok(HttpResponse::BadRequest().json(response));
        }
    };

    // let user_id = match user {
    //     User::IdOnly(user) => user.id,
    //     User::Compact(user) => user.id,
    //     User::Default(user) => user.id,
    //     User::Detailed(user) => user.id,
    // };

    let user_id = match user.0 {
        Some(user) => match user {
            User::IdOnly(user) => Some(user.id),
            User::Compact(user) => Some(user.id),
            User::Default(user) => Some(user.id),
            User::Detailed(user) => Some(user.id),
        },
        None => None,
    };

    // let item_ids: Result<!, _> = CreatableItem::bulk_insert(data.to_vec(), pool).await;

    let mut order_ids = Vec::new();

    for order in data {
        let order_id = order.insert(pool, &omise_secret, user_id).await;

        order_ids.push(order_id);
    }

    let order_ids: Result<Vec<Uuid>, _> = order_ids.into_iter().collect();

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

    let order_ids = match order_ids {
        Ok(order_ids) => order_ids,
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 400,
                    error_type: "bad_request".to_string(),
                    detail: err.to_string(),
                    source: format!("/orders"),
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

    let orders = Order::get_by_ids(
        pool,
        order_ids,
        Some(&fetch_level),
        Some(&descendant_fetch_level),
    )
    .await;

    match orders {
        Ok(orders) => {
            let response: ResponseType<Vec<Order>> =
                ResponseType::new(orders, Some(MetadataType::new(None::<PaginationType>)));

            Ok(HttpResponse::Ok().json(response))
        }
        Err(err) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    error_type: "internal_server_error".to_string(),
                    detail: err.to_string(),
                    source: format!("/orders"),
                },
                Some(MetadataType::new(None::<PaginationType>)),
            );

            Ok(HttpResponse::InternalServerError().json(response))
        }
    }
}
