use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub profile: Option<String>,
    pub firstName: Option<String>,
    pub lastName: Option<String>,
    pub createdAt: Option<DateTime<Utc>>,
}
