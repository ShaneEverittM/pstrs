use axum::{
    extract::{Host, Path, State},
    http::{StatusCode, Uri},
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use crate::{app::App, error::Result};

const USAGE: &str = "
    USAGE

      POST /

          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content

      GET /<id>

          retrieves the content for the paste with id `<id>`
    ";

/// Return the usage string for our web app.
pub async fn index() -> &'static str { USAGE }

/// Retrieve a paste by its UUID.
///
/// Extracts the UUID from the query parameters, and a database connection from
/// the applications state.
pub async fn retrieve(
    Path(id): Path<Uuid>,
    State(state): State<App>,
) -> Result<(StatusCode, String)> {
    let paste = state.db.get_paste(id).await?;

    let response = match paste {
        Some(p) => (StatusCode::OK, p.content),
        None => (StatusCode::NOT_FOUND, "Paste not found".to_string()),
    };

    Ok(response)
}

/// Upload a paste.
///
/// Extracts the host url, body of the request, and a database connection from
/// the application state.
pub async fn upload(
    State(state): State<App>,
    Host(host): Host,
    body: String,
) -> Result<String> {
    let paste = state.db.create_paste(body).await?;

    // Construct a complete URI to the paste,
    // so the user can easily copy and save it.
    let paste_uri = format!("https://{}/{}", host, paste.id).parse::<Uri>()?;

    Ok(paste_uri.to_string())
}

pub fn make_router() -> Router<App> {
    Router::new()
        .route("/", get(index))
        .route("/", post(upload))
        .route("/:id", get(retrieve))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use async_trait::async_trait;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;
    use tokio::sync::Mutex;

    use super::*;
    use crate::paste::{Paste, PasteStore};

    // Create Mock database type.
    #[derive(Default)]
    struct MockPasteStore {
        pub entries: Mutex<HashMap<Uuid, String>>,
    }

    // Make convenience methods for it.
    impl MockPasteStore {
        pub fn arc() -> Arc<Self> { Arc::new(Self::default()) }
    }

    // Implement our database trait on it.
    #[async_trait]
    impl PasteStore for MockPasteStore {
        async fn get_paste(&self, id: Uuid) -> Result<Option<Paste>> {
            let lock = self.entries.lock().await;
            let paste = lock.get(&id).map(|c| Paste::new(id, c.clone()));
            Ok(paste)
        }

        async fn create_paste(&self, content: String) -> Result<Paste> {
            let id = Uuid::new_v4();
            let mut lock = self.entries.lock().await;
            lock.insert(id, content.clone());
            Ok(Paste { id, content })
        }
    }

    // Extend app to have a mock method that uses the Mock database.
    impl App {
        pub fn mock() -> Self {
            Self {
                db: MockPasteStore::arc(),
            }
        }
    }

    impl Paste {
        pub fn new(id: Uuid, content: String) -> Self { Self { id, content } }
    }

    // Get a test client suitable for use within tests,
    // sans any infrastructural setup (Databases, services, etc.).
    fn get_client() -> TestClient {
        // Construct router with mock db.
        let router = make_router().with_state(App::mock());

        // Create test client to router.
        TestClient::new(router)
    }

    #[tokio::test]
    async fn test_index() -> Result<()> {
        let client = get_client();

        // Test that index succeeds.
        let response = client.get("/").send().await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await, USAGE);

        Ok(())
    }

    #[tokio::test]
    async fn test_add_get() -> Result<()> {
        let client = get_client();

        // Create a paste to upload then retrieve.
        let paste = "This is a paste!";

        // Test that post succeeds.
        let response = client.post("/").body(paste.to_string()).send().await;
        assert_eq!(response.status(), StatusCode::OK);

        // Get the paste id from the response.
        let body = response.text().await;
        let uri = body.parse::<Uri>()?;
        let id = uri.path();

        // Test that get succeeds.
        let response = client.get(id).send().await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.text().await, paste);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_non_existent() -> Result<()> {
        let client = get_client();

        // Test that get fails the way we expect.
        let id = Uuid::new_v4();
        let response = client.get(&format!("/{}", id)).send().await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
}
