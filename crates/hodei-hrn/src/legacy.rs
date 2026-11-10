//! Compatibility with the 0.1 field-object and its serde representation.
//!
//! 0.1 serialized `Hrn` as a JSON object with public fields. 0.2 serializes the
//! canonical string. [`LegacyHrnFields`] lets a consumer deserialize the old
//! object shape and convert it explicitly, without making the canonical type
//! writable or weakening validation.

use serde::{Deserialize, Serialize};

use crate::migration::Origin;
use crate::{Hrn, HrnError};

/// Explicit provenance required to interpret a legacy field object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationContext {
    origin: Origin,
}

impl MigrationContext {
    pub fn new(origin: Origin) -> Self {
        Self { origin }
    }

    pub fn origin(self) -> Origin {
        self.origin
    }
}

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

impl LegacyHrnFields {
    /// Convert the 0.1 field object only when the caller supplies provenance.
    ///
    /// `tenant_id` is not universally synonymous with `account`: Authz v1 and
    /// Audit Trail layouts are ambiguous. Those origins are rejected instead of
    /// silently reinterpreting the field.
    pub fn into_canonical(self, context: MigrationContext) -> Result<Hrn, HrnError> {
        match context.origin {
            Origin::ArtifactsV1 if !self.region.is_empty() => Err(HrnError::AmbiguousOrigin),
            Origin::ArtifactsV1 | Origin::ArtifactsV2 => Hrn::new(
                &self.partition,
                &self.service,
                &self.region,
                &self.tenant_id,
                &self.resource_type,
                &self.resource_id,
            ),
            Origin::HodeiAuthzV1 | Origin::AuditTrail | Origin::Unknown => {
                Err(HrnError::AmbiguousOrigin)
            }
        }
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
