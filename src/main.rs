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
    // Initialize the router.
    // Map our routes and set up our state.
    let router = Router::new()
        .route("/", get(routes::index))
        .route("/", post(routes::upload))
        .route("/:id", get(routes::retrieve))
        .with_state(app::App { db: pool });

    // Let shuttle take the wheel :^)
    Ok(router.into())
}
