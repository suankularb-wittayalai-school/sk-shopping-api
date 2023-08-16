use base64::{engine::general_purpose, prelude::*};
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GbPrimePayQRRequest {
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

#[derive(Debug, Deserialize, Serialize)]
pub enum GBPRetryFlag {
    #[serde(rename = "Y")]
    Retry,
    #[serde(rename = "N")]
    FirstTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ResultCode {
    #[serde(rename = "00")]
    Success,
    #[serde(rename = "11")]
    InvalidReferenceNo,
    #[serde(rename = "12")]
    InvalidGBReferenceNo,
    #[serde(rename = "14")]
    InvalidAmount,
    #[serde(rename = "21")]
    DuplicateTransaction,
    #[serde(rename = "22")]
    OverDue,
    #[serde(rename = "99")]
    SystemError,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GbPrimePayWebHookRequest {
    pub amount: i64,
    pub reference_no: String,
    pub gbp_reference_no: String,
    pub result_code: ResultCode,
    pub date: Option<String>, // DDMMYYYY
    pub time: Option<String>, // HHMMSS
    pub currency_code: Option<String>,
    pub detail: Option<String>,
    pub customer_name: Option<String>,
    pub customer_email: Option<String>,
    pub customer_telephone: Option<String>,
    pub customer_address: Option<String>,
    pub retry_flag: Option<GBPRetryFlag>,
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
            background_url: "https://api.shopping.skkornor.org/order-webhook".to_string(),
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
pub async fn create_qr_code(
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
