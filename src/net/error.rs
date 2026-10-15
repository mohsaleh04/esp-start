#[derive(Debug)]
pub enum HttpClientError {
    ConnectTimeout,
    ConnectFailed,
    RequestFailed,
    BodyReadFailed,
}
