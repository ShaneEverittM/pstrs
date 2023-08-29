use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct App {
    pub db: Pool<Postgres>,
}
