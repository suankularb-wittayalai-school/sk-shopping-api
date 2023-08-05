use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub(crate) mod auth;
mod doc;
pub(crate) mod health;
pub(crate) mod items;
pub(crate) mod listings;

use doc::ApiDoc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_check);
    cfg.service(auth::google::google_oauth_handler);
    cfg.service(auth::user::get_user);
    cfg.service(items::item_detail::item_detail);
    cfg.service(listings::listing_detail::listing_detail);
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
}
