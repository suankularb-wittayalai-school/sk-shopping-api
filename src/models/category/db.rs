use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoryTable {
    pub id: sqlx::types::Uuid,
    pub created_at: Option<DateTime<Utc>>,
    pub name_th: String,
    pub name_en: String,
}

impl CategoryTable {
    pub async fn get_all(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let query = sqlx::query_as::<_, Self>(
            r#"
            SELECT id, created_at, name_th, name_en
            FROM categories
            "#,
        )
        .fetch_all(pool)
        .await;

        query
    }
}
