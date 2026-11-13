// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

//! Generate `tests/fixtures/lahiri-regression.json` from the library's own
//! `lahiri_ayanamsa` output.
//!
//! ```sh
//! cargo run --locked --example lahiri_regression_fixture -- \
//!     --validated-by /path/to/lahiri-report-final.json \
//!     --output tests/fixtures/lahiri-regression.json
//! ```
//!
//! The fixture is a *regression* fixture: it detects unintended changes, it does
//! not establish accuracy. Accuracy comes from the external benchmark, so this
//! generator refuses to run unless it is given a passing, final benchmark report
//! whose validated `src/` tree is identical to the current one. See
//! `tools/verification/conventions.md` ("Lahiri regression fixture").

use astrology_engine::{lahiri_ayanamsa, CalculationError, Epoch};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::process::Command;

const FORMAT: &str = "lahiri-regression/1";
const TOLERANCE_ARCSEC: f64 = 1e-6;
const POLICY: &str = "Regression fixture generated from this library's own lahiri_ayanamsa output. \
It detects unintended changes; it does not establish accuracy, which rests on the external benchmark \
recorded under `validation`. Do not regenerate it to make a failing test pass: a mismatch means the \
output changed, and the change must be explained and re-validated against the benchmark first.";

fn git(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Deterministic inputs: a 97-day grid across the supported range, a dense run in
/// 2026 (short-period nutation), fixed reference instants and range edges.
fn inputs() -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    let mut epoch = Epoch::from_gregorian_utc(1800, 1, 2, 0, 0, 0, 0);
    let end = Epoch::from_gregorian_utc(2199, 12, 1, 0, 0, 0, 0);
    while epoch <= end {
        out.push((epoch, "grid-97d"));
        epoch += hifitime::Duration::from_days(97.0);
    }
    let mut dense = Epoch::from_gregorian_utc(2026, 1, 1, 0, 0, 0, 0);
    for _ in 0..80 {
        out.push((dense, "dense-2026"));
        dense += hifitime::Duration::from_hours(8.88);
    }
    let fixed: [(i32, u8, u8, u8, u8, u8); 5] = [
        (2000, 1, 1, 12, 0, 0),     // J2000 (UTC)
        (2026, 12, 31, 23, 58, 51), // IAE 2027 table, 1 January, 5h29m IST ~ 0h TT
        (1956, 3, 21, 0, 0, 0),     // historical Lahiri definition date
        (1800, 1, 1, 0, 0, 0),      // lower edge: inside (library TT = UTC + delta-T)
        (2199, 12, 31, 0, 0, 0),    // upper edge: inside
    ];
    for (y, m, d, h, mi, s) in fixed {
        out.push((Epoch::from_gregorian_utc(y, m, d, h, mi, s, 0), "edge"));
    }
    let outside: [(i32, u8, u8, u8, u8, u8); 2] = [(1799, 12, 31, 23, 0, 0), (2200, 1, 1, 0, 0, 0)];
    for (y, m, d, h, mi, s) in outside {
        out.push((Epoch::from_gregorian_utc(y, m, d, h, mi, s, 0), "outside"));
    }
    out.sort_by_key(|a| a.0);
    out.dedup_by(|a, b| a.0 == b.0);
    out.into_iter()
        .map(|(e, kind)| (e.to_string(), kind))
        .collect()
}

