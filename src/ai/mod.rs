pub use client::AiClient;
pub use error::AiError;
pub use models::ai_chat::AiChatRequest;
pub use models::ai_chat::AiChatResponse;
pub use models::ai_model_list::AiModel;

mod client;
mod error;
mod models;

pub const DEFAULT_MODEL_KEY: &str = "google/gemma-4-e4b";
