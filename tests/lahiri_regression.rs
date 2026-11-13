// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

//! Lahiri regression fixture: the public `lahiri_ayanamsa` must reproduce the
//! values recorded in `tests/fixtures/lahiri-regression.json` within 1 µas.
//!
//! This detects regressions; it does not establish accuracy. See the fixture's
//! `policy` field and `tools/verification/conventions.md` ("Lahiri regression
//! fixture") before regenerating it.

use astrology_engine::{lahiri_ayanamsa, CalculationError, Epoch};
use serde_json::Value;
use std::collections::HashSet;

const FIXTURE: &str = include_str!("fixtures/lahiri-regression.json");
const MIN_CASES: usize = 1_500;

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).expect("fixture is valid JSON")
}

#[test]
fn fixture_is_well_formed() {
    let fixture = fixture();
    assert_eq!(fixture["format"], "lahiri-regression/1");
    assert!(fixture["policy"]
        .as_str()
        .is_some_and(|p| p.contains("does not establish accuracy")));
    assert_eq!(fixture["toleranceArcsec"].as_f64(), Some(1e-6));
    assert_eq!(fixture["validation"]["status"], "pass");
    let cases = fixture["cases"].as_array().expect("cases array");
    assert!(cases.len() >= MIN_CASES, "only {} cases", cases.len());

    let mut seen = HashSet::new();
    let mut outside = 0;
    for case in cases {
        let utc = case["utc"].as_str().expect("utc string");
        assert!(seen.insert(utc), "duplicate input {utc}");
        match (case.get("ayanamsaDeg"), case.get("error")) {
            (Some(value), None) => assert!(
                value.as_f64().is_some_and(f64::is_finite),
                "{utc}: missing or non-finite value"
            ),
            (None, Some(error)) => {
                assert_eq!(error, "InvalidInput", "{utc}");
                assert_eq!(
                    case["kind"], "outside",
                    "{utc}: only out-of-range cases may error"
                );
                outside += 1;
            }
            _ => panic!("{utc}: a case needs exactly one of ayanamsaDeg or error"),
        }
    }
    assert_eq!(outside, 2, "both ends of the range must be pinned");
}

#[test]
fn lahiri_ayanamsa_matches_the_regression_fixture() {
    let fixture = fixture();
    let tolerance_deg = fixture["toleranceArcsec"].as_f64().unwrap() / 3600.0;
    let mut failures = Vec::new();
    let mut worst: f64 = 0.0;
    let cases = fixture["cases"].as_array().unwrap();
    for case in cases {
        let utc = case["utc"].as_str().unwrap();
        let epoch = Epoch::from_gregorian_str(utc).unwrap_or_else(|e| panic!("{utc}: {e}"));
        match (lahiri_ayanamsa(epoch), case["ayanamsaDeg"].as_f64()) {
            (Ok(got), Some(want)) => {
                let diff = (got - want).abs();
                worst = worst.max(diff);
                // A NaN difference must fail, not slip past the comparison.
                if diff.is_nan() || diff > tolerance_deg {
                    failures.push(format!(
                        "{utc}: got {got}, fixture {want} ({:.3e}\")",
                        diff * 3600.0
                    ));
                }
            }
            (Err(CalculationError::InvalidInput(_)), None) => {}
            (got, want) => failures.push(format!("{utc}: got {got:?}, fixture {want:?}")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases differ (worst {:.3e}\"):\n{}",
        failures.len(),
        cases.len(),
        worst * 3600.0,
        failures
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
