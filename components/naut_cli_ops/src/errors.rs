use sic_parser::errors::SicParserError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SicCliOpsError {
    #[error("Unable to parse: {0}")]
    ParserError(#[from] SicParserError),

    #[error("Failed to parse value of type `{typ}`, error:\n\t{err}")]
    UnableToParseValueOfType { err: SicParserError, typ: String },

    #[error("Internal Error: {0}")]
    InternalError(InternalErrorSource),

    #[error("Expected argument for image operation '{0}' (argument #{1})")]
    ExpectedArgumentForImageOperation(String, usize),
}

#[derive(Debug, Error)]
pub enum InternalErrorSource {
    #[error("no matching image operation found")]
    NoMatchingOperator,
}
