# Changelog

## 0.2.0 — HRN v2 contract

- Canonical parsing and serialization of `hrn:<partition>:<service>:<region>:<account>:<type>/<path>`.
- Strict field validation, exact limits and segment-based descendancy.
- `tenant_id`/`resource_id` compatibility aliases and `HrnBuilder` with the 0.1 defaults.
- Origin-aware `classify` rejects ambiguous legacy layouts without provenance.
- `ActionId` is a separate identity with an injective Cedar namespace.
- **Breaking:** public fields are now private; serde uses the canonical string; the `sqlx` feature is removed. See `MIGRATION.md`.
