#[path = "../tools/verification/common/mod.rs"]
mod common;
use astrology_engine::{
    calculate_chart, coverage, effective_configuration, lahiri_ayanamsa, ChartInput, ChartValues,
    Ephemeris, Epoch, HouseSystem, Location, SignDivisions, ZodiacConfiguration, ZodiacReference,
};

#[test]
fn lahiri_wire_configuration_and_divisions() {
    let config: ZodiacConfiguration =
        serde_json::from_str(r#"{"reference":"lahiri","divisions":"equal","ophiuchus":null}"#)
            .unwrap();
    assert_eq!(config.reference, ZodiacReference::Lahiri);
    assert_eq!(effective_configuration(config).unwrap(), config);
    assert_eq!(serde_json::to_value(config).unwrap()["reference"], "lahiri");
    assert!(effective_configuration(ZodiacConfiguration {
        divisions: SignDivisions::Constellation,
        ..config
    })
    .is_err());
}

fn chart(ephemeris: &Ephemeris, epoch: Epoch, reference: ZodiacReference) -> ChartValues {
    calculate_chart(
        ephemeris,
        ChartInput {
            epoch,
            location: Location {
                latitude: 51.5,
                longitude: 0.0,
            },
            house_system: HouseSystem::Equal,
            zodiac: ZodiacConfiguration {
                reference,
                divisions: SignDivisions::Equal,
                ophiuchus: None,
            },
        },
    )
    .unwrap()
}

/// Signed difference `a - b` in arcseconds, wrapped to (-180°, 180°].
fn wrapped_arcsec(a: f64, b: f64) -> f64 {
    ((a - b + 180.0).rem_euclid(360.0) - 180.0) * 3600.0
}

#[test]
fn charts_report_lahiri_ayanamsa_and_project_within_the_documented_bound() {
    let ephemeris = Ephemeris::parse(common::polynomial_blob()).unwrap();
    let window = coverage(&ephemeris);
    for et in [window.lo_et, 0.0, window.hi_et] {
        let epoch = Epoch::from_et_seconds(et);
        let ayanamsa = lahiri_ayanamsa(epoch).unwrap();
        let tropical = chart(&ephemeris, epoch, ZodiacReference::Tropical);
        let sidereal = chart(&ephemeris, epoch, ZodiacReference::Lahiri);
        assert_eq!(sidereal.reference_offset_degrees, ayanamsa);
        let positions = tropical.bodies.iter().zip(&sidereal.bodies);
        for (t, s) in positions {
            let gap = wrapped_arcsec(t.values.longitude - s.values.longitude, ayanamsa);
            assert!(gap.abs() < 0.27, "{}: {gap}", t.values.name);
        }
        for (t, s) in tropical.cusps.iter().zip(&sidereal.cusps) {
            assert!(wrapped_arcsec(t - s, ayanamsa).abs() < 0.27);
        }
    }
}

/// The documented Lahiri exception, exactly: `tropical − sidereal` equals the
/// reported ayanamsa minus the nutation the engine's five-term series drops.
#[cfg(feature = "generation")]
#[test]
fn lahiri_projection_gap_is_exactly_the_dropped_nutation() {
    use astrology_engine::tooling::{frames::e_tilt, nutation::iau2000b_full, time::AstroTime};
    let ephemeris = Ephemeris::parse(common::polynomial_blob()).unwrap();
    let window = coverage(&ephemeris);
    for et in [window.lo_et, 0.0, window.hi_et] {
        let epoch = Epoch::from_et_seconds(et);
        let time = AstroTime::from_tdb(epoch);
        let dropped = (iau2000b_full(time).dpsi - e_tilt(time).dpsi) / 3600.0;
        let tropical = chart(&ephemeris, epoch, ZodiacReference::Tropical);
        let sidereal = chart(&ephemeris, epoch, ZodiacReference::Lahiri);
        let projection = sidereal.reference_offset_degrees - dropped;
        for (t, s) in tropical.bodies.iter().zip(&sidereal.bodies) {
            let gap = wrapped_arcsec(t.values.longitude - s.values.longitude, projection);
            assert!(gap.abs() < 1.0e-9, "{}: {gap}", t.values.name);
        }
    }
}
