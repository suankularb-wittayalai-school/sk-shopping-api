use base64::{engine::general_purpose, prelude::*};
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use super::Order;

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

#[derive(Debug, Serialize, PartialEq)]
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
    Unknown,
}

impl<'de> Deserialize<'de> for ResultCode {
    fn deserialize<D>(deserializer: D) -> Result<ResultCode, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "00" => Ok(ResultCode::Success),
            "11" => Ok(ResultCode::InvalidReferenceNo),
            "12" => Ok(ResultCode::InvalidGBReferenceNo),
            "14" => Ok(ResultCode::InvalidAmount),
            "21" => Ok(ResultCode::DuplicateTransaction),
            "22" => Ok(ResultCode::OverDue),
            "99" => Ok(ResultCode::SystemError),
            _ => Ok(ResultCode::Unknown),
        }
    }
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
    // fn new(
    //     token: String,
    //     amount: i64,
    //     reference_no: String,
    //     detail: Option<String>,
    //     customer_name: Option<String>,
    //     customer_email: Option<String>,
    //     customer_telephone: Option<String>,
    //     customer_address: Option<String>,
    // )

    fn new(token: String, order: Order) -> Self {
        let (
            amount,
            reference_no,
            detail,
            customer_name,
            customer_email,
            customer_telephone,
            customer_address,
        ) = match order {
            Order::Default(order) => (
                order.total_price,
                order.ref_id,
                None,
                order.receiver_name,
                order.contact_email,
                order.contact_phone_number,
                format!(
                    "{} {} {} {} {}",
                    order.street_address_line_1.unwrap_or_default(),
                    order.street_address_line_2.unwrap_or_default(),
                    order.district.unwrap_or_default(),
                    order.province.unwrap_or_default(),
                    order.zip_code.unwrap_or_default()
                ),
            ),
            Order::Detailed(order) => (
                order.total_price,
                order.ref_id,
                None,
                order.receiver_name,
                order.contact_email,
                order.contact_phone_number,
                format!(
                    "{} {} {} {} {}",
                    order.street_address_line_1.unwrap_or_default(),
                    order.street_address_line_2.unwrap_or_default(),
                    order.district.unwrap_or_default(),
                    order.province.unwrap_or_default(),
                    order.zip_code.unwrap_or_default()
                ),
            ),
            _ => panic!("Order type not supported"),
        };

        Self {
            token,
            amount,
            reference_no,
            background_url: "https://api.shopping.skkornor.org/orders/webhook".to_string(),
            detail,
            customer_name: Some(customer_name),
            customer_email: Some(customer_email),
            customer_telephone,
            customer_address: Some(customer_address),
        }
    }
}

// fetch API from gbprimepay as application/x-www-form-urlencoded and return as image/png
// return the image/png as base64
pub async fn create_qr_code(token: String, order: Order) -> Result<String, reqwest::Error> {
    let request = GbPrimePayQRRequest::new(token, order);

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

impl GbPrimePayWebHookRequest {
    pub async fn update_order_status(&self, pool: &sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
        let order_id = sqlx::query(
            r#"
            UPDATE orders
            SET is_paid = true, is_verified = true
            WHERE ref_id = $2
            RETURNING id
            "#,
        )
        .bind(self.reference_no.clone())
        .fetch_one(pool)
        .await?
        .get::<Uuid, _>("id");

        Ok(order_id)
    }
}
