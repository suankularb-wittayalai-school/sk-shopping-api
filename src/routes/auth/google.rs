use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    post, web, HttpResponse, Responder,
};
use chrono::{prelude::*, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use mysk_lib::models::common::response::{ErrorResponseType, ErrorType, ResponseType};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    models::auth::{
        oauth::{verify_id_token, GoogleUserResult, OAuthRequest, TokenClaims},
        user::UserTable,
    },
    AppState,
};

#[derive(Debug, Serialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: i64,
    token_type: String,
    scope: String,
    id_token: String,
}

#[post("/auth/sessions/oauth/google")]
async fn google_oauth_handler(
    data: web::Data<AppState>,
    query: web::Json<OAuthRequest>,
) -> impl Responder {
    // dbg!(query);

    let id_token: String = query.credential.to_owned();

    // dbg!(id_token.as_str());

    if id_token.is_empty() {
        return HttpResponse::Unauthorized().json(ErrorResponseType::new(
            ErrorType {
                id: Uuid::new_v4().to_string(),
                code: 500,
                detail: "id_token is empty".to_owned(),
                error_type: "invalid_token".to_owned(),
                source: "/auth/sessions/oauth/google".to_owned(),
            },
            None,
        ));
    }

    // decode id_token to get google user info with jwt and get access_token and verify it with google secret
    // let google_user = jsonwebtoken::decode(&id_token, key, validation)

    let google_id_data = match verify_id_token(&id_token, &data.env).await {
        Ok(data) => data,
        Err(err) => {
            return HttpResponse::Unauthorized().json(ErrorResponseType::new(
                ErrorType {
                    id: Uuid::new_v4().to_string(),
                    code: 500,
                    detail: err,
                    error_type: "invalid_token".to_owned(),
                    source: "/auth/sessions/oauth/google".to_owned(),
                },
                None,
            ));
        }
    };

    let google_user = GoogleUserResult::from_token_payload(google_id_data);

    dbg!(&google_user);

    // let mut vec = data.db.lock().unwrap();
    // let email = google_user.email.to_lowercase();
    // let user = vec.iter_mut().find(|user| user.email == email);

    let user = UserTable::get_by_email(&data.db, &google_user.email).await;

    let user_id = match user {
        Some(user) => user.id,
        None => {
            let user = UserTable::create_user_from_google(&data.db, google_user).await;

            match user {
                Ok(user) => user.id,
                Err(err) => {
                    return HttpResponse::InternalServerError().json(ErrorResponseType::new(
                        ErrorType {
                            id: Uuid::new_v4().to_string(),
                            code: 500,
                            detail: err.to_string(),
                            error_type: "user_not_created".to_owned(),
                            source: "/auth/sessions/oauth/google".to_owned(),
                        },
                        None,
                    ));
                }
            }
        }
    };

    let jwt_secret = data.env.jwt_secret.to_owned();
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(data.env.jwt_max_age)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    );

    match token {
        Ok(token) => {
            let cookie = Cookie::build("token", token.to_owned())
                .secure(true)
                .http_only(true)
                .max_age(ActixWebDuration::days(30))
                .same_site(actix_web::cookie::SameSite::Strict)
                .finish();

            let response: ResponseType<GoogleTokenResponse> = ResponseType::new(
                GoogleTokenResponse {
                    access_token: token,
                    expires_in: data.env.jwt_max_age * 60,
                    token_type: "Bearer".to_owned(),
                    scope: "email profile".to_owned(),
                    id_token,
                },
                None,
            );

            HttpResponse::Ok().cookie(cookie).json(response)
        }
        Err(err) => HttpResponse::InternalServerError().json(ErrorResponseType::new(
            ErrorType {
                id: Uuid::new_v4().to_string(),
                code: 500,
                detail: err.to_string(),
                error_type: "token_not_generated".to_owned(),
                source: "/auth/sessions/oauth/google".to_owned(),
            },
            None,
        )),
    }
}
