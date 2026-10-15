use crate::ai::AiError;
use alloc::{string::String, vec::Vec};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModelList {
    pub models: Vec<AiModel>,
}

#[derive(Debug, Deserialize)]
pub struct AiModel {
    pub key: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub model_type: String,
}

#[derive(Debug)]
pub struct AiState {
    pub selected_model: Option<AiModel>,
}

pub(super) fn from_json(json: &[u8]) -> Result<ModelList, AiError> {
    serde_json::from_slice(json).map_err(|err| AiError::Json(err))
}
