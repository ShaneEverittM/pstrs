use async_trait::async_trait;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;

/// A paste row in our database.
#[derive(Debug, Serialize)]
pub struct Paste {
    pub id: Uuid,
    pub content: String,
}

/// Trait for interacting with the paste database.
///
/// Requires `Send + Sync` so that it can be shared between worker threads.
/// Normally you wouldn't require these in the trait definition, but for this
/// application, it wouldn't be much use to have storage that can't be shared.
///
/// Once object-safe async_fn_in_trait is stable, we can remove the async_trait.
/// See: https://rust-lang.github.io/async-fundamentals-initiative/explainer/async_fn_in_dyn_trait.html
#[async_trait]
pub trait PasteStore: Send + Sync {
    /// Get a paste by its ID.
    async fn get(&self, id: Uuid) -> Result<Option<Paste>>;

    /// Create a new paste.
    async fn create(&self, content: String) -> Result<Paste>;

    /// Remove a paste.
    async fn remove(&self, id: Uuid) -> Result<Option<Paste>>;
}

#[async_trait]
impl PasteStore for PgPool {
    async fn get(&self, id: Uuid) -> Result<Option<Paste>> {
        let paste = sqlx::query_as!(
            crate::paste::Paste,
            "SELECT id, content FROM pastes WHERE id = $1",
            id
        )
        .fetch_optional(self)
        .await?;

        Ok(paste)
    }

    async fn create(&self, content: String) -> Result<Paste> {
        let paste = sqlx::query_as!(
            crate::paste::Paste,
            "INSERT INTO pastes(content) VALUES ($1) RETURNING id, content",
            content
        )
        .fetch_one(self)
        .await?;

        Ok(paste)
    }

    async fn remove(&self, id: Uuid) -> Result<Option<Paste>> {
        let paste = sqlx::query_as!(
            crate::paste::Paste,
            "DELETE FROM pastes WHERE id = $1 RETURNING id, content",
            id
        )
        .fetch_optional(self)
        .await?;

        Ok(paste)
    }
}
