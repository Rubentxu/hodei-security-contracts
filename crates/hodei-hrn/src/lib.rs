//! Canonical Hodei Resource Names.
//!
//! This crate is the executable reference for the HRN contract. It deliberately
//! has no IO, no Cedar and no Tokio: resource identity is a pure contract.
//!
//! Resource HRNs identify resources; actions use [`ActionId`] and are never
//! represented as an [`Hrn`].

#![forbid(unsafe_code)]

pub mod action;
pub mod api;
pub mod builder;
pub mod error;
pub mod hrn;
pub mod legacy;
pub mod migration;

pub use action::{ActionId, MAX_ACTION_INPUT_LEN};
pub use builder::HrnBuilder;
pub use error::{ActionIdError, HrnError};
pub use hrn::{
    Hrn, MAX_ACCOUNT_LEN, MAX_HRN_LEN, MAX_PARTITION_LEN, MAX_REGION_LEN, MAX_RESOURCE_PATH_LEN,
    MAX_RESOURCE_TYPE_LEN, MAX_SERVICE_LEN,
};
pub use legacy::{LegacyHrnFields, MigrationContext};
pub use migration::{MigrationClass, Origin, classify};
