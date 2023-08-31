use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A type alias for `Result<T, AppError>` that is suitable
/// for use as the primary error type for this application.
///
/// It employs type erasure through `anyhow::Error` to allow
/// for easy conversion from other error types, and since our error
/// path isn't critical the performance overhead isn't a problem.
pub type Result<T> = anyhow::Result<T, AppError>;

/// A new type around `anyhow::Error` so that we can implement [IntoResponse].
#[derive(Debug)]
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>`
// (or any thing convertable to `anyhow::Error` for that matter) to turn them
// into `Result<_, AppError>`.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self { Self(err.into()) }
}
