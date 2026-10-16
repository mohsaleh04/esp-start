use crate::ai::AiError;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatRequest {
    #[serde(rename = "model")]
    pub selected_model_name: String,
    pub system_prompt: Option<String>,
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatResponse {
    pub model_instance_id: String,
    pub output: Vec<AiChatOutput>,
    pub stats: AiChatStats,
    pub response_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatOutput {
    #[serde(rename = "type")]
    pub output_type: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiChatStats {
    pub input_tokens: u32,
    pub total_output_tokens: u32,
    pub reasoning_output_tokens: u32,
    pub tokens_per_second: f32,
    pub time_to_first_token_seconds: f32,
}

pub fn from_json(json: &[u8]) -> Result<AiChatResponse, AiError> {
    serde_json::from_slice(json).map_err(AiError::Json)
}

pub fn to_json(message: AiChatRequest) -> Result<Vec<u8>, AiError> {
    serde_json::to_vec(&message).map_err(|err| AiError::Json(err))
}
