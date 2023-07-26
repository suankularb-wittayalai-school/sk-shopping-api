use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use super::health::{HealthCheckResponse, __path_health_check};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut();

        let components = match components {
            Some(components) => components,
            None => return,
        };

        components.add_security_scheme(
            "JWT Token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("Bearer")
                    .build(),
            ),
        )
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SK Actix Web Example",
        description = "Actix Web Template for Suankularb Wittayalai School related web service",
    ),
    paths(health_check),
    components(schemas(HealthCheckResponse)),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
