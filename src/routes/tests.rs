use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::http::{StatusCode, Uri};
use axum_test_helper::TestClient;
use strip_ansi_escapes::strip_str;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::handlers::*;
use crate::{
    app::App,
    error::Result,
    paste::{Paste, PasteStore},
};

// Create Mock database type.
#[derive(Default)]
struct MockPasteStore {
    entries: Mutex<HashMap<Uuid, String>>,
}

// Implement our database trait on it.
#[async_trait]
impl PasteStore for MockPasteStore {
    async fn get(&self, id: Uuid) -> Result<Option<Paste>> {
        let lock = self.entries.lock().await;
        let paste = lock.get(&id).map(|c| {
            let content = c.clone();
            Paste { id, content }
        });
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
        let paste = lock.remove(&id).map(|c|{
            let content = c.clone();
            Paste { id, content }
        });
        Ok(paste)
    }
}

// Get a test client suitable for use within tests,
// sans any infrastructural setup (Databases, services, etc.).
fn get_client() -> TestClient {
    // Construct router with mock store.
    let store = Arc::new(MockPasteStore::default());
    let state = App { pastes: store };
    let router = make_router().with_state(state);

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
    assert_eq!(response.status(), StatusCode::CREATED);

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
    let id = Uuid::new_v4(); // Least flaky possible test ;)
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
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get the paste id from the response.
    let body = response.text().await;
    let uri = body.parse::<Uri>()?;
    let id = uri.path();

    // Test that get succeeds.
    let response = client.get(id).send().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await, paste);

    // Test that delete succeeds.
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

#[tokio::test]
async fn test_get_syntax_highlighted() -> Result<()> {
    let client = get_client();

    // Create a paste to upload then retrieve.
    let paste = "let x = 5;";

    // Test that post succeeds.
    let response = client.post("/").body(paste.to_string()).send().await;
    assert_eq!(response.status(), StatusCode::CREATED);

    // Get the paste id from the response.
    let body = response.text().await;
    let uri = body.parse::<Uri>()?;
    let id = uri.path();

    // Test that get succeeds.
    let response = client.get(id).send().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await, paste);

    // Test that syntax highlighted get succeeds.
    let response = client.get(&format!("{}/rs", id)).send().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(strip_str(response.text().await), paste);

    Ok(())
}
