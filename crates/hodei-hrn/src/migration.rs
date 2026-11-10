//! Origin-aware classification for persisted HRN strings.
//!
//! Syntax alone cannot tell an Audit Trail tenant from a canonical region.
//! Callers that read legacy data must use [`classify`] with the origin of the
//! data; [`crate::Hrn::parse`] is syntax-only and must not be used to
//! reinterpret legacy layouts.

use crate::{Hrn, HrnError};

/// System that produced a persisted HRN string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    /// Artifacts v2: canonical layout.
    ArtifactsV2,
    /// Artifacts v1: empty region is the v1 alias.
    ArtifactsV1,
    /// hodei-authz v1: the fourth field is a tenant, not necessarily an account.
    HodeiAuthzV1,
    /// Audit Trail: tenant and region are swapped relative to the contract.
    AuditTrail,
    /// No provenance available.
    Unknown,
}

impl Origin {
    /// Parse the origin label used by the conformance vectors.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "artifacts-v2" => Some(Self::ArtifactsV2),
            "artifacts-v1" | "artifacts-legacy" => Some(Self::ArtifactsV1),
            "hodei-authz" => Some(Self::HodeiAuthzV1),
            "audit-trail" => Some(Self::AuditTrail),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Outcome of an origin-aware read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationClass {
    /// Byte-identical canonical form.
    Canonical,
    /// Accepted, but the canonical form differs from the stored bytes.
    Alias,
}

/// Classify a persisted HRN string using its origin.
///
/// Ambiguous origins (Authz v1, Audit Trail, or unknown) are rejected with
/// [`HrnError::AmbiguousOrigin`] even when the string is syntactically valid;
/// the caller must resolve the provenance before using it as a canonical HRN.
pub fn classify(origin: Origin, input: &str) -> Result<(Hrn, MigrationClass), HrnError> {
    let hrn = Hrn::parse(input)?;
    match origin {
        Origin::ArtifactsV2 => {
            let class = if hrn.to_string() == input {
                MigrationClass::Canonical
            } else {
                MigrationClass::Alias
            };
            Ok((hrn, class))
        }
        Origin::ArtifactsV1 if hrn.region().is_empty() => Ok((hrn, MigrationClass::Alias)),
        Origin::ArtifactsV1 => Err(HrnError::AmbiguousOrigin),
        Origin::HodeiAuthzV1 | Origin::AuditTrail | Origin::Unknown => {
            Err(HrnError::AmbiguousOrigin)
        }
    }
}
