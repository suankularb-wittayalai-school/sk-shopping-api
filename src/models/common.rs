use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeQuery {
    pub min: i64,
    pub max: i64,
}
