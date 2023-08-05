use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IdOnlyOrder {
    pub id: sqlx::types::Uuid,
}

impl From<super::super::db::OrderTable> for IdOnlyOrder {
    fn from(order: super::super::db::OrderTable) -> Self {
        Self { id: order.id }
    }
}
