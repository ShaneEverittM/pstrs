use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TodoNew {
    pub note: String,
}

#[derive(Serialize)]
pub struct Todo {
    pub id: i32,
    pub note: String,
}
