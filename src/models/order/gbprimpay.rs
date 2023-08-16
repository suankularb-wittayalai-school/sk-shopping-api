use base64::{engine::general_purpose, prelude::*};
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GbPrimePayQRRequest {
    token: String,
    amount: i64,
    reference_no: String,
    background_url: String,
    detail: Option<String>,
    customer_name: Option<String>,
    customer_email: Option<String>,
    customer_telephone: Option<String>,
    customer_address: Option<String>,
}

impl GbPrimePayQRRequest {
    fn new(
        token: String,
        amount: i64,
        reference_no: String,
        detail: Option<String>,
        customer_name: Option<String>,
        customer_email: Option<String>,
        customer_telephone: Option<String>,
        customer_address: Option<String>,
    ) -> Self {
        Self {
            token,
            amount,
            reference_no,
            background_url: "https://api.shopping.skkornor.org/order_webhook".to_string(),
            detail,
            customer_name,
            customer_email,
            customer_telephone,
            customer_address,
        }
    }
}

// fetch API from gbprimepay as application/x-www-form-urlencoded and return as image/png
// return the image/png as base64
pub async fn get_qrcode(
    token: String,
    amount: i64,
    reference_no: String,
    detail: Option<String>,
    customer_name: Option<String>,
    customer_email: Option<String>,
    customer_telephone: Option<String>,
    customer_address: Option<String>,
) -> Result<String, reqwest::Error> {
    let request = GbPrimePayQRRequest::new(
        token,
        amount,
        reference_no,
        detail,
        customer_name,
        customer_email,
        customer_telephone,
        customer_address,
    );

    let data = serde_urlencoded::to_string(&request).expect("serialize issue");

    let res = Client::new()
        .post("https://api.gbprimepay.com/v3/qrcode")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(data)
        .send()
        .await?;

    // let encoded = base64::encode(res.bytes().await?);
    let encoded = general_purpose::STANDARD.encode(res.bytes().await?);

    Ok(encoded)
}
