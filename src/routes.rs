use axum::{
    extract::{Host, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use syntect::{
    easy::HighlightLines,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
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
    let paste = state.pastes.get(id).await?;

    let response = match paste {
        Some(p) => (StatusCode::OK, p.content),
        None => (StatusCode::NOT_FOUND, "Paste not found".to_string()),
    };

    Ok(response)
}

pub async fn retrieve_and_syntax_highlight(
    Path((id, lang)): Path<(Uuid, String)>,
    State(state): State<App>,
) -> Result<(StatusCode, String)> {
    let paste = state.pastes.get(id).await?;
    let syntax = state.syntax_set.find_syntax_by_extension(&lang);

    let response = match paste {
        Some(p) => match syntax {
            Some(syntax) => {
                let mut highlighter = HighlightLines::new(
                    syntax,
                    &state.theme_set.themes["base16-ocean.dark"],
                );
                let mut lines = Vec::new();
                for line in LinesWithEndings::from(&p.content) {
                    let ranges = highlighter.highlight_line(line, &state.syntax_set)?;
                    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                    lines.push(escaped + "\x1b[0m");
                }
                (StatusCode::OK, lines.join(""))
            }
            None => (StatusCode::OK, p.content),
        },
        None => (StatusCode::NOT_FOUND, "Paste not found".to_string()),
    };

    Ok(response)
}
/// myapp.com/a/b
/// myapp.com/a/b/c where c is optional but not not provided

pub async fn remove(
    Path(id): Path<Uuid>,
    State(state): State<App>,
) -> Result<(StatusCode, &'static str)> {
    let paste = state.pastes.remove(id).await?;

    let response = match paste {
        Some(_) => (StatusCode::OK, "Deleted!"),
        None => (StatusCode::NOT_FOUND, "Paste not found"),
    };

    Ok(response)
}

fn scheme(host: &str) -> &'static str {
    if host.contains("127.0.0.1") || host.contains("localhost") {
        "http"
    } else {
        "https"
    }
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
    let paste = state.pastes.create(body).await?;

    // Construct a complete URI to the paste,
    // so the user can easily copy and save it.
    Ok(format!("{}://{}/{}", scheme(&host), host, paste.id))
}

pub fn make_router() -> Router<App> {
    Router::new()
        .route("/", get(index))
        .route("/", post(upload))
        .route("/:id", get(retrieve))
        .route("/:id/:lang", get(retrieve_and_syntax_highlight))
        .route("/:id", delete(remove))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use async_trait::async_trait;
    use axum::http::{StatusCode, Uri};
    use axum_test_helper::TestClient;
    use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
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
        async fn get(&self, id: Uuid) -> Result<Option<Paste>> {
            let lock = self.entries.lock().await;
            let paste = lock.get(&id).map(|c| Paste::new(id, c.clone()));
            Ok(paste)
        }

        async fn create(&self, content: String) -> Result<Paste> {
            let id = Uuid::new_v4();
            let mut lock = self.entries.lock().await;
            lock.insert(id, content.clone());
            Ok(Paste { id, content })
        }

        async fn remove(&self, id: Uuid) -> Result<Option<Paste>> {
            let mut lock = self.entries.lock().await;
            let paste = lock.remove(&id).map(|c| Paste::new(id, c));
            Ok(paste)
        }
    }

    // Extend app to have a mock method that uses the Mock database.
    impl App {
        pub fn mock() -> Self {
            Self {
                pastes: MockPasteStore::arc(),
                syntax_set: Arc::new(SyntaxSet::load_defaults_newlines()),
                theme_set: Arc::new(ThemeSet::new()),
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

    #[tokio::test]
    async fn test_delete() -> Result<()> {
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

        let response = client.delete(id).send().await;
        assert_eq!(response.status(), StatusCode::OK);

        // Test that get fails the way we expect.
        let response = client.get(id).send().await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_non_existent() -> Result<()> {
        let client = get_client();

        // Test that get fails the way we expect.
        let id = Uuid::new_v4();
        let response = client.delete(&format!("/{}", id)).send().await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        Ok(())
    }
}
