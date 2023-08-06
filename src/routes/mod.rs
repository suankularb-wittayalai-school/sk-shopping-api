use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) mod auth;
pub(crate) mod category;
pub(crate) mod collections;
mod doc;
pub(crate) mod health;
pub(crate) mod items;
pub(crate) mod listings;
pub(crate) mod shops;

use doc::ApiDoc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_check);
    cfg.service(auth::google::google_oauth_handler);
    cfg.service(auth::user::get_user);
    cfg.service(items::item_detail::item_detail);
    cfg.service(items::query_items::query_items);
    cfg.service(listings::listing_detail::listing_detail);
    cfg.service(collections::collection_detail::collection_detail);
    cfg.service(shops::shop_detail::shop_detail);
    cfg.service(category::all_categories::all_categories);
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
}
