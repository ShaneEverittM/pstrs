use axum::http::StatusCode;
use easy_ext::ext;

/// Extension method(s) for converting `Option`s to HTTP response codes.
#[ext(OptionHttpExt)]
pub impl<T> Option<T>
where
    T: Default,
{
    /// Unwraps an `Option`, yielding the content of a `Some` paired with 200
    /// OK, or a default value paired with 404 Not Found
    fn unwrap_or_not_found(self) -> (StatusCode, T) {
        match self {
            Some(content) => (StatusCode::OK, content),
            None => (StatusCode::NOT_FOUND, T::default()),
        }
    }
}
