use serde::{Deserialize, Serialize};

use crate::models::common::RangeQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableListing {
    pub name: Option<String>,
    pub description: Option<String>,
    pub id: Option<sqlx::types::Uuid>,
    pub shop_ids: Option<Vec<sqlx::types::Uuid>>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub category_ids: Option<Vec<sqlx::types::Uuid>>,
    pub item_ids: Option<Vec<sqlx::types::Uuid>>,
    pub is_hidden: Option<bool>,
    pub price_range: Option<RangeQuery>,
    pub stock_range: Option<RangeQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableListing {
    Id,
    Name,
    CreatedAt,
    Stock,
    Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatableListing {
    // pub shop_id: sqlx::types::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_hidden: Option<bool>,
}

impl UpdatableListing {
    pub async fn commit_changes(
        &self,
        pool: &sqlx::PgPool,
        listing_id: sqlx::types::Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut query = String::from("UPDATE listings SET ");
        let mut param_count = 1;

        let mut param_segments = Vec::new();
        let mut string_params = Vec::new();
        let mut bool_params = Vec::new();

        if let Some(name) = &self.name {
            param_segments.push(format!("name = ${}", param_count));
            string_params.push(name);
            param_count += 1;
        }

        if let Some(description) = &self.description {
            param_segments.push(format!("description = ${}", param_count));
            string_params.push(description);
            param_count += 1;
        }

        if let Some(thumbnail_url) = &self.thumbnail_url {
            param_segments.push(format!("thumbnail_url = ${}", param_count));
            string_params.push(thumbnail_url);
            param_count += 1;
        }

        if let Some(is_hidden) = &self.is_hidden {
            param_segments.push(format!("is_hidden = ${}", param_count));
            bool_params.push(is_hidden);
            param_count += 1;
        }

        query.push_str(&param_segments.join(", "));

        query.push_str(" WHERE id = $");

        query.push_str(&param_count.to_string());

        let mut query_builder = sqlx::query(&query);

        for param in string_params {
            query_builder = query_builder.bind(param);
        }

        for param in bool_params {
            query_builder = query_builder.bind(param);
        }

        query_builder = query_builder.bind(listing_id);

        query_builder.execute(pool).await?;

        Ok(())
    }
}
