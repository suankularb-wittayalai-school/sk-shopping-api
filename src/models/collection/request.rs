use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableCollection {
    pub id: Option<sqlx::types::Uuid>,
    pub shop_ids: Option<Vec<sqlx::types::Uuid>>,
    pub listing_ids: Option<Vec<sqlx::types::Uuid>>,
    pub item_ids: Option<Vec<sqlx::types::Uuid>>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableCollection {
    Id,
    Name,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatableCollection {
    pub shop_id: sqlx::types::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
}

impl CreatableCollection {
    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
        let res = sqlx::query(
            r#"
            INSERT INTO collections (shop_id, name, description, foreground_url, background_url)
            VALUES ($1, $2, $3, $4, $5)
            returning id
            "#,
        )
        .bind(&self.shop_id)
        .bind(&self.name)
        .bind(&self.description)
        .bind(&self.foreground_url)
        .bind(&self.background_url)
        .fetch_one(pool)
        .await?;

        Ok(res.get("id"))
    }

    pub async fn bulk_insert(
        collections: Vec<CreatableCollection>,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let mut ids = Vec::new();

        for collection in collections {
            let res = sqlx::query(
                r#"
                INSERT INTO collections (shop_id, name, description, foreground_url, background_url)
                VALUES ($1, $2, $3, $4, $5)
                returning id
                "#,
            )
            .bind(&collection.shop_id)
            .bind(&collection.name)
            .bind(&collection.description.unwrap_or("".to_string()))
            .bind(&collection.foreground_url)
            .bind(&collection.background_url)
            .fetch_one(transaction.as_mut())
            .await?;

            ids.push(res.get("id"));
        }

        transaction.commit().await?;

        Ok(ids)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatableCollection {
    // pub shop_id: sqlx::types::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub foreground_url: Option<String>,
    pub background_url: Option<String>,
}

impl UpdatableCollection {
    pub async fn commit_changes(
        &self,
        pool: &sqlx::PgPool,
        collection_id: sqlx::types::Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut query = String::from("UPDATE collections SET ");
        let mut param_count = 1;

        let mut param_segments = Vec::new();
        let mut string_params = Vec::new();

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

        if let Some(foreground_url) = &self.foreground_url {
            param_segments.push(format!("foreground_url = ${}", param_count));
            string_params.push(foreground_url);
            param_count += 1;
        }

        if let Some(background_url) = &self.background_url {
            param_segments.push(format!("background_url = ${}", param_count));
            string_params.push(background_url);
            param_count += 1;
        }

        query.push_str(&param_segments.join(", "));

        query.push_str(" WHERE id = $");

        query.push_str(&param_count.to_string());

        let mut query_builder = sqlx::query(&query);

        for param in string_params {
            query_builder = query_builder.bind(param);
        }

        query_builder = query_builder.bind(collection_id);

        query_builder.execute(pool).await?;

        Ok(())
    }
}
