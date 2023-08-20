use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, web, App, HttpServer};
use dotenv::dotenv;
use lettre::transport::smtp::authentication::Credentials;
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

mod models;
mod routes;
mod utils;

pub struct AppState {
    db: Pool<Postgres>,
    smtp_credential: Credentials,
    env: utils::common::config::Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();
    env_logger::init();

    let env = utils::common::config::Config::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let smtp_credential = Credentials::new(
        env.google_email_user.clone(),
        env.google_email_password.clone(),
    );

    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    // builder
    //     .set_private_key_file("ssl/privkey.pem", SslFiletype::PEM)
    //     .unwrap();
    // builder.set_certificate_chain_file("ssl/cert.pem").unwrap();

    println!("🚀 Server started successfully");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin_fn(|origin, req_head| {
                // if req_head.method == "POST"  and it is fetching /orders/webhook
                // allow all origin
                dbg!(&req_head);
                dbg!(&origin);
                dbg!(req_head.uri.path() == "/orders/webhook");
                dbg!(req_head.method.as_str());

                if (req_head.method.as_str() == "POST" || req_head.method.as_str() == "OPTIONS")
                    && req_head.uri.path() == "/orders/webhook"
                {
                    dbg!("allow all origin");
                    return true;
                }
                origin.as_bytes().ends_with(b".skkornor.org")
                    || origin.as_bytes().ends_with(b".gbprimepay.com")
                    || origin.as_bytes().ends_with(b".globalprimepay.com")
                    || origin.as_bytes().ends_with(b".mysk.school")
                    || origin.as_bytes().ends_with(b".shopping.skkornor.org")
            })
            // .allowed_origin("http://localhost:3000")
            // .allowed_origin("http://127.0.0.1:3000")
            // .allowed_origin("http://[::1]:3000")
            // .allowed_origin("http://localhost:8000")
            // .allowed_origin("http://localhost")
            // .allowed_origin("https://mysk.school")
            // .allowed_origin("https://mysk.school")
            // .allowed_origin("https://shopping.skkornor.org")
            // .allowed_origin("https://preview.shopping.skkornor.org")
            // .allowed_origin("https://pr.shopping.skkornor.org")
            // // allow origin for gbprimepay to access webhook
            // .allowed_origin("https://api.globalprimepay.com")
            // .allowed_origin("https://api.gbprimepay.com")
            // .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "PUT"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                header::ACCEPT,
                // Custom headers
                header::HeaderName::from_lowercase(b"x-api-key").unwrap(),
            ])
            .supports_credentials();
        App::new()
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                env: env.clone(),
                smtp_credential: smtp_credential.clone(),
            }))
            // .service(web::scope("/api/v1").configure(routes::config))
            .configure(routes::config)
            .wrap(cors)
            .wrap(Logger::default())
    })
    // .bind_openssl(("0.0.0.0", 4430), builder)?
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
