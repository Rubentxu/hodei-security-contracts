//! Compatibility module for the 0.1 `hodei_hrn::api` path.
//!
//! The canonical items now live at the crate root; this module keeps existing
//! imports compiling without reintroducing public fields.

pub use crate::builder::HrnBuilder;
pub use crate::error::HrnError;
pub use crate::hrn::Hrn;
pub use crate::legacy::{LegacyHrnFields, MigrationContext};
