use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Address {
    pub id: Option<Uuid>,
    pub street_address_line_1: String,
    pub street_address_line_2: Option<String>,
    pub province: String,
    pub district: String,
    pub zip_code: i64,
}

impl Address {
    pub async fn get_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Address>("SELECT id, street_address_line_1, street_address_line_2, province, district, zip_code FROM addresses WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn get_by_user_id(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as::<_, Address>("SELECT id, street_address_line_1, street_address_line_2, province, district, zip_code FROM addresses WHERE owner_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await
    }

    pub async fn create(&self, pool: &PgPool, user_id: Uuid) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Address>(
            "INSERT INTO addresses (street_address_line_1, street_address_line_2, province, district, zip_code, owner_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, street_address_line_1, street_address_line_2, province, district, zip_code",
        )
        .bind(&self.street_address_line_1)
        .bind(&self.street_address_line_2)
        .bind(&self.province)
        .bind(&self.district)
        .bind(&self.zip_code)
        .bind(user_id)
        .fetch_one(pool)
        .await
    }

    pub async fn delete_by_ids(pool: &PgPool, id: Vec<Uuid>) -> Result<(), sqlx::Error> {
        let _ = sqlx::query("DELETE FROM addresses WHERE id = ANY($1)")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
