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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatableItem {
    pub name: Option<String>,
    pub variant_name: Option<String>,
    pub price: Option<i64>,
    pub discounted_price: Option<i64>,
    pub preorder_start: Option<DateTime<Utc>>,
    pub preorder_end: Option<DateTime<Utc>>,
    // will delete all existing colors and replace with new ones
    pub colors: Option<Vec<String>>,
    // will delete all existing images and replace with new ones
    pub images_url: Option<Vec<String>>,
}

impl UpdatableItem {
    pub async fn commit_changes(
        &self,
        pool: &sqlx::PgPool,
        item_id: sqlx::types::Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut query = String::from("UPDATE items SET ");
        let mut param_count = 1;

        let mut param_segments = Vec::new();
        let mut string_params = Vec::new();
        let mut int_params = Vec::new();
        let mut datetime_params = Vec::new();

        if let Some(name) = &self.name {
            param_segments.push(format!("name = ${}", param_count));
            string_params.push(name);
            param_count += 1;
        }

        if let Some(variant_name) = &self.variant_name {
            param_segments.push(format!("variant_name = ${}", param_count));
            string_params.push(variant_name);
            param_count += 1;
        }

        if let Some(price) = &self.price {
            param_segments.push(format!("price = ${}", param_count));
            int_params.push(price);
            param_count += 1;
        }

        if let Some(discounted_price) = &self.discounted_price {
            param_segments.push(format!("discounted_price = ${}", param_count));
            int_params.push(discounted_price);
            param_count += 1;
        }

        if let Some(preorder_start) = &self.preorder_start {
            param_segments.push(format!("preorder_start = ${}", param_count));
            datetime_params.push(preorder_start);
            param_count += 1;
        }

        if let Some(preorder_end) = &self.preorder_end {
            param_segments.push(format!("preorder_end = ${}", param_count));
            datetime_params.push(preorder_end);
            param_count += 1;
        }

        query.push_str(&param_segments.join(", "));

        query.push_str(" WHERE id = $");

        query.push_str(&param_count.to_string());

        // dbg!(query.as_str());

        let mut transaction = pool.begin().await?;

        let mut query_builder = sqlx::query(&query);

        for param in string_params.clone() {
            query_builder = query_builder.bind(param);
        }

        for param in int_params.clone() {
            query_builder = query_builder.bind(param);
        }

        for param in datetime_params.clone() {
            query_builder = query_builder.bind(param);
        }

        query_builder = query_builder.bind(item_id);

        if (string_params.len() + int_params.len() + datetime_params.len()) != 0 {
            query_builder.execute(transaction.as_mut()).await?;
        }

        // update item images
        match &self.images_url {
            Some(images_url) => {
                sqlx::query(
                    r#"
                    DELETE FROM item_images WHERE item_id = $1
                    "#,
                )
                .bind(&item_id)
                .execute(transaction.as_mut())
                .await?;

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

        // update item colors
        match &self.colors {
            Some(colors) => {
                sqlx::query(
                    r#"
                    DELETE FROM item_colors WHERE item_id = $1
                    "#,
                )
                .bind(&item_id)
                .execute(transaction.as_mut())
                .await?;

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

        transaction.commit().await?;

        Ok(())
    }
}

impl CreatableItem {
    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<Uuid, sqlx::Error> {
        let mut transaction = pool.begin().await?;

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
                    .execute(transaction.as_mut())
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
                .execute(transaction.as_mut())
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
                    .execute(transaction.as_mut())
                    .await?;
                }
            }
            None => {}
        };

        transaction.commit().await?;

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
