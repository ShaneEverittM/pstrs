use axum::{
    extract::{Host, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use uuid::Uuid;

use crate::{app::App, error::Result, ext::OptionHttpExt, util};

pub const USAGE: &str = "
    USAGE

      POST /

          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content

      GET /<id>

          retrieves the content for the paste with id `<id>`
    ";

async fn index() -> &'static str { USAGE }

async fn upload(
    State(state): State<App>,
    Host(host): Host,
    body: String,
) -> Result<(StatusCode, String)> {
    let paste = state.pastes.create(body).await?;

    // Construct a complete URI to the paste,
    // so the user can easily copy and save it.
    let response = format!("{}://{}/{}", util::scheme(&host), host, paste.id);

    Ok((StatusCode::CREATED, response))
}

async fn retrieve(
    Path(id): Path<Uuid>,
    State(state): State<App>,
) -> Result<(StatusCode, String)> {
    // Retrieve the paste from the database.
    let content = state.pastes.get(id).await?.map(|p| p.content);

    // Map the paste to a response.
    let response = content.unwrap_or_not_found();

    // Return the response.
    Ok(response)
}

async fn retrieve_highlighted(
    Path((id, lang)): Path<(Uuid, String)>,
    State(state): State<App>,
) -> Result<(StatusCode, String)> {
    let paste = state.pastes.get(id).await?;

    let response = paste
        .map(|p| p.to_highlighted(&lang, "base16-ocean.dark"))
        .unwrap_or_not_found();

    Ok(response)
}

async fn remove(
    Path(id): Path<Uuid>,
    State(state): State<App>,
) -> Result<(StatusCode, &'static str)> {
    let paste = state.pastes.remove(id).await?;

    let response = paste.map(|_| "Deleted!").unwrap_or_not_found();

    Ok(response)
}

pub fn make_router() -> Router<App> {
    Router::new()
        .route("/", get(index))
        .route("/", post(upload))
        .route("/:id", get(retrieve))
        .route("/:id/:lang", get(retrieve_highlighted))
        .route("/:id", delete(remove))
}
