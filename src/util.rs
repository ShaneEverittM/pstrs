/// Determine what scheme the URI for the currently running
/// app likely is.
///
/// If the app appears to be running on localhost, return
/// "http", otherwise return "https".
pub fn scheme(host: &str) -> &'static str {
    if host.contains("127.0.0.1") || host.contains("localhost") {
        "http"
    } else {
        "https"
    }
}