fn validation(report_path: &str) -> Result<Value, String> {
    let bytes = std::fs::read(report_path).map_err(|e| format!("{report_path}: {e}"))?;
    let report: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{report_path}: {e}"))?;
    if report["status"] != "pass" || report["samples"]["final"] != true {
        return Err("the benchmark report must be a passing --final run".into());
    }
    for check in ["nutation", "ayanamsa", "iaeTable"] {
        if report["checks"][check].is_null() {
            return Err(format!("the benchmark report lacks the {check} check"));
        }
    }
    let revision = report["library"]["revision"]
        .as_str()
        .ok_or("report lacks library.revision")?;
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("report library.revision must be a full Git commit hash".into());
    }
    git(&["rev-parse", "--verify", &format!("{revision}^{{commit}}")])?;
    // git diff omits untracked files, including a newly added runtime module.
    if !git(&["ls-files", "--others", "--", "src"])?.is_empty() {
        return Err(
            "src/ contains untracked files; commit and validate the complete source first".into(),
        );
    }
    // The validated source must be exactly the source we are about to fixture.
    if !git(&["diff", "--name-only", revision, "--", "src"])?.is_empty() {
        return Err(format!(
            "src/ differs from the validated revision {revision}; re-run the benchmark first"
        ));
    }
    Ok(json!({
        "report": std::path::Path::new(report_path).file_name().and_then(|n| n.to_str()),
        "reportSha256": format!("{:x}", Sha256::digest(&bytes)),
        "status": report["status"],
        "validatedRevision": revision,
        "samples": report["samples"]["count"],
        "holdoutSeed": report["samples"]["holdoutSeed"],
        "benchmark": {
            "definition": report["benchmark"]["reference"]["definition"],
            "nutation": report["benchmark"]["reference"]["nutation"],
            "novasSdistSha256": report["benchmark"]["provenance"]["novasSdist"],
            "libnovasSha256": report["benchmark"]["provenance"]["libnovas"],
        },
        "maxAbsArcsec": report["checks"]["ayanamsa"]["value"]["maxAbs"],
        "iaeTable": format!("{}/{}", report["checks"]["iaeTable"]["roundsToTable"], report["checks"]["iaeTable"]["of"]),
    }))
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut report, mut output) = (None, None);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--validated-by" => report = args.next(),
            "--output" => output = args.next(),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let report = report.ok_or("--validated-by <benchmark report> is required")?;
    let output = output.ok_or("--output <path> is required")?;
    let validation = validation(&report)?;

    let mut cases = Vec::new();
    for (utc, kind) in inputs() {
        let epoch = Epoch::from_gregorian_str(&utc).map_err(|e| format!("{utc}: {e}"))?;
        let case = match (lahiri_ayanamsa(epoch), kind) {
            (Ok(value), "outside") => {
                return Err(format!("{utc} should be outside the range, got {value}"))
            }
            (Ok(value), _) if !value.is_finite() => {
                return Err(format!("{utc}: non-finite {value}"))
            }
            (Ok(value), _) => json!({"utc": utc, "kind": kind, "ayanamsaDeg": value}),
            (Err(CalculationError::InvalidInput(message)), "outside") => {
                json!({"utc": utc, "kind": kind, "error": "InvalidInput", "message": message})
            }
            (Err(error), _) => return Err(format!("{utc}: unexpected error {error:?}")),
        };
        cases.push(case);
    }
    let fixture = json!({
        "format": FORMAT,
        "policy": POLICY,
        "definition": {
            "function": "astrology_engine::lahiri_ayanamsa (degrees)",
            "mean": "IAE 2027 p. 382, transcribed literally (including the printed T^4 coefficient)",
            "nutation": "IAU 2000B, complete 77-term series (table from NOVAS C 3.1)",
            "input": "UTC strings as accepted by Epoch::from_gregorian_str",
        },
        "toleranceArcsec": TOLERANCE_ARCSEC,
        "generator": {
            "command": "cargo run --locked --example lahiri_regression_fixture -- --validated-by <report> --output tests/fixtures/lahiri-regression.json",
            // Last commit that changed src/, not HEAD: stable across unrelated commits,
            // so regeneration from unchanged source is byte-identical.
            "sourceRevision": git(&["log", "-1", "--format=%H", "--", "src"])?,
        },
        "validation": validation,
        "cases": cases,
    });
    let mut text = serde_json::to_string_pretty(&fixture).map_err(|e| e.to_string())?;
    text.push('\n');
    std::fs::write(&output, text).map_err(|e| format!("{output}: {e}"))?;
    println!(
        "wrote {output}: {} cases",
        fixture["cases"].as_array().map_or(0, Vec::len)
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
