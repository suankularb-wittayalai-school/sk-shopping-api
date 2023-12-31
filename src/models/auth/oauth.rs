use crate::utils::common::config::Config;
use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
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
}

impl GoogleUserResult {
    pub fn from_token_payload(payload: TokenPayload) -> Self {
        Self {
            id: payload.sub,
            email: payload.email,
            verified_email: payload.email_verified,
            name: payload.name,
            given_name: payload.given_name,
            family_name: payload.family_name,
            picture: payload.picture,
        }
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

    // dbg!(&public_keys);

    let header = jsonwebtoken::decode_header(id_token).map_err(|err| err.to_string())?;

    let kid = header.kid.ok_or("Missing 'kid' in ID token header")?;

    let public_key = public_keys[kid.as_str()].as_str();

    // dbg!(&public_key);

    // let public_key = jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())
    //     .map_err(|err| err.to_string())?; // cause invalid key format error

    let public_key = jsonwebtoken::DecodingKey::from_rsa_components(public_key, "AQAB")
        .map_err(|err| err.to_string())?;

    let mut validation = Validation::new(header.alg);

    validation.set_audience(&[env.google_oauth_client_id.to_owned()]);
    validation.iss = Some(HashSet::from(["https://accounts.google.com".to_owned()]));

    dbg!(&validation);

    let token_payload = jsonwebtoken::decode::<TokenPayload>(id_token, &public_key, &validation)
        .map_err(|err| err.to_string())?;

    Ok(token_payload.claims)
}
