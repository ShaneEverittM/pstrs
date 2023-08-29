use axum::{
    routing::{get, post},
    Router,
};
use shuttle_axum::ShuttleAxum;
use shuttle_shared_db::Postgres;
use sqlx::PgPool;

mod app;
mod error;
mod models;
mod routes;

#[shuttle_runtime::main]
async fn axum(#[Postgres] pool: PgPool) -> ShuttleAxum {
    let router = Router::new()
        .route("/todo", post(routes::create_todo))
        .route("/todo/:id", get(routes::retrieve))
        .with_state(app::App { db: pool });

    Ok(router.into())
}
