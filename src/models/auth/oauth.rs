use crate::{lib::common::config::Config, AppState};
use actix_web::web;
use jsonwebtoken::Validation;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct OAuthRequest {
    pub credential: String,
}

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub id_token: String,
}

#[derive(Deserialize, Debug)]
pub struct GoogleUserResult {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

pub async fn request_token(
    id_token: &str,
    data: &web::Data<AppState>,
) -> Result<OAuthResponse, Box<dyn Error>> {
    let redirect_url = data.env.google_oauth_redirect_url.to_owned();
    let client_secret = data.env.google_oauth_client_secret.to_owned();
    let client_id = data.env.google_oauth_client_id.to_owned();

    let root_url = "https://oauth2.googleapis.com/token";
    let client = Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("redirect_uri", redirect_url.as_str()),
        ("code", id_token),
    ];
    let response = client.post(root_url).form(&params).send().await?;

    if response.status().is_success() {
        let oauth_response = response.json::<OAuthResponse>().await?;
        Ok(oauth_response)
    } else {
        let message = response.text().await?;
        Err(From::from(message))
    }
}

pub async fn get_google_user(
    access_token: &str,
    id_token: &str,
) -> Result<GoogleUserResult, Box<dyn Error>> {
    let client = Client::new();
    let mut url = Url::parse("https://www.googleapis.com/oauth2/v1/userinfo").unwrap();
    url.query_pairs_mut().append_pair("alt", "json");
    url.query_pairs_mut()
        .append_pair("access_token", access_token);

    let response = client.get(url).bearer_auth(id_token).send().await?;

    if response.status().is_success() {
        let user_info = response.json::<GoogleUserResult>().await?;
        Ok(user_info)
    } else {
        let message = "An error occurred while trying to retrieve user information.";
        Err(From::from(message))
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenPayload {
    // Add fields here as needed to capture the claims from the ID token
    // For example: iss, aud, exp, sub, email, etc.
    aud: String,
    azp: String,
    email: String,
    email_verified: bool,
    exp: usize,
    given_name: String,
    family_name: String,
    iat: usize,
    iss: String,
    jti: String,
    name: String,
    nbf: usize,
    picture: String,
    sub: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GooglePublicKey {
    alg: String,
    e: String,
    kid: String,
    kty: String,
    n: String,
    #[serde(rename = "use")]
    use_: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GooglePublicKeys {
    keys: Vec<GooglePublicKey>,
}

pub async fn verify_id_token(id_token: &str, env: &Config) -> Result<TokenPayload, String> {
    let public_keys_url = "https://www.googleapis.com/oauth2/v3/certs";
    let public_keys_response: reqwest::Response = reqwest::Client::new()
        .get(public_keys_url)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if !public_keys_response.status().is_success() {
        return Err("Failed to retrieve Google's public keys".to_owned());
    }

    // public key response is array of keys convert to hashmap with kid as key
    let public_keys: GooglePublicKeys = public_keys_response
        .json()
        .await
        .map_err(|err| err.to_string())?;

    let public_keys: HashMap<String, String> = public_keys.keys.into_iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, String>, key| {
            acc.insert(key.kid, key.n);
            acc
        },
    );

    dbg!(&public_keys);

    let header = jsonwebtoken::decode_header(id_token).map_err(|err| err.to_string())?;

    let kid = header.kid.ok_or("Missing 'kid' in ID token header")?;

    let public_key = public_keys[kid.as_str()].as_str();

    dbg!(&public_key);

    // let public_key = jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())
    //     .map_err(|err| err.to_string())?; // cause invalid key format error

    let public_key = jsonwebtoken::DecodingKey::from_rsa_components(public_key, "AQAB")
        .map_err(|err| err.to_string())?;

    let mut validation = Validation::new(header.alg);

    validation.set_audience(&[env.google_oauth_client_id.to_owned()]);

    dbg!(&validation);

    let token_payload = jsonwebtoken::decode::<TokenPayload>(id_token, &public_key, &validation)
        .map_err(|err| err.to_string())?;

    Ok(token_payload.claims)
}
