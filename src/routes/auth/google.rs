use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    get, post, web, HttpResponse, Responder,
};
use chrono::{prelude::*, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::header::LOCATION;
use uuid::Uuid;

use crate::{
    models::auth::{
        oauth::{get_google_user, request_token, OAuthRequest, TokenClaims},
        user::User,
    },
    AppState,
};

#[post("/auth/sessions/oauth/google")]
async fn google_oauth_handler(
    data: web::Data<AppState>,
    query: web::Json<OAuthRequest>,
) -> impl Responder {
    // dbg!(query);

    let id_token: String = query.credential.to_owned();

    dbg!(id_token.as_str());

    if id_token.is_empty() {
        return HttpResponse::Unauthorized().json(
            serde_json::json!({"status": "fail", "message": "Authorization code not provided!"}),
        );
    }

    // decode id_token to get google user info with jwt and get access_token and verify it with google secret
    // let google_user = jsonwebtoken::decode(&id_token, key, validation)

    let google_user = get_google_user(&token_response.access_token, &token_response.id_token).await;
    if google_user.is_err() {
        let message = match google_user.err() {
            Some(err) => err.to_string(),
            None => "Failed to get google user".to_owned(),
        };
        return HttpResponse::BadGateway()
            .json(serde_json::json!({"status": "fail", "message": message}));
    }

    let google_user = match google_user {
        Ok(user) => user,
        Err(err) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": err.to_string()}))
        }
    };

    dbg!(google_user);

    // let mut vec = data.db.lock().unwrap();
    // let email = google_user.email.to_lowercase();
    // let user = vec.iter_mut().find(|user| user.email == email);

    let user: Option<User> = todo!("Find user by email");

    let user_id = if user.is_some() {
        todo!("Query user id from database")
    } else {
        todo!("Create new user")
    };

    let jwt_secret = data.env.jwt_secret.to_owned();
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(data.env.jwt_max_age)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id,
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token)
        .path("/")
        .max_age(ActixWebDuration::new(60 * data.env.jwt_max_age, 0))
        .http_only(true)
        .finish();

    let frontend_origin = data.env.client_origin.to_owned();
    let mut response = HttpResponse::Found();
    response.append_header((LOCATION, format!("{}", frontend_origin)));
    response.cookie(cookie);
    response.finish()
}
