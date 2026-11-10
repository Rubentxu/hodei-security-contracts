//! Action identities are separate from resource HRNs.

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{ActionIdError, escape_value};

/// Maximum accepted legacy/UID action input, in bytes.
pub const MAX_ACTION_INPUT_LEN: usize = 512;

/// A Cedar action identity: `<ServiceNamespace>::Action::"<name>"`.
///
/// Actions are verbs, not resources. They are never represented as an
/// [`crate::Hrn`]; the legacy `hrn:...:Action/<name>` form is only a migration
/// input. The Cedar namespace is derived injectively from the service:
/// hyphens become underscores and the first letter is uppercased. For example
/// `audit-trail` becomes `Audit_trail`, while `a-1` and `a1` stay distinct.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId {
    service: String,
    name: String,
}

impl ActionId {
    /// Build a canonical action identity.
    pub fn new(service: &str, name: &str) -> Result<Self, ActionIdError> {
        if service.len() > 64 {
            return Err(ActionIdError::InvalidService(format!(
                "<{} bytes>",
                service.len()
            )));
        }
        if name.len() > 128 {
            return Err(ActionIdError::InvalidName(format!(
                "<{} bytes>",
                name.len()
            )));
        }

        let service = service.to_ascii_lowercase();
        if !valid_service(&service) {
            return Err(ActionIdError::InvalidService(escape_value(&service)));
        }
        if !valid_action_name(name) {
            return Err(ActionIdError::InvalidName(escape_value(name)));
        }
        Ok(Self {
            service,
            name: name.to_string(),
        })
    }

    /// Convert the legacy `hrn:<partition>:<service>:::Action/<name>` form.
    ///
    /// The legacy form is rejected by [`crate::Hrn::parse`] on purpose. It is
    /// accepted here only to migrate persisted action strings, and it must not
    /// carry region or account scope.
    pub fn from_legacy_hrn(input: &str) -> Result<Self, ActionIdError> {
        if input.len() > MAX_ACTION_INPUT_LEN {
            return Err(ActionIdError::TooLong(MAX_ACTION_INPUT_LEN));
        }
        if !input.is_ascii() {
            return Err(ActionIdError::InvalidCedarUid(escape_value(input)));
        }
        let rest = input
            .strip_prefix("hrn:")
            .ok_or(ActionIdError::InvalidPrefix)?;
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() != 5 {
            return Err(ActionIdError::InvalidComponentCount(parts.len() + 1));
        }

        if !valid_partition(parts[0]) {
            return Err(ActionIdError::InvalidPartition(escape_value(parts[0])));
        }
        let service = parts[1].to_ascii_lowercase();
        if !valid_service(&service) {
            return Err(ActionIdError::InvalidService(escape_value(parts[1])));
        }
        if !parts[2].is_empty() || !parts[3].is_empty() {
            return Err(ActionIdError::ScopedAction);
        }

        let (resource_type, name) = parts[4].split_once('/').ok_or(ActionIdError::NotAnAction)?;
        if !resource_type.eq_ignore_ascii_case("Action") {
            return Err(ActionIdError::NotAnAction);
        }
        if !valid_action_name(name) {
            return Err(ActionIdError::InvalidName(escape_value(name)));
        }

        Ok(Self {
            service,
            name: name.to_string(),
        })
    }

    /// Parse a canonical Cedar action UID.
    pub fn from_cedar_uid(input: &str) -> Result<Self, ActionIdError> {
        if input.len() > MAX_ACTION_INPUT_LEN {
            return Err(ActionIdError::TooLong(MAX_ACTION_INPUT_LEN));
        }
        if !input.is_ascii() {
            return Err(ActionIdError::InvalidCedarUid(escape_value(input)));
        }

        let (namespace, quoted) = input
            .split_once("::Action::\"")
            .ok_or_else(|| ActionIdError::InvalidCedarUid(escape_value(input)))?;
        let name = quoted
            .strip_suffix('"')
            .ok_or_else(|| ActionIdError::InvalidCedarUid(escape_value(input)))?;
        if name.contains('"') {
            return Err(ActionIdError::InvalidCedarUid(escape_value(input)));
        }

        let service = namespace.to_ascii_lowercase().replace('_', "-");
        if !valid_service(&service) {
            return Err(ActionIdError::InvalidService(escape_value(namespace)));
        }
        if !valid_action_name(name) {
            return Err(ActionIdError::InvalidName(escape_value(name)));
        }
        if cedar_namespace(&service) != namespace {
            return Err(ActionIdError::InvalidCedarUid(escape_value(input)));
        }

        Ok(Self {
            service,
            name: name.to_string(),
        })
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Canonical Cedar entity UID used by the policy engine.
    pub fn to_cedar_uid(&self) -> String {
        format!(
            "{}::Action::\"{}\"",
            cedar_namespace(&self.service),
            self.name
        )
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_cedar_uid())
    }
}

impl FromStr for ActionId {
    type Err = ActionIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_cedar_uid(s)
    }
}

impl Serialize for ActionId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_cedar_uid())
    }
}

impl<'de> Deserialize<'de> for ActionId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        ActionId::from_cedar_uid(&raw).map_err(D::Error::custom)
    }
}

fn valid_partition(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

fn valid_service(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !value.ends_with('-')
}

fn valid_action_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Injective Cedar namespace for a canonical service name.
///
/// `-` becomes `_`, which is not a service character, so two different
/// services cannot collapse to the same namespace. The first character must
/// already be a lowercase letter.
fn cedar_namespace(service: &str) -> String {
    // `ActionId` fields are private and every constructor validates a
    // non-empty service, so this `expect` is not reachable from public input.
    let first = service
        .chars()
        .next()
        .expect("validated service is non-empty")
        .to_ascii_uppercase();
    let rest: String = service
        .chars()
        .skip(1)
        .map(|c| if c == '-' { '_' } else { c })
        .collect();
    format!("{first}{rest}")
}
