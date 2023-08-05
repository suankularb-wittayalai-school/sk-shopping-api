use mysk_lib::models::common::string::MultiLangString;
use serde::{Deserialize, Serialize};

pub(crate) mod db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: uuid::Uuid,
    pub name: MultiLangString,
}

impl From<db::CategoryTable> for Category {
    fn from(category_table: db::CategoryTable) -> Self {
        Self {
            id: category_table.id,
            name: MultiLangString {
                th: category_table.name_th,
                en: Some(category_table.name_en),
            },
        }
    }
}

impl Category {
    pub async fn get_all(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let categories = db::CategoryTable::get_all(pool).await?;

        let categories: Vec<Self> = categories.into_iter().map(|c| c.into()).collect();

        Ok(categories)
    }
}
