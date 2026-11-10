//! Compatibility builder for the 0.1 API.
//!
//! The builder keeps the legacy method names (`tenant_id`, `resource`) and
//! defaults (`hodei`, `global`), but every `build()` goes through the strict
//! canonical parser. Public fields are no longer writable: use [`Hrn::new`] or
//! [`HrnBuilder`] so an invalid HRN cannot be constructed.

use crate::{Hrn, HrnError};

/// Builder with the legacy 0.1 method names.
///
/// Defaults are `partition = "hodei"` and `region = "global"`, matching the
/// published 0.1 behavior. Call `.region("")` for the canonical empty region:
/// `global` is a literal region and is **not** equivalent to `""`.
#[derive(Debug, Clone)]
pub struct HrnBuilder {
    partition: Option<String>,
    service: Option<String>,
    region: Option<String>,
    account: Option<String>,
    resource_type: Option<String>,
    resource_path: Option<String>,
}

impl Default for HrnBuilder {
    /// The 0.1 derive default: no field is set and `build()` fails until the
    /// caller supplies the parts. Use [`HrnBuilder::new`] for the
    /// `hodei`/`global` defaults.
    fn default() -> Self {
        Self {
            partition: None,
            service: None,
            region: None,
            account: None,
            resource_type: None,
            resource_path: None,
        }
    }
}

impl HrnBuilder {
    pub fn new() -> Self {
        Self {
            partition: Some("hodei".to_string()),
            service: None,
            region: Some("global".to_string()),
            account: None,
            resource_type: None,
            resource_path: None,
        }
    }

    pub fn partition(mut self, partition: &str) -> Self {
        self.partition = Some(partition.to_string());
        self
    }

    pub fn service(mut self, service: &str) -> Self {
        self.service = Some(service.to_string());
        self
    }

    pub fn region(mut self, region: &str) -> Self {
        self.region = Some(region.to_string());
        self
    }

    /// Legacy name for the account field.
    pub fn tenant_id(self, tenant_id: &str) -> Self {
        self.account(tenant_id)
    }

    pub fn account(mut self, account: &str) -> Self {
        self.account = Some(account.to_string());
        self
    }

    /// Split a `type/path` resource string.
    pub fn resource(mut self, resource_path: &str) -> Result<Self, HrnError> {
        let (resource_type, resource_path) = resource_path
            .split_once('/')
            .ok_or(HrnError::InvalidResourcePart)?;
        self.resource_type = Some(resource_type.to_string());
        self.resource_path = Some(resource_path.to_string());
        Ok(self)
    }

    pub fn resource_type(mut self, resource_type: &str) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self
    }

    pub fn resource_path(mut self, resource_path: &str) -> Self {
        self.resource_path = Some(resource_path.to_string());
        self
    }

    pub fn build(self) -> Result<Hrn, HrnError> {
        let partition = self
            .partition
            .ok_or_else(|| HrnError::MissingPart("partition".to_string()))?;
        let service = self
            .service
            .ok_or_else(|| HrnError::MissingPart("service".to_string()))?;
        let region = self
            .region
            .ok_or_else(|| HrnError::MissingPart("region".to_string()))?;
        let account = self
            .account
            .ok_or_else(|| HrnError::MissingPart("tenant_id".to_string()))?;
        let resource_type = self
            .resource_type
            .ok_or_else(|| HrnError::MissingPart("resource_type".to_string()))?;
        let resource_path = self
            .resource_path
            .ok_or_else(|| HrnError::MissingPart("resource_id".to_string()))?;

        Hrn::new(
            &partition,
            &service,
            &region,
            &account,
            &resource_type,
            &resource_path,
        )
    }
}
