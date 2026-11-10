//! Canonical resource HRN parsing and scope-aware comparison.

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{HrnError, escape_value};

/// Maximum accepted HRN length, in bytes.
///
/// It is the exact sum of the per-field limits plus the prefix and separators:
/// `4 + 64 + 1 + 64 + 1 + 64 + 1 + 128 + 1 + 128 + 1 + 1024 = 1481`.
pub const MAX_HRN_LEN: usize = 1481;

pub const MAX_PARTITION_LEN: usize = 64;
pub const MAX_SERVICE_LEN: usize = 64;
pub const MAX_REGION_LEN: usize = 64;
pub const MAX_ACCOUNT_LEN: usize = 128;
pub const MAX_RESOURCE_TYPE_LEN: usize = 128;
pub const MAX_RESOURCE_PATH_LEN: usize = 1024;

/// A canonical Hodei Resource Name.
///
/// Grammar:
/// `hrn:<partition>:<service>:<region>:<account>:<resource-type>/<resource-path>`
///
/// The `region` may be empty (legacy v1 alias and global resources); the
/// empty field is preserved byte-for-byte. The `service` is the only field
/// normalized: it is lowercased. Everything else is preserved.
///
/// `Hrn::parse` is syntax-only. Legacy strings that are structurally valid but
/// semantically ambiguous (for example Audit Trail's tenant/region swap) must
/// be handled through [`crate::migration::classify`] with their origin; the
/// parser cannot detect that ambiguity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hrn {
    partition: String,
    service: String,
    region: String,
    account: String,
    resource_type: String,
    resource_path: String,
}

impl Hrn {
    /// Parse and canonicalize a resource HRN.
    pub fn parse(input: &str) -> Result<Self, HrnError> {
        // The length check comes first: an over-long input must not be
        // scanned or allocated before it is rejected.
        if input.len() > MAX_HRN_LEN {
            return Err(HrnError::TooLong(MAX_HRN_LEN));
        }
        if !input.is_ascii() {
            return Err(HrnError::NonAscii);
        }

        let rest = input.strip_prefix("hrn:").ok_or(HrnError::InvalidPrefix)?;
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() != 5 {
            return Err(HrnError::InvalidComponentCount(parts.len() + 1));
        }

        let partition = require(parts[0], "partition")?;
        let service = require(parts[1], "service")?.to_ascii_lowercase();
        let region = parts[2];
        let resource = require(parts[4], "resource")?;

        if !valid_name(partition, MAX_PARTITION_LEN) {
            return Err(invalid("partition", partition));
        }
        if !valid_name(&service, MAX_SERVICE_LEN) {
            return Err(invalid("service", parts[1]));
        }
        if !region.is_empty() && !valid_name(region, MAX_REGION_LEN) {
            return Err(invalid("region", region));
        }

        let (resource_type, resource_path) = resource
            .split_once('/')
            .ok_or_else(|| invalid("resource", resource))?;
        if resource_type.is_empty() {
            return Err(HrnError::EmptyField {
                field: "resource_type",
            });
        }
        // Action is checked before the account so every legacy action form
        // consistently reports ActionIsNotResource, even when its scope is
        // empty.
        if resource_type.eq_ignore_ascii_case("action") {
            return Err(HrnError::ActionIsNotResource);
        }
        if resource_path.is_empty() {
            return Err(HrnError::EmptyField {
                field: "resource_path",
            });
        }
        if !valid_resource_type(resource_type) {
            return Err(invalid("resource_type", resource_type));
        }
        if resource_path.len() > MAX_RESOURCE_PATH_LEN || !valid_path(resource_path) {
            return Err(invalid("resource_path", resource_path));
        }

        let account = require(parts[3], "account")?;
        if !valid_account(account) {
            return Err(invalid("account", account));
        }

        Ok(Self {
            partition: partition.to_string(),
            service,
            region: region.to_string(),
            account: account.to_string(),
            resource_type: resource_type.to_string(),
            resource_path: resource_path.to_string(),
        })
    }

    /// Canonical string representation.
    pub fn canonical(&self) -> String {
        self.to_string()
    }

    pub fn partition(&self) -> &str {
        &self.partition
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn account(&self) -> &str {
        &self.account
    }

    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    pub fn resource_path(&self) -> &str {
        &self.resource_path
    }

    /// Whether this HRN is a strict descendant of `ancestor`.
    ///
    /// Descendancy is decided by **complete path segments**, never by a raw
    /// string prefix: `repository/releases/v2` is a descendant of
    /// `repository/releases`, while `repository/releases-old` is not.
    pub fn is_descendant_of(&self, ancestor: &Self) -> bool {
        if self == ancestor {
            return false;
        }
        if self.partition != ancestor.partition
            || self.service != ancestor.service
            || self.region != ancestor.region
            || self.account != ancestor.account
            || self.resource_type != ancestor.resource_type
        {
            return false;
        }
        if self.resource_path.len() <= ancestor.resource_path.len()
            || !self.resource_path.starts_with(&ancestor.resource_path)
        {
            return false;
        }
        self.resource_path.as_bytes()[ancestor.resource_path.len()] == b'/'
    }

    /// Parent HRN, or `None` when the path has a single segment.
    pub fn parent(&self) -> Option<Self> {
        let (parent, _) = self.resource_path.rsplit_once('/')?;
        Some(Self {
            resource_path: parent.to_string(),
            ..self.clone()
        })
    }
}

impl fmt::Display for Hrn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hrn:{}:{}:{}:{}:{}/{}",
            self.partition,
            self.service,
            self.region,
            self.account,
            self.resource_type,
            self.resource_path
        )
    }
}

impl FromStr for Hrn {
    type Err = HrnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl Serialize for Hrn {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Hrn {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Hrn::parse(&raw).map_err(D::Error::custom)
    }
}

fn require<'a>(value: &'a str, field: &'static str) -> Result<&'a str, HrnError> {
    if value.is_empty() {
        Err(HrnError::EmptyField { field })
    } else {
        Ok(value)
    }
}

fn invalid(field: &'static str, value: &str) -> HrnError {
    HrnError::InvalidField {
        field,
        value: escape_value(value),
    }
}

fn valid_name(value: &str, max_len: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !value.starts_with('-')
        && !value.ends_with('-')
}

fn valid_account(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ACCOUNT_LEN
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
}

fn valid_resource_type(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_RESOURCE_TYPE_LEN
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn valid_path(value: &str) -> bool {
    value.split('/').all(|segment| {
        !segment.is_empty()
            && segment != "."
            && segment != ".."
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '~' | '-'))
    })
}
