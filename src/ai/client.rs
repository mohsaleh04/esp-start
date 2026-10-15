use crate::ai::{AiError, models, models::ModelList};
use crate::net::{HttpClient, HttpRequest};
use reqwless::request::Method;

pub struct AiClient;

impl AiClient {
    pub async fn get_models<F>(http_client: &mut HttpClient, on_result: F)
    where
        F: FnOnce(Result<ModelList, AiError>),
    {
        let request = HttpRequest {
            method: Method::GET,
            path: "/api/v1/models",
            headers: &[("Accept", "application/json")],
            body: None,
        };

        let result = http_client
            .send(request, |status, body| {
                let status_code = status.0;
                if status_code == 200 {
                    models::from_json(body).map_err(AiError::from)
                } else {
                    Err(AiError::HttpStatus(status_code))
                }
            })
            .await
            .map_err(AiError::Http)
            .flatten(); // Note: equal as *.and_then(|inner| inner) for earlier Rust v1.89

        on_result(result);
    }
}
