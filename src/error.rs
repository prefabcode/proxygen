use super::serde_json::Error as JsonError;

#[derive(Debug)]
pub enum ProxygenError {
    TooManyCards,
    DecklistParseError(String),
    InvalidCardName(String),
    MulticardHasNoNames(String),
    MulticardHasMalformedNames(String),
    JsonError(JsonError),
}

impl From<JsonError> for ProxygenError {
    fn from(e: JsonError) -> ProxygenError {
        ProxygenError::JsonError(e)
    }
}
