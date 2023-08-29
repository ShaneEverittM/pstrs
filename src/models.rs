use serde::Serialize;

#[derive(Serialize)]
pub struct Paste {
    pub id: i32,
    pub content: String,
}
