//! Errors returned by the canonical HRN parser and action mapper.

use thiserror::Error;

/// Escapes control characters before a raw value is embedded in an error.
///
/// The parser rejects non-ASCII, but ASCII control characters are legal bytes;
/// without escaping they could forge log lines or manipulate a terminal.
pub(crate) fn escape_value(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

/// Errors returned when parsing a resource HRN.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum HrnError {
    #[error("HRN must be ASCII")]
    NonAscii,

    #[error("HRN exceeds the {0}-byte limit")]
    TooLong(usize),

    #[error("HRN must start with 'hrn:'")]
    InvalidPrefix,

    #[error("HRN must have exactly 6 colon-separated components, found {0}")]
    InvalidComponentCount(usize),

    #[error("HRN field '{field}' must not be empty")]
    EmptyField { field: &'static str },

    #[error("HRN field '{field}' has invalid value '{value}'")]
    InvalidField { field: &'static str, value: String },

    #[error("resource type 'Action' is not a resource; use ActionId")]
    ActionIsNotResource,

    #[error(
        "HRN is syntactically valid but ambiguous for its origin; do not reinterpret it without provenance"
    )]
    AmbiguousOrigin,
}

/// Errors returned when building or migrating an action identifier.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum ActionIdError {
    #[error("legacy action must start with 'hrn:'")]
    InvalidPrefix,

    #[error("legacy action must have exactly 6 colon-separated components, found {0}")]
    InvalidComponentCount(usize),

    #[error("action partition has invalid value '{0}'")]
    InvalidPartition(String),

    #[error("action service has invalid value '{0}'")]
    InvalidService(String),

    #[error("action name has invalid value '{0}'")]
    InvalidName(String),

    #[error("legacy HRN is not an Action")]
    NotAnAction,

    #[error("legacy action must not carry region or account scope")]
    ScopedAction,

    #[error("action input exceeds the {0}-byte limit")]
    TooLong(usize),

    #[error("Cedar action UID has invalid value '{0}'")]
    InvalidCedarUid(String),
}
