use mysk_lib::models::common::string::FlexibleMultiLangString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableShop {
    pub id: Option<sqlx::types::Uuid>,
    pub name: Option<String>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub listing_ids: Option<Vec<sqlx::types::Uuid>>,
    pub item_ids: Option<Vec<sqlx::types::Uuid>>,
    pub manager_ids: Option<Vec<sqlx::types::Uuid>>,
    pub accept_promptpay: Option<bool>,
    pub accept_cod: Option<bool>,
    pub is_school_pickup_allowed: Option<bool>,
    pub is_delivery_allowed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableShop {
    Id,
    NameTh,
    NameEn,
    CreatedAt,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdatableShop {
    pub name: Option<FlexibleMultiLangString>,
    pub pickup_location: Option<String>,
    pub pickup_description: Option<String>,
    pub accent_color: Option<String>,
    pub background_color: Option<String>,
    pub logo_url: Option<String>,
    pub accept_promptpay: Option<bool>,
    pub accept_cod: Option<bool>,
    pub is_school_pickup_allowed: Option<bool>,
    pub is_delivery_allowed: Option<bool>,
}

impl UpdatableShop {
    pub async fn commit_changes(
        &self,
        pool: &sqlx::PgPool,
        shop_id: sqlx::types::Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut query = String::from("UPDATE shops SET ");
        let mut param_count = 1;

        let mut param_segments = Vec::new();
        let mut string_params = Vec::new();
        let mut bool_params = Vec::new();

        if let Some(name) = &self.name {
            // param_segments.push(format!("name = ${}", param_count));
            // string_params.push(name);
            // param_count += 1;

            if let Some(name_th) = &name.th {
                param_segments.push(format!("name_th = ${}", param_count));
                string_params.push(name_th);
                param_count += 1;
            }

            if let Some(name_en) = &name.en {
                param_segments.push(format!("name_en = ${}", param_count));
                string_params.push(name_en);
                param_count += 1;
            }
        }

        if let Some(pickup_location) = &self.pickup_location {
            param_segments.push(format!("pickup_location = ${}", param_count));
            string_params.push(pickup_location);
            param_count += 1;
        }

        if let Some(pickup_description) = &self.pickup_description {
            param_segments.push(format!("pickup_description = ${}", param_count));
            string_params.push(pickup_description);
            param_count += 1;
        }

        if let Some(accent_color) = &self.accent_color {
            param_segments.push(format!("accent_color = ${}", param_count));
            string_params.push(accent_color);
            param_count += 1;
        }

        if let Some(background_color) = &self.background_color {
            param_segments.push(format!("background_color = ${}", param_count));
            string_params.push(background_color);
            param_count += 1;
        }

        if let Some(logo_url) = &self.logo_url {
            param_segments.push(format!("logo_url = ${}", param_count));
            string_params.push(logo_url);
            param_count += 1;
        }

        if let Some(accept_promptpay) = &self.accept_promptpay {
            param_segments.push(format!("accept_promptpay = ${}", param_count));
            bool_params.push(accept_promptpay);
            param_count += 1;
        }

        if let Some(accept_cod) = &self.accept_cod {
            param_segments.push(format!("accept_cod = ${}", param_count));
            bool_params.push(accept_cod);
            param_count += 1;
        }

        if let Some(is_school_pickup_allowed) = &self.is_school_pickup_allowed {
            param_segments.push(format!("is_school_pickup_allowed = ${}", param_count));
            bool_params.push(is_school_pickup_allowed);
            param_count += 1;
        }

        if let Some(is_delivery_allowed) = &self.is_delivery_allowed {
            param_segments.push(format!("is_delivery_allowed = ${}", param_count));
            bool_params.push(is_delivery_allowed);
            param_count += 1;
        }

        query.push_str(&param_segments.join(", "));
        query.push_str(&format!(" WHERE id = ${}", param_count));

        // dbg!(&query);

        let mut query_builder = sqlx::query(&query);

        for param in string_params {
            query_builder = query_builder.bind(param);
        }

        for param in bool_params {
            query_builder = query_builder.bind(param);
        }

        query_builder = query_builder.bind(shop_id);

        query_builder.execute(pool).await?;

        Ok(())
    }
}
