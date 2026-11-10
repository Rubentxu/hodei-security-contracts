//! Executable conformance for the canonical HRN contract.

use std::collections::BTreeSet;

use hodei_hrn::{ActionId, Hrn, HrnError, MigrationClass, Origin, classify};
use serde_json::Value;

const VECTORS: &str = include_str!("fixtures/vectors.json");

fn document() -> Value {
    serde_json::from_str(VECTORS).expect("vectors.json must be valid JSON")
}

#[test]
fn vector_document_has_a_stable_shape_and_categories() {
    let doc = document();
    assert_eq!(doc["schemaVersion"], 1);
    assert_eq!(doc["kind"], "hodei-hrn-contract-vectors");
    assert_eq!(doc["contract"], "hrn:v2");

    let vectors = doc["vectors"].as_array().expect("vectors array");
    assert!(vectors.len() >= 35, "the contract must cover enough cases");

    let mut ids = BTreeSet::new();
    let mut syntaxes = BTreeSet::new();
    let mut migrations = BTreeSet::new();
    for vector in vectors {
        let id = vector["id"].as_str().expect("vector id");
        assert!(ids.insert(id.to_string()), "duplicate vector id: {id}");
        syntaxes.insert(vector["syntax"].as_str().unwrap());
        migrations.insert(vector["migration"].as_str().unwrap());
    }

    assert!(syntaxes.contains("valid"));
    assert!(syntaxes.contains("invalid"));
    for expected in [
        "canonical",
        "alias",
        "canonicalization",
        "rejected",
        "ambiguous",
        "action",
    ] {
        assert!(
            migrations.contains(expected),
            "missing migration label {expected}"
        );
    }
}

#[test]
fn resource_vectors_match_the_parser_and_origin_policy() {
    let doc = document();
    for vector in doc["vectors"].as_array().unwrap() {
        let id = vector["id"].as_str().unwrap();
        let input = vector["input"].as_str().unwrap();
        let origin = Origin::from_label(vector["origin"].as_str().unwrap())
            .unwrap_or_else(|| panic!("{id}: unknown origin label"));

        match vector["syntax"].as_str().unwrap() {
            "valid" => {
                let hrn = Hrn::parse(input).unwrap_or_else(|error| {
                    panic!("{id}: expected valid input {input:?}, got {error}")
                });

                let parsed = &vector["parsed"];
                assert_eq!(
                    hrn.partition(),
                    parsed["partition"].as_str().unwrap(),
                    "{id}"
                );
                assert_eq!(hrn.service(), parsed["service"].as_str().unwrap(), "{id}");
                assert_eq!(hrn.region(), parsed["region"].as_str().unwrap(), "{id}");
                assert_eq!(hrn.account(), parsed["account"].as_str().unwrap(), "{id}");
                assert_eq!(
                    hrn.resource_type(),
                    parsed["resource_type"].as_str().unwrap(),
                    "{id}"
                );
                assert_eq!(
                    hrn.resource_path(),
                    parsed["resource_path"].as_str().unwrap(),
                    "{id}"
                );
                assert_eq!(
                    hrn.to_string(),
                    vector["canonical"].as_str().unwrap(),
                    "{id}: canonical form"
                );
                let json = serde_json::to_string(&hrn).unwrap();
                let round_trip: Hrn = serde_json::from_str(&json).unwrap();
                assert_eq!(round_trip, hrn, "{id}: serde round-trip");

                if vector["migration"] == "ambiguous" {
                    assert!(
                        matches!(classify(origin, input), Err(HrnError::AmbiguousOrigin)),
                        "{id}: an ambiguous layout must not become a canonical HRN without provenance"
                    );
                } else {
                    let (_, class) = classify(origin, input).unwrap_or_else(|error| {
                        panic!("{id}: origin classification failed: {error}")
                    });
                    match vector["migration"].as_str().unwrap() {
                        "canonical" => assert_eq!(class, MigrationClass::Canonical, "{id}"),
                        "alias" | "canonicalization" => {
                            assert_eq!(class, MigrationClass::Alias, "{id}")
                        }
                        other => panic!("{id}: unexpected accepted migration label {other}"),
                    }
                }
            }
            "invalid" => {
                let error = Hrn::parse(input)
                    .err()
                    .unwrap_or_else(|| panic!("{id}: expected invalid input {input:?}"));

                if let Some(expected) = vector["error"].as_str() {
                    assert_eq!(error_name(&error), expected, "{id}: error taxonomy");
                }
                if let Some(expected) = vector["errorField"].as_str() {
                    let field = match &error {
                        HrnError::EmptyField { field } | HrnError::InvalidField { field, .. } => {
                            *field
                        }
                        other => panic!("{id}: expected a field error, got {other}"),
                    };
                    assert_eq!(field, expected, "{id}: error field");
                }

                if vector["migration"] == "action" {
                    let action = ActionId::from_legacy_hrn(input)
                        .unwrap_or_else(|error| panic!("{id}: action migration failed: {error}"));
                    assert_eq!(
                        action.to_cedar_uid(),
                        vector["actionUid"].as_str().unwrap(),
                        "{id}: action UID"
                    );
                }
            }
            other => panic!("{id}: unknown syntax outcome {other}"),
        }
    }
}

