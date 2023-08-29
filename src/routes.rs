use axum::{
    extract::{Host, Path, State},
    http::Uri,
};
use uuid::Uuid;

use crate::{app::App, error::Result};

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

pub async fn retrieve(
    Path(id): Path<Uuid>,
    State(state): State<App>,
) -> Result<String> {
    let paste = sqlx::query_as!(
        crate::models::Paste,
        "SELECT id, content FROM pastes WHERE id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(paste.content)
}

pub async fn upload(
    State(state): State<App>,
    Host(host): Host,
    body: String,
) -> Result<String> {
    let paste = sqlx::query_as!(
        crate::models::Paste,
        "INSERT INTO pastes(content) VALUES ($1) RETURNING id, content",
        body
    )
    .fetch_one(&state.db)
    .await?;

    let paste_uri = Uri::builder()
        .scheme("https")
        .authority(host)
        .path_and_query(String::from("/") + &paste.id.to_string())
        .build()?;

    Ok(paste_uri.to_string())
}
