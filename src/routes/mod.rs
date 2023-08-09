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
pub(crate) mod orders;
pub(crate) mod shops;

use doc::ApiDoc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health::health_check);

    cfg.service(auth::google::google_oauth_handler);
    cfg.service(auth::user::get_user);
    cfg.service(auth::user_wishlists::get_user_wishlists);
    cfg.service(auth::create_user_addresses::create_user_addresses);
    cfg.service(auth::delete_user_addresses::delete_user_addresses);

    cfg.service(items::item_detail::item_detail);
    cfg.service(items::query_items::query_items);
    cfg.service(items::add_to_cart::add_to_cart);

    cfg.service(listings::listing_detail::listing_detail);
    cfg.service(listings::query_listings::query_listings);
    cfg.service(listings::delete_listings::delete_listings);
    cfg.service(listings::update_listing_by_id::update_listing_by_id);
    cfg.service(listings::add_to_wishlist::add_to_wishlist);

    cfg.service(collections::collection_detail::collection_detail);
    cfg.service(collections::query_collections::query_collections);
    cfg.service(collections::create_collections::create_collections);
    cfg.service(collections::delete_collections::delete_collections);
    cfg.service(collections::delete_collection_by_id::delete_collection);
    cfg.service(collections::update_collection_by_id::update_collection_by_id);

    cfg.service(shops::shop_detail::shop_detail);
    cfg.service(shops::query_shops::query_shops);
    cfg.service(shops::update_shop_by_id::update_shop_by_id);

    cfg.service(orders::order_detail::order_detail);
    cfg.service(orders::query_orders::query_orders);

    cfg.service(category::all_categories::all_categories);
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
}