fn error_name(error: &HrnError) -> &'static str {
    match error {
        HrnError::NonAscii => "NonAscii",
        HrnError::TooLong(_) => "TooLong",
        HrnError::InvalidPrefix => "InvalidPrefix",
        HrnError::InvalidComponentCount(_) => "InvalidComponentCount",
        HrnError::EmptyField { .. } => "EmptyField",
        HrnError::InvalidField { .. } => "InvalidField",
        HrnError::ActionIsNotResource => "ActionIsNotResource",
        HrnError::AmbiguousOrigin => "AmbiguousOrigin",
        _ => "Unknown",
    }
}

#[test]
fn relations_match_equality_descendancy_and_parents() {
    let doc = document();
    for relation in doc["relations"].as_array().unwrap() {
        let id = relation["id"].as_str().unwrap();
        match relation["kind"].as_str().unwrap() {
            "equality" => {
                let left = Hrn::parse(relation["left"].as_str().unwrap()).unwrap();
                let right = Hrn::parse(relation["right"].as_str().unwrap()).unwrap();
                assert_eq!(
                    left == right,
                    relation["expected"].as_bool().unwrap(),
                    "{id}"
                );
            }
            "descendant" => {
                let ancestor = Hrn::parse(relation["ancestor"].as_str().unwrap()).unwrap();
                let descendant = Hrn::parse(relation["descendant"].as_str().unwrap()).unwrap();
                assert_eq!(
                    descendant.is_descendant_of(&ancestor),
                    relation["expected"].as_bool().unwrap(),
                    "{id}"
                );
            }
            "parent" => {
                let child = Hrn::parse(relation["child"].as_str().unwrap()).unwrap();
                let expected = relation["parent"].as_str().map(str::to_string);
                assert_eq!(
                    child.parent().map(|parent| parent.to_string()),
                    expected,
                    "{id}"
                );
            }
            other => panic!("{id}: unknown relation kind {other}"),
        }
    }
}

#[test]
fn actions_and_resources_have_distinct_identities() {
    let legacy = "hrn:aws:iam:::Action/read";
    assert!(
        Hrn::parse(legacy).is_err(),
        "an action must never parse as a resource"
    );

    let action = ActionId::from_legacy_hrn(legacy).expect("legacy action migration");
    assert_eq!(action.service(), "iam");
    assert_eq!(action.name(), "read");
    assert_eq!(action.to_cedar_uid(), "Iam::Action::\"read\"");

    assert!(
        ActionId::from_legacy_hrn("hrn:hodei:iam::acme:Action/read").is_err(),
        "a scoped action is not migrated silently"
    );
}

#[test]
fn action_namespace_is_injective() {
    let hyphen = ActionId::new("a-1", "read").unwrap();
    let digit = ActionId::new("a1", "read").unwrap();
    assert_ne!(hyphen.to_cedar_uid(), digit.to_cedar_uid());

    let single = ActionId::new("foo-bar", "read").unwrap();
    let double = ActionId::new("foo--bar", "read").unwrap();
    assert_ne!(single.to_cedar_uid(), double.to_cedar_uid());

    assert_eq!(
        ActionId::new("audit-trail", "read").unwrap().to_cedar_uid(),
        "Audit_trail::Action::\"read\""
    );
    assert!(ActionId::new("9a", "read").is_err());
}

#[test]
fn action_uid_parser_accepts_only_the_canonical_form() {
    assert!(ActionId::from_cedar_uid("IAM::Action::\"read\"").is_err());
    assert!(ActionId::from_cedar_uid("Iam-User::Action::\"read\"").is_err());

    for service in ["a-1", "a1", "foo--bar", "audit-trail"] {
        let action = ActionId::new(service, "read").unwrap();
        assert_eq!(
            ActionId::from_cedar_uid(&action.to_cedar_uid()).unwrap(),
            action
        );
    }
}

#[test]
fn action_inputs_are_bounded_and_errors_are_escaped() {
    let long = "a".repeat(10_000);
    assert!(ActionId::new(&long, "read").is_err());

    let error = ActionId::new("IAM\n", "read").unwrap_err();
    assert!(!error.to_string().contains('\n'));
    assert!(error.to_string().contains("\\n"));
}

