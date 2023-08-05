use actix_web::error::{ErrorNotFound, ErrorUnauthorized};
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpRequest};
use chrono::{DateTime, Utc};
use futures::Future as FutureTrait;
use jsonwebtoken::{decode, DecodingKey, Validation};
use mysk_lib::models::common::requests::FetchLevel;
use mysk_lib::models::common::response::{ErrorResponseType, ErrorType};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, Error, FromRow, PgPool};

use std::pin::Pin;

use crate::models::auth::oauth::TokenClaims;
use crate::AppState;

use super::oauth::GoogleUserResult;
// use sqlx::types::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, FromRow)]
pub struct UserTable {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub profile: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl UserTable {
    pub async fn get_by_email(pool: &PgPool, email: &str) -> Option<Self> {
        sqlx::query_as::<_, UserTable>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(pool)
            .await
            .ok()
    }

    pub async fn create_user_from_google(
        pool: &PgPool,
        google_user: GoogleUserResult,
    ) -> Result<Self, Error> {
        sqlx::query_as::<_, UserTable>("INSERT INTO users (username, email, profile, first_name, last_name) VALUES ($1, $2, $3, $4, $5) RETURNING *")
            .bind(google_user.name)
            .bind(google_user.email)
            .bind(google_user.picture)
            .bind(google_user.given_name)
            .bind(google_user.family_name)
            .fetch_one(pool)
            .await
    }

    pub async fn from_id(pool: &PgPool, id: Uuid) -> Result<Self, Error> {
        sqlx::query_as::<_, UserTable>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyUser {
    pub id: Uuid,
}

impl IdOnlyUser {
    pub fn from_table(user: UserTable) -> Self {
        Self { id: user.id }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompactUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub profile: Option<String>,
}

impl CompactUser {
    pub fn from_table(user: UserTable) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            profile: user.profile,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub profile: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl DefaultUser {
    pub fn from_table(user: UserTable) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            profile: user.profile,
            first_name: user.first_name,
            last_name: user.last_name,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub profile: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    // pub addresses: Vec<Address>
}

impl DetailedUser {
    pub async fn from_table(user: UserTable) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            profile: user.profile,
            first_name: user.first_name,
            last_name: user.last_name,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum User {
    IdOnly(IdOnlyUser),
    Compact(CompactUser),
    Default(DefaultUser),
    Detailed(DetailedUser),
}

impl User {
    pub async fn from_table(user: UserTable, fetch_level: Option<&FetchLevel>) -> Self {
        match fetch_level {
            Some(level) => match level {
                FetchLevel::IdOnly => Self::IdOnly(IdOnlyUser::from_table(user)),
                FetchLevel::Compact => Self::Compact(CompactUser::from_table(user)),
                FetchLevel::Default => Self::Default(DefaultUser::from_table(user)),
                FetchLevel::Detailed => Self::Detailed(DetailedUser::from_table(user).await),
            },
            None => Self::Default(DefaultUser::from_table(user)),
        }
    }

    pub async fn from_id(
        id: Uuid,
        pool: &PgPool,
        fetch_level: Option<&FetchLevel>,
    ) -> Result<Self, Error> {
        let user = UserTable::from_id(pool, id).await?;

        match fetch_level {
            Some(level) => match level {
                FetchLevel::IdOnly => Ok(Self::IdOnly(IdOnlyUser::from_table(user))),
                FetchLevel::Compact => Ok(Self::Compact(CompactUser::from_table(user))),
                FetchLevel::Default => Ok(Self::Default(DefaultUser::from_table(user))),
                FetchLevel::Detailed => Ok(Self::Detailed(DetailedUser::from_table(user).await)),
            },
            None => Ok(Self::Default(DefaultUser::from_table(user))),
        }
    }
}

impl From<UserTable> for User {
    fn from(user: UserTable) -> Self {
        Self::Default(DefaultUser::from_table(user))
    }
}

impl Serialize for User {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            User::IdOnly(user) => user.serialize(serializer),
            User::Compact(user) => user.serialize(serializer),
            User::Default(user) => user.serialize(serializer),
            User::Detailed(user) => user.serialize(serializer),
        }
    }
}

impl FromRequest for User {
    type Error = ActixWebError;
    type Future = Pin<Box<dyn FutureTrait<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(state) => state,
            None => {
                return Box::pin(async {
                    Err(ErrorNotFound(ErrorResponseType::new(
                        ErrorType {
                            id: Uuid::new_v4().to_string(),
                            code: 500,
                            detail: "AppState not found".to_owned(),
                            error_type: "internal_server_error".to_owned(),
                            source: "".to_owned(),
                        },
                        None,
                    )))
                })
            }
        };

        let pool = app_state.db.clone();
        let jwt_secret = app_state.env.jwt_secret.clone();

        let auth_header = req.headers().get(http::header::AUTHORIZATION);

        let token = match auth_header {
            Some(token) => match token.to_str() {
                Ok(token) => token,
                Err(_) => {
                    return Box::pin(async {
                        // return 401 unauthorized if the token is not a string as ResponseType
                        Err(ErrorUnauthorized(ErrorResponseType::new(
                            ErrorType {
                                id: "401".to_string(),
                                detail: "Invalid token".to_string(),
                                code: 401,
                                error_type: "invalid_token".to_string(),
                                source: "".to_string(),
                            },
                            None,
                        )))
                    });
                }
            },
            None => {
                return Box::pin(async {
                    Err(ErrorUnauthorized(ErrorResponseType::new(
                        ErrorType {
                            id: "401".to_string(),
                            detail: "Missing Token".to_string(),
                            code: 401,
                            error_type: "missing_token".to_string(),
                            source: "".to_string(),
                        },
                        None,
                    )))
                })
            }
        };

        let token = token.trim_start_matches("Bearer ");

        let claims = match decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::default(),
        ) {
            Ok(claims) => claims,
            Err(_) => {
                return Box::pin(async {
                    Err(ErrorUnauthorized(ErrorResponseType::new(
                        ErrorType {
                            id: "401".to_string(),
                            detail: "Invalid token".to_string(),
                            code: 401,
                            error_type: "invalid_token".to_string(),
                            source: "".to_string(),
                        },
                        None,
                    )))
                })
            }
        };

        let user_id = match Uuid::parse_str(&claims.claims.sub) {
            Ok(user_id) => user_id,
            Err(_) => {
                return Box::pin(async {
                    Err(ErrorNotFound(ErrorResponseType::new(
                        ErrorType {
                            id: "404".to_string(),
                            detail: "User not found".to_string(),
                            code: 404,
                            error_type: "entity_not_found".to_string(),
                            source: "".to_string(),
                        },
                        None,
                    )))
                })
            }
        };

        Box::pin(async move {
            let user = User::from_id(user_id, &pool, None).await;

            match user {
                Ok(user) => Ok(user),
                Err(_) => Err(ErrorNotFound(ErrorResponseType::new(
                    ErrorType {
                        id: "404".to_string(),
                        detail: "User not found".to_string(),
                        code: 404,
                        error_type: "entity_not_found".to_string(),
                        source: "".to_string(),
                    },
                    None,
                ))),
            }
        })
    }
}
