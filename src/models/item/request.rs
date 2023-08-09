use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::models::common::RangeQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryableItem {
    pub id: Option<sqlx::types::Uuid>,
    pub shop_ids: Option<Vec<sqlx::types::Uuid>>,
    pub collection_ids: Option<Vec<sqlx::types::Uuid>>,
    pub listing_ids: Option<Vec<sqlx::types::Uuid>>,
    pub name: Option<String>,
    pub stock_range: Option<RangeQuery>,
    pub price_range: Option<RangeQuery>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortableItem {
    Id,
    Name,
    CreatedAt,
    // Stock,
    Price,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatableItem {
    pub name: String,
    pub shop_id: Option<sqlx::types::Uuid>,
    pub price: i64,
    pub discounted_price: Option<i64>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    pub variant_name: Option<String>,
    pub colors: Option<Vec<String>>,
    // if images_url is not None, then it will be added to item_images and first image will be used as listing thumbnail
    pub images_url: Option<Vec<String>>,
    // if initial_stock is not None, then it will be added to item_stock_updates
    pub initial_stock: Option<i64>,
    // if listing_id is None, then one listing will be created
    pub listing_id: Option<sqlx::types::Uuid>,
    // description is optional and only used to create listing
    pub description: Option<String>,
}

impl CreatableItem {
    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
        // check listing id
        let listing_id = match self.listing_id {
            Some(listing_id) => listing_id,
            None => {
                let thumbnail_url = match &self.images_url {
                    Some(images_url) => {
                        if images_url.len() > 0 {
                            Some(images_url[0].clone())
                        } else {
                            None
                        }
                    }
                    None => None,
                };

                let listing_id = sqlx::query(
                    r#"
                    INSERT INTO listings (shop_id, description, name, thumbnail_url)
                    VALUES ($1, $2)
                    returning id
                    "#,
                )
                .bind(&self.shop_id)
                .bind(&self.description)
                .bind(&self.name)
                .bind(&thumbnail_url)
                .fetch_one(pool)
                .await?;

                listing_id.get("id")
            }
        };

        // insert item
        let item_id = sqlx::query(
            r#"
            INSERT INTO items (name, listing_id, price, discounted_price, preorder_start, preorder_end, variant_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            returning id
            "#,
        )
        .bind(&self.name)
        .bind(&listing_id)
        .bind(&self.price)
        .bind(&self.discounted_price)
        .bind(&self.preorder_start)
        .bind(&self.preorder_end)
        .bind(&self.variant_name)
        .fetch_one(pool)
        .await?;

        let item_id = item_id.get::<Uuid, _>("id");

        // insert item images
        match &self.images_url {
            Some(images_url) => {
                for image_url in images_url {
                    sqlx::query(
                        r#"
                        INSERT INTO item_images (item_id, image_url)
                        VALUES ($1, $2)
                        "#,
                    )
                    .bind(&item_id)
                    .bind(&image_url)
                    .execute(pool)
                    .await?;
                }
            }
            None => {}
        };

        // insert item stock updates
        match self.initial_stock {
            Some(initial_stock) => {
                sqlx::query(
                    r#"
                    INSERT INTO item_stock_updates (item_id, stock_added)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(&item_id)
                .bind(&initial_stock)
                .execute(pool)
                .await?;
            }
            None => {}
        };

        // insert item colors
        match &self.colors {
            Some(colors) => {
                for color in colors {
                    sqlx::query(
                        r#"
                        INSERT INTO item_colors (item_id, color)
                        VALUES ($1, $2)
                        "#,
                    )
                    .bind(&item_id)
                    .bind(&color)
                    .execute(pool)
                    .await?;
                }
            }
            None => {}
        };

        Ok(item_id)
    }

    pub async fn bulk_insert(
        data: Vec<CreatableItem>,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let mut transaction = pool.begin().await?;

        let mut item_ids = Vec::new();

        for item in data {
            let listing_id = match item.listing_id {
                Some(listing_id) => listing_id,
                None => {
                    let thumbnail_url = match &item.images_url {
                        Some(images_url) => {
                            if images_url.len() > 0 {
                                Some(images_url[0].clone())
                            } else {
                                None
                            }
                        }
                        None => None,
                    };

                    let listing_id = sqlx::query(
                        r#"
                        INSERT INTO listings (shop_id, description, name, thumbnail_url)
                        VALUES ($1, $2, $3, $4)
                        returning id
                        "#,
                    )
                    .bind(&item.shop_id)
                    .bind(&item.description.unwrap_or_default())
                    .bind(&item.name)
                    .bind(&thumbnail_url)
                    .fetch_one(transaction.as_mut())
                    .await?;

                    listing_id.get("id")
                }
            };

            // insert item
            let item_id = sqlx::query(
                r#"
                INSERT INTO items (name, listing_id, price, discounted_price, preorder_start, preorder_end, variant_name)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                returning id
                "#,
            )
            .bind(&item.name)
            .bind(&listing_id)
            .bind(&item.price)
            .bind(&item.discounted_price)
            .bind(&item.preorder_start)
            .bind(&item.preorder_end)
            .bind(&item.variant_name)
            .fetch_one(transaction.as_mut())
            .await?;

            let item_id = item_id.get::<Uuid, _>("id");

            // insert item images
            match &item.images_url {
                Some(images_url) => {
                    for image_url in images_url {
                        sqlx::query(
                            r#"
                            INSERT INTO item_images (item_id, image_url)
                            VALUES ($1, $2)
                            "#,
                        )
                        .bind(&item_id)
                        .bind(&image_url)
                        .execute(transaction.as_mut())
                        .await?;
                    }
                }
                None => {}
            };

            // insert item stock updates
            match item.initial_stock {
                Some(initial_stock) => {
                    sqlx::query(
                        r#"
                        INSERT INTO item_stock_updates (item_id, stock_added)
                        VALUES ($1, $2)
                        "#,
                    )
                    .bind(&item_id)
                    .bind(&initial_stock)
                    .execute(transaction.as_mut())
                    .await?;
                }
                None => {}
            };

            // insert item colors
            match &item.colors {
                Some(colors) => {
                    for color in colors {
                        sqlx::query(
                            r#"
                            INSERT INTO item_colors (item_id, color)
                            VALUES ($1, $2)
                            "#,
                        )
                        .bind(&item_id)
                        .bind(&color)
                        .execute(transaction.as_mut())
                        .await?;
                    }
                }
                None => {}
            };

            item_ids.push(item_id);
        }

        transaction.commit().await?;

        Ok(item_ids)
    }
}
