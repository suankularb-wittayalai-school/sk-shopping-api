use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::db::PaymentMethod;

// {
//     "object": "charge",
//     "id": "chrg_test_5sqhgjedme2u2atfhuc",
//     "location": "/charges/chrg_test_5sqhgjedme2u2atfhuc",
//     "amount": 400000,
//     "net": 392938,
//     "fee": 6600,
//     "fee_vat": 462,
//     "interest": 0,
//     "interest_vat": 0,
//     "funding_amount": 400000,
//     "refunded_amount": 0,
//     "transaction_fees": {
//       "fee_flat": "0.0",
//       "fee_rate": "1.65",
//       "vat_rate": "7.0"
//     },
//     "platform_fee": {
//       "fixed": null,
//       "amount": null,
//       "percentage": null
//     },
//     "currency": "THB",
//     "funding_currency": "THB",
//     "ip": null,
//     "refunds": {
//       "object": "list",
//       "data": [],
//       "limit": 20,
//       "offset": 0,
//       "total": 0,
//       "location": "/charges/chrg_test_5sqhgjedme2u2atfhuc/refunds",
//       "order": "chronological",
//       "from": "1970-01-01T00:00:00Z",
//       "to": "2022-08-08T03:20:44Z"
//     },
//     "link": null,
//     "description": null,
//     "metadata": {},
//     "card": null,
//     "source": {
//       "object": "source",
//       "id": "src_test_5sqhgizua2ugu18pszz",
//       "livemode": false,
//       "location": "/sources/src_test_5sqhgizua2ugu18pszz",
//       "amount": 400000,
//       "barcode": null,
//       "bank": null,
//       "created_at": "2022-08-08T03:20:42Z",
//       "currency": "THB",
//       "email": null,
//       "flow": "offline",
//       "installment_term": null,
//       "absorption_type": null,
//       "name": null,
//       "mobile_number": null,
//       "phone_number": null,
//       "platform_type": null,
//       "scannable_code": {
//         "object": "barcode",
//         "type": "qr",
//         "image": {
//           "object": "document",
//           "livemode": false,
//           "id": "docu_test_5sqhgjg71l2hm8dxra4",
//           "deleted": false,
//           "filename": "qrcode_test.svg",
//           "location": "/charges/chrg_test_5sqhgjedme2u2atfhuc/documents/docu_test_5sqhgjg71l2hm8dxra4",
//           "kind": "qr",
//           "download_uri": "https://api.omise.co/charges/chrg_test_5sqhgjedme2u2atfhuc/documents/docu_test_5sqhgjg71l2hm8dxra4/downloads/167C00410B655C0D",
//           "created_at": "2022-08-08T03:20:44Z"
//         }
//       },
//       "references": null,
//       "store_id": null,
//       "store_name": null,
//       "terminal_id": null,
//       "type": "promptpay",
//       "zero_interest_installments": null,
//       "charge_status": "pending",
//       "receipt_amount": null,
//       "discounts": []
//     },
//     "schedule": null,
//     "customer": null,
//     "dispute": null,
//     "transaction": null,
//     "failure_code": null,
//     "failure_message": null,
//     "status": "pending",
//     "authorize_uri": null,
//     "return_uri": null,
//     "created_at": "2022-08-08T03:20:44Z",
//     "paid_at": null,
//     "expires_at": "2022-08-09T03:20:44Z",
//     "expired_at": null,
//     "reversed_at": null,
//     "zero_interest_installments": false,
//     "branch": null,
//     "terminal": null,
//     "device": null,
//     "authorized": false,
//     "capturable": false,
//     "capture": true,
//     "disputable": false,
//     "livemode": false,
//     "refundable": false,
//     "reversed": false,
//     "reversible": false,
//     "voided": false,
//     "paid": false,
//     "expired": false
//   }

#[derive(Debug, Deserialize, Serialize)]
pub struct OmiseCharge {
    pub object: String,
    pub id: String,
    pub location: String,
    pub amount: i64,
    pub net: i64,
    pub fee: i64,
    pub fee_vat: i64,
    pub interest: i64,
    pub interest_vat: i64,
    pub funding_amount: i64,
    pub refunded_amount: i64,
    pub transaction_fees: TransactionFees,
    pub platform_fee: PlatformFee,
    pub currency: String,
    pub funding_currency: String,
    pub ip: String,
    pub refunds: Refunds,
    pub link: Option<String>,
    pub description: Option<String>,
    pub metadata: Metadata,
    pub card: Option<String>,
    pub source: Source,
    pub schedule: String,
    pub customer: String,
    pub dispute: String,
    pub transaction: String,
    pub failure_code: String,
    pub failure_message: String,
    pub status: String,
    pub authorize_uri: String,
    pub return_uri: String,
    pub created_at: String,
    pub paid_at: String,
    pub expires_at: String,
    pub expired_at: Option<String>,
    pub reversed_at: Option<String>,
    pub zero_interest_installments: bool,
    pub branch: Option<String>,
    pub terminal: Option<String>,
    pub device: Option<String>,
    pub authorized: bool,
    pub capturable: bool,
    pub capture: bool,
    pub disputable: bool,
    pub livemode: bool,
    pub refundable: bool,
    pub reversed: bool,
    pub reversible: bool,
    pub voided: bool,
    pub paid: bool,
    pub expired: bool,
}

impl OmiseCharge {
    pub async fn new(
        amount: i64,
        // currency: &str,
        source_type: PaymentMethod,
        omise_secret_key: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut map = HashMap::new();
        map.insert("amount", (amount * 100).to_string());
        map.insert("currency", "THB".to_string());
        map.insert("source[type]", source_type.to_string());

        let res = client
            .post("https://api.omise.co/charges")
            .basic_auth(omise_secret_key, Some(""))
            .form(&map)
            .send()
            .await?
            .json::<OmiseCharge>()
            .await?;

        Ok(res)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionFees {
    pub fixed: Option<i64>,
    pub amount: Option<i64>,
    pub percentage: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformFee {
    pub fixed: Option<i64>,
    pub amount: Option<i64>,
    pub percentage: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Refunds {
    pub object: String,
    pub data: Vec<String>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
    pub location: String,
    pub order: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Source {
    pub object: String,
    pub id: String,
    pub livemode: bool,
    pub location: String,
    pub amount: i64,
    pub barcode: Option<String>,
    pub bank: Option<String>,
    pub created_at: String,
    pub currency: String,
    pub email: Option<String>,
    pub flow: String,
    pub installment_term: String,
    pub absorption_type: String,
    pub name: Option<String>,
    pub mobile_number: String,
    pub phone_number: String,
    pub platform_type: String,
    pub scannable_code: ScannableCode,
    pub references: Option<String>,
    pub store_id: Option<String>,
    pub store_name: Option<String>,
    pub terminal_id: Option<String>,
    #[serde(rename = "type")]
    pub r#type: String,
    pub zero_interest_installments: Option<bool>,
    pub charge_status: Option<String>,
    pub receipt_amount: String,
    pub discounts: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScannableCode {
    pub object: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub image: Image,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    pub object: String,
    pub livemode: bool,
    pub id: String,
    pub deleted: bool,
    pub filename: String,
    pub location: String,
    pub kind: String,
    pub download_uri: String,
    pub created_at: String,
}
