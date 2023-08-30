use std::sync::Arc;

use sqlx::PgPool;

use crate::db::PasteDatabase;

/// Application state.
///
/// This is accessible to all handlers via `State<App>`. It should be
/// thread-safe (`Send + Sync`) and be _shared_ state. A database connection
/// pool here so that each handler can access the database without needing to
/// create a new connection each time (which is expensive). The `#[Postgres]`
/// attribute on an argument to main will have shuttle provision and connect to
/// a Postgres database for you.
#[derive(Clone)]
pub struct App {
    pub db: Arc<dyn PasteDatabase + Send + Sync>,
}

impl App {
    // Construct application state with a postgres connection pool.
    pub fn postgres(pool: PgPool) -> Self { Self { db: Arc::new(pool) } }
}
