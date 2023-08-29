use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    app::App,
    error::Result,
    models::{Todo, TodoNew},
};

pub async fn retrieve(
    Path(id): Path<i32>,
    State(state): State<App>,
) -> Result<Json<Todo>> {
    let todo = sqlx::query_as!(Todo, "SELECT id, note FROM todos WHERE id = $1", id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(todo))
}

pub async fn create_todo(
    State(state): State<App>,
    Json(json): Json<TodoNew>,
) -> Result<Json<Todo>> {
    let todo = sqlx::query_as!(
        Todo,
        "INSERT INTO todos(note) VALUES ($1) RETURNING id, note",
        &json.note
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(todo))
}
