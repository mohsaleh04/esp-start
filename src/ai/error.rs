use crate::net::HttpClientError;

#[derive(Debug)]
pub enum AiError {
    Http(HttpClientError),
    HttpStatus(u16),
    Json(serde_json::Error),
}
