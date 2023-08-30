use axum::{
    extract::{Host, Path, State},
    http::Uri,
};
use uuid::Uuid;

use crate::{app::App, error::Result};

/// Return the usage string for our web app.
pub async fn index() -> &'static str {
    "
    USAGE

      POST /

          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content

      GET /<id>

          retrieves the content for the paste with id `<id>`
    "
}

/// Retrieve a paste by its UUID.
///
/// Extracts the UUID from the query parameters, and a database connection from
/// the applications state.
pub async fn retrieve(
    Path(id): Path<Uuid>,
    State(state): State<App>,
) -> Result<String> {
    // Compile time checked query.
    // Run `cargo sqlx prepare` to update checked queries.
    let paste = sqlx::query_as!(
        crate::models::Paste,
        "SELECT id, content FROM pastes WHERE id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(paste.content)
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
    // Compile time checked query.
    // Run `cargo sqlx prepare` to update checked queries.
    let paste = sqlx::query_as!(
        crate::models::Paste,
        "INSERT INTO pastes(content) VALUES ($1) RETURNING id, content",
        body
    )
    .fetch_one(&state.db)
    .await?;

    // Construct a complete URI to the paste,
    // so the user can easily copy and save it.
    let paste_uri = format!("https://{}/{}", host, paste.id).parse::<Uri>()?;

    Ok(paste_uri.to_string())
}
