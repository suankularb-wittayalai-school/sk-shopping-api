use actix_web::{get, web, HttpResponse, Responder};
use mysk_lib::models::common::response::{ErrorResponseType, ErrorType, MetadataType};
use uuid::Uuid;

use crate::{models::category::Category, AppState};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryableCategory;

#[get("/categories")]
pub async fn all_categories(data: web::Data<AppState>) -> Result<impl Responder, actix_web::Error> {
    let pool = &data.db;

    let categories = Category::get_all(pool).await;

    match categories {
        Ok(categories) => Ok(HttpResponse::Ok().json(categories)),
        Err(e) => {
            let response: ErrorResponseType = ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 404,
                    error_type: "entity_not_found".to_string(),
                    detail: e.to_string(),
                    source: "/categories".to_string(),
                },
                None::<MetadataType>,
            );

            Ok(HttpResponse::NotFound().json(response))
        }
    }
}
