use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::Result, models::Paste};

/// Trait for interacting with the database.
///
/// Once object-safe async_fn_in_trait is stable, we can remove the async_trait.
/// See: https://rust-lang.github.io/async-fundamentals-initiative/explainer/async_fn_in_dyn_trait.html
#[async_trait]
pub trait PasteDatabase {
    async fn get_paste(&self, id: Uuid) -> Result<Paste>;
    async fn create_paste(&self, content: String) -> Result<Paste>;
}

#[async_trait]
impl PasteDatabase for PgPool {
    async fn get_paste(&self, id: Uuid) -> Result<Paste> {
        let paste = sqlx::query_as!(
            crate::models::Paste,
            "SELECT id, content FROM pastes WHERE id = $1",
            id
        )
        .fetch_one(self)
        .await?;

        Ok(paste)
    }

    async fn create_paste(&self, content: String) -> Result<Paste> {
        let paste = sqlx::query_as!(
            crate::models::Paste,
            "INSERT INTO pastes(content) VALUES ($1) RETURNING id, content",
            content
        )
        .fetch_one(self)
        .await?;

        Ok(paste)
    }
}
