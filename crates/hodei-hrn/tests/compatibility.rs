//! Compatibility tests for the published 0.1 API.

use hodei_hrn::legacy::MigrationContext;
use hodei_hrn::migration::Origin;
use hodei_hrn::{Hrn, HrnBuilder, HrnError, LegacyHrnFields};

#[test]
fn builder_keeps_the_legacy_methods_and_defaults() {
    let hrn = Hrn::builder()
        .service("documents-api")
        .tenant_id("tenant-a")
        .resource("document/doc-1")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(hrn.partition(), "hodei");
    assert_eq!(hrn.service(), "documents-api");
    assert_eq!(hrn.region(), "global");
    assert_eq!(hrn.account(), "tenant-a");
    assert_eq!(hrn.tenant_id(), "tenant-a");
    assert_eq!(hrn.resource_type(), "document");
    assert_eq!(hrn.resource_path(), "doc-1");
    assert_eq!(hrn.resource_id(), "doc-1");
    assert_eq!(
        hrn.to_string(),
        "hrn:hodei:documents-api:global:tenant-a:document/doc-1"
    );
}

#[test]
fn builder_can_select_the_canonical_empty_region() {
    let hrn = Hrn::builder()
        .service("artifacts")
        .region("")
        .account("acme")
        .resource("repository/releases")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(hrn.region(), "");
    assert_eq!(
        hrn.to_string(),
        "hrn:hodei:artifacts::acme:repository/releases"
    );
}

#[test]
fn from_string_is_the_option_compatibility_parser() {
    let hrn =
        Hrn::from_string("hrn:hodei:users-api:global:tenant-b:user/alice").expect("legacy parser");
    assert_eq!(hrn.tenant_id(), "tenant-b");
    assert_eq!(hrn.resource_id(), "alice");

    assert!(Hrn::from_string("not-an-hrn").is_none());
}

#[test]
fn new_canonicalizes_service_and_validates_every_part() {
    let hrn = Hrn::new(
        "hodei",
        "Artifacts",
        "eu-west-1",
        "acme",
        "repository",
        "releases/v2",
    )
    .unwrap();

    assert_eq!(hrn.service(), "artifacts");
    assert_eq!(hrn.resource_path(), "releases/v2");
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "a/b", "c").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "", "repository", "r").is_err());
}

#[test]
fn builder_reports_missing_parts_with_the_legacy_names() {
    let error = Hrn::builder()
        .service("svc")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap_err();
    assert!(matches!(error, HrnError::MissingPart(part) if part == "tenant_id"));

    let error = Hrn::builder().service("svc").build().unwrap_err();
    assert!(matches!(
        error,
        HrnError::InvalidResourcePart
            | HrnError::MissingPart(_)
            | HrnError::EmptyField { .. }
            | HrnError::InvalidField { .. }
    ));
}

#[test]
fn builder_rejects_a_resource_without_a_slash() {
    let error = Hrn::builder()
        .service("svc")
        .tenant_id("t1")
        .resource("user")
        .unwrap_err();
    assert!(matches!(error, HrnError::InvalidResourcePart));
}

#[test]
fn new_rejects_separator_injection_and_actions() {
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "a:b", "c").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "Action", "read").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "repository", "a\u{1}").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "repository", "../admin").is_err());
}

#[test]
fn empty_region_alias_compares_with_the_parsed_form() {
    let built = Hrn::builder()
        .service("artifacts")
        .region("")
        .account("acme")
        .resource("repository/releases")
        .unwrap()
        .build()
        .unwrap();
    let parsed = Hrn::parse("hrn:hodei:artifacts::acme:repository/releases").unwrap();
    assert_eq!(built, parsed);
}

#[test]
fn path_prefix_is_not_descendancy() {
    let ancestor = Hrn::parse("hrn:hodei:artifacts:eu:acme:repository/releases").unwrap();
    let prefix_trap = Hrn::parse("hrn:hodei:artifacts:eu:acme:repository/releases-old").unwrap();
    let child = Hrn::parse("hrn:hodei:artifacts:eu:acme:repository/releases/v2").unwrap();

    assert!(!prefix_trap.is_descendant_of(&ancestor));
    assert!(child.is_descendant_of(&ancestor));
}

#[test]
fn serde_uses_the_canonical_string() {
    let hrn = Hrn::builder()
        .service("svc")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();

    let json = serde_json::to_string(&hrn).unwrap();
    assert_eq!(json, "\"hrn:hodei:svc:global:t1:user/1\"");
    let back: Hrn = serde_json::from_str(&json).unwrap();
    assert_eq!(back, hrn);
}

#[test]
fn constructor_rejects_separators_in_every_field() {
    assert!(Hrn::new("ho:dei", "artifacts", "eu", "acme", "repository", "r").is_err());
    assert!(Hrn::new("hodei", "arti:facts", "eu", "acme", "repository", "r").is_err());
    assert!(Hrn::new("hodei", "artifacts", "e:u", "acme", "repository", "r").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "ac:me", "repository", "r").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "repo:sitory", "r").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "repository", "r:s").is_err());
}

