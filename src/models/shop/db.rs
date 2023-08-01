use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Default)]
pub struct ShopTable {
    pub id: String,
    pub created_at: Option<NaiveDateTime>,
    pub name_th: String,
    pub name_en: Option<String>,
    pub logo_url: String,
    pub is_school_pickup_allowed: bool,
    pub pickup_location: Option<String>,
    pub is_delivery_allowed: bool,
    pub accept_promptpay: bool,
    pub promptpay_number: Option<String>,
    pub accept_cod: bool,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
}
