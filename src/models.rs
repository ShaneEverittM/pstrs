use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Paste {
    pub id: Uuid,
    pub content: String,
}
