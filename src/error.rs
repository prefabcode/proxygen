use super::serde_json::Error as JsonError;

#[derive(Debug)]
pub enum ProxygenError {
    DecklistParseError(String),
    InvalidCardName(String),
    JsonError(JsonError),
}

impl From<JsonError> for ProxygenError {
    fn from(e: JsonError) -> ProxygenError {
        ProxygenError::JsonError(e)
    }
}
