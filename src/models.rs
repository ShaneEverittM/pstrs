use serde::Serialize;
use uuid::Uuid;

/// A paste row in our database.
#[derive(Serialize)]
pub struct Paste {
    pub id: Uuid,
    pub content: String,
}