#[test]
fn constructor_and_builder_reject_actions_and_dirty_paths() {
    assert!(Hrn::new("hodei", "iam", "eu", "acme", "Action", "read").is_err());
    assert!(
        Hrn::builder()
            .service("iam")
            .account("acme")
            .resource("Action/read")
            .unwrap()
            .build()
            .is_err()
    );

    for resource in [
        "repository/../admin",
        "repository//x",
        "repository/.",
        "repository/",
    ] {
        assert!(
            Hrn::builder()
                .service("artifacts")
                .account("acme")
                .resource(resource)
                .unwrap()
                .build()
                .is_err(),
            "resource {resource} must be rejected"
        );
    }
}

#[test]
fn from_string_rejects_invalid_fields_without_an_error_type() {
    assert!(Hrn::from_string("hrn:hodei:artifacts:eu:acme:repository/..").is_none());
    assert!(Hrn::from_string("hrn:hodei:artifacts:eu:acme:repository/a\u{1}").is_none());
}

#[test]
fn constructor_rejects_an_over_long_input_before_formatting() {
    let huge = "a".repeat(2000);
    assert!(matches!(
        Hrn::new("hodei", &huge, "eu", "acme", "repository", "r"),
        Err(HrnError::TooLong(_))
    ));
}

#[test]
fn api_module_path_is_still_available() {
    let hrn: hodei_hrn::api::Hrn = "hrn:hodei:artifacts:eu:acme:repository/r".parse().unwrap();
    assert_eq!(hrn.account(), "acme");
}

#[test]
fn legacy_json_object_can_be_converted_explicitly() {
    let json = r#"{
        "partition": "hodei",
        "service": "artifacts",
        "region": "global",
        "tenant_id": "acme",
        "resource_type": "repository",
        "resource_id": "releases"
    }"#;
    let fields: LegacyHrnFields = serde_json::from_str(json).unwrap();
    let hrn = fields
        .into_canonical(MigrationContext::new(Origin::ArtifactsV2))
        .unwrap();
    assert_eq!(
        hrn.to_string(),
        "hrn:hodei:artifacts:global:acme:repository/releases"
    );

    let back = LegacyHrnFields::from(&hrn);
    assert_eq!(back.tenant_id, "acme");
    assert_eq!(back.resource_id, "releases");
}

#[test]
fn nested_resource_id_is_the_whole_path() {
    let hrn = Hrn::builder()
        .service("api")
        .tenant_id("tenant-123")
        .resource("document/folder/subfolder/file.txt")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(hrn.resource_type(), "document");
    assert_eq!(hrn.resource_id(), "folder/subfolder/file.txt");
    assert_eq!(hrn.resource_path(), "folder/subfolder/file.txt");
}

#[test]
fn builder_display_round_trips_through_from_str() {
    let hrn = Hrn::builder()
        .service("svc")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();
    let parsed: Hrn = hrn.to_string().parse().unwrap();
    assert_eq!(parsed, hrn);
}

#[test]
fn default_builder_is_empty_and_new_has_legacy_defaults() {
    assert!(HrnBuilder::default().build().is_err());

    let hrn = HrnBuilder::new()
        .service("svc")
        .tenant_id("t1")
        .resource("user/1")
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(hrn.partition(), "hodei");
    assert_eq!(hrn.region(), "global");
}

#[test]
fn constructor_rejects_invalid_partition_and_region() {
    assert!(Hrn::new("Hodei", "artifacts", "eu", "acme", "repository", "r").is_err());
    assert!(Hrn::new("hodei", "artifacts", "EU", "acme", "repository", "r").is_err());
    assert!(Hrn::new("hodei", "artifacts", "eu", "acme", "repository", "..").is_err());
}

#[test]
fn legacy_field_conversion_requires_an_explicit_non_ambiguous_origin() {
    let fields = LegacyHrnFields {
        partition: "hodei".to_string(),
        service: "iam".to_string(),
        region: String::new(),
        tenant_id: "tenant-a".to_string(),
        resource_type: "User".to_string(),
        resource_id: "alice".to_string(),
    };

    let canonical = fields
        .clone()
        .into_canonical(MigrationContext::new(Origin::ArtifactsV1))
        .unwrap();
    assert_eq!(canonical.account(), "tenant-a");

    let mut v1_with_region = fields.clone();
    v1_with_region.region = "global".to_string();
    assert_eq!(
        v1_with_region
            .into_canonical(MigrationContext::new(Origin::ArtifactsV1))
            .unwrap_err(),
        HrnError::AmbiguousOrigin,
        "ArtifactsV1 with a non-empty region is ambiguous per the migration matrix"
    );

    for origin in [Origin::HodeiAuthzV1, Origin::AuditTrail, Origin::Unknown] {
        assert_eq!(
            fields
                .clone()
                .into_canonical(MigrationContext::new(origin))
                .unwrap_err(),
            HrnError::AmbiguousOrigin,
            "origin {origin:?} must not imply tenant_id == account"
        );
    }
}
