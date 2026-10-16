use crate::ai::{models::{ai_chat, ai_chat::AiChatRequest, ai_model_list, ai_model_list::ModelList}, AiChatResponse, AiError};
use crate::net::{HttpClient, HttpRequest};
use reqwless::{request::Method, response::StatusCode};

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

        on_result(
            Self::send_req(http_client, request, |status, body| {
                if status.is_successful() {
                    ai_model_list::from_json(body).map_err(AiError::from)
                } else {
                    Err(AiError::HttpStatus(status.0))
                }
            }).await
        );
    }

    pub async fn send_message<F>(http_client: &mut HttpClient, chat_message: AiChatRequest, on_result: F)
    where
        F: FnOnce(Result<AiChatResponse, AiError>),
    {
        let request_body = match ai_chat::to_json(chat_message).map_err(AiError::from) {
            Ok(body) => body,
            Err(e) => {
                on_result(Err(e));
                return;
            }
        };

        let request = HttpRequest {
            method: Method::POST,
            path: "/api/v1/chat",
            headers: &[
                ("Accept", "application/json"),
                ("Content-Type", "application/json")
            ],
            body: Some(request_body.as_slice()),
        };

        on_result(
            Self::send_req(http_client, request, |status, body| {
                if status.is_successful() {
                    ai_chat::from_json(body).map_err(AiError::from)
                } else {
                    Err(AiError::HttpStatus(status.0))
                }
            }).await
        );
    }

    async fn send_req<F, R>(
        http_client: &mut HttpClient,
        request: HttpRequest<'_>,
        on_response: F,
    ) -> Result<R, AiError> where
        F: FnOnce(StatusCode, &[u8]) -> Result<R, AiError>,
    {
        http_client.send(request, on_response).await.map_err(AiError::Http).flatten()
    }
}
