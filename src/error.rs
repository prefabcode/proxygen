use std::error::Error as StdError;
use super::serde_json::Error as JsonError;
use super::ease::Error as EaseError;

#[derive(Debug)]
pub enum ProxygenError {
    DecklistParseError(String),
    InvalidCardName(String),
    JsonError(JsonError),
    EaseError(EaseError),
}

impl From<JsonError> for ProxygenError {
    fn from(e: JsonError) -> ProxygenError {
        ProxygenError::JsonError(e)
    }
}

impl From<EaseError> for ProxygenError {
    fn from(e: EaseError) -> ProxygenError {
        ProxygenError::EaseError(e)
    }
}
