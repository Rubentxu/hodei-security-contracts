//! Compatibility with the 0.1 field-object and its serde representation.
//!
//! 0.1 serialized `Hrn` as a JSON object with public fields. 0.2 serializes the
//! canonical string. [`LegacyHrnFields`] lets a consumer deserialize the old
//! object shape and convert it explicitly, without making the canonical type
//! writable or weakening validation.

use serde::{Deserialize, Serialize};

use crate::{Hrn, HrnError};

/// The JSON/field shape used by `hodei-hrn` 0.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyHrnFields {
    pub partition: String,
    pub service: String,
    pub region: String,
    pub tenant_id: String,
    pub resource_type: String,
    pub resource_id: String,
}

impl TryFrom<LegacyHrnFields> for Hrn {
    type Error = HrnError;

    fn try_from(value: LegacyHrnFields) -> Result<Self, Self::Error> {
        Hrn::new(
            &value.partition,
            &value.service,
            &value.region,
            &value.tenant_id,
            &value.resource_type,
            &value.resource_id,
        )
    }
}

impl From<&Hrn> for LegacyHrnFields {
    fn from(value: &Hrn) -> Self {
        Self {
            partition: value.partition().to_string(),
            service: value.service().to_string(),
            region: value.region().to_string(),
            tenant_id: value.account().to_string(),
            resource_type: value.resource_type().to_string(),
            resource_id: value.resource_path().to_string(),
        }
    }
}
