use shuttle_axum::ShuttleAxum;
use shuttle_shared_db::Postgres;
use sqlx::PgPool;

mod app;
mod error;
mod paste;
mod routes;

#[shuttle_runtime::main]
async fn axum(#[Postgres] pool: PgPool) -> ShuttleAxum {
    // Initialize the router.
    let router = routes::make_router().with_state(app::App::postgres(pool));

    // Let shuttle take the wheel :^)
    Ok(router.into())
}