#[test]
fn actions_round_trip_through_cedar_uid_and_serde() {
    let action = ActionId::new("audit-trail", "read").unwrap();
    let uid = action.to_cedar_uid();
    assert_eq!(ActionId::from_cedar_uid(&uid).unwrap(), action);
    assert_eq!(uid.parse::<ActionId>().unwrap(), action);

    let json = serde_json::to_string(&action).unwrap();
    let back: ActionId = serde_json::from_str(&json).unwrap();
    assert_eq!(back, action);
}

#[test]
fn canonical_strings_round_trip_through_serde() {
    let hrn = Hrn::parse("hrn:hodei:artifacts:eu-west-1:acme:repository/releases").unwrap();
    let json = serde_json::to_string(&hrn).unwrap();
    assert_eq!(
        json,
        "\"hrn:hodei:artifacts:eu-west-1:acme:repository/releases\""
    );
    let back: Hrn = serde_json::from_str(&json).unwrap();
    assert_eq!(back, hrn);
}

#[test]
fn serde_rejects_invalid_hrns() {
    assert!(serde_json::from_str::<Hrn>("\"not-an-hrn\"").is_err());
}

#[test]
fn over_long_inputs_are_rejected_before_validation() {
    let path = "a".repeat(1500);
    let input = format!("hrn:hodei:artifacts:eu:acme:repository/{path}");
    assert!(matches!(Hrn::parse(&input), Err(HrnError::TooLong(_))));

    let action = format!("hrn:aws:iam:::Action/{}", "a".repeat(600));
    assert!(ActionId::from_legacy_hrn(&action).is_err());
}

#[test]
fn length_boundary_is_exact() {
    let resource_type = format!("T{}", "t".repeat(127));
    let input = format!(
        "hrn:{}:{}:{}:{}:{}/{}",
        "p".repeat(64),
        "s".repeat(64),
        "r".repeat(64),
        "a".repeat(128),
        resource_type,
        "a".repeat(1024),
    );
    assert_eq!(input.len(), 1481);
    assert!(Hrn::parse(&input).is_ok(), "1481 bytes must be accepted");

    let over = format!("{input}a");
    assert!(matches!(Hrn::parse(&over), Err(HrnError::TooLong(_))));
}

#[test]
fn control_characters_are_escaped_in_errors() {
    let error = Hrn::parse("hrn:hodei:artifacts:eu:acme:repository/a\nb").unwrap_err();
    let rendered = error.to_string();
    assert!(
        !rendered.contains('\n'),
        "error must not contain a raw newline"
    );
    assert!(
        rendered.contains("\\n"),
        "error must escape the newline: {rendered}"
    );
}

#[test]
fn per_field_limits_are_enforced() {
    fn value(
        partition: &str,
        service: &str,
        region: &str,
        account: &str,
        kind: &str,
        path: &str,
    ) -> String {
        format!("hrn:{partition}:{service}:{region}:{account}:{kind}/{path}")
    }

    assert!(Hrn::parse(&value(&"p".repeat(64), "s", "r", "a", "T", "x")).is_ok());
    assert!(Hrn::parse(&value(&"p".repeat(65), "s", "r", "a", "T", "x")).is_err());
    assert!(Hrn::parse(&value("p", &"s".repeat(64), "r", "a", "T", "x")).is_ok());
    assert!(Hrn::parse(&value("p", &"s".repeat(65), "r", "a", "T", "x")).is_err());
    assert!(Hrn::parse(&value("p", "s", &"r".repeat(64), "a", "T", "x")).is_ok());
    assert!(Hrn::parse(&value("p", "s", &"r".repeat(65), "a", "T", "x")).is_err());
    assert!(Hrn::parse(&value("p", "s", "r", &"a".repeat(128), "T", "x")).is_ok());
    assert!(Hrn::parse(&value("p", "s", "r", &"a".repeat(129), "T", "x")).is_err());
    assert!(
        Hrn::parse(&value(
            "p",
            "s",
            "r",
            "a",
            &format!("T{}", "t".repeat(127)),
            "x"
        ))
        .is_ok()
    );
    assert!(
        Hrn::parse(&value(
            "p",
            "s",
            "r",
            "a",
            &format!("T{}", "t".repeat(128)),
            "x"
        ))
        .is_err()
    );
    assert!(Hrn::parse(&value("p", "s", "r", "a", "T", &"x".repeat(1024))).is_ok());
    assert!(Hrn::parse(&value("p", "s", "r", "a", "T", &"x".repeat(1025))).is_err());
}
