// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

//! Fagan–Bradley and current PAC Lahiri longitude offsets.
//!
//! This is the traditional longitude-offset convention on the ecliptic of
//! date, not a three-dimensional rotation onto a fixed B1950 ecliptic.
//!
//! Lahiri has two offsets. The *reported* true ayanamsa is the IAE mean plus the
//! complete IAU 2000B nutation in longitude, as the IAE tabulates it. The
//! *projection* offset uses the engine's five-term nutation instead, because the
//! dataset's tropical longitudes carry that same truncated nutation; subtracting
//! it makes nutation cancel, so sidereal longitudes stay nutation-free. The two
//! differ by at most 0.27″ over the supported range.
use crate::astro::frames::{e_tilt, precession_matrix};
use crate::astro::nutation::iau2000b_full;
use crate::astro::AstroTime;
use crate::{CalculationError, Epoch};

const J2000: f64 = 2_451_545.0;
const B1950: f64 = 2_433_282.423_459_05;
const B1950_OFFSET: f64 = 24.0 + 2.0 / 60.0 + 31.36 / 3600.0;
const MODERN_PRECESSION_CORRECTION: f64 = 0.41256 / 3600.0;
const MIN_JD: f64 = 2_378_496.5; // 1800-01-01 TT
const MAX_JD: f64 = 2_524_593.5; // 2200-01-01 TT

fn mean_ayanamsa(tt: f64) -> f64 {
    let time = AstroTime { ut: tt, tt };
    let reference = AstroTime {
        ut: B1950 - J2000,
        tt: B1950 - J2000,
    };
    // The date's equinox [1, 0, 0], carried backwards to J2000 and
    // forwards to B1950. Matrices store their column vectors as rows.
    let date_matrix = precession_matrix(time);
    let reference_matrix = precession_matrix(reference);
    let vector: [f64; 3] = core::array::from_fn(|i| {
        (0..3)
            .map(|j| reference_matrix[j][i] * date_matrix[j][0])
            .sum()
    });
    let epsilon = e_tilt(reference).mobl.to_radians();
    let longitude = (vector[1] * epsilon.cos() + vector[2] * epsilon.sin())
        .atan2(vector[0])
        .to_degrees();
    B1950_OFFSET - MODERN_PRECESSION_CORRECTION - longitude
}

fn true_ayanamsa(time: AstroTime) -> f64 {
    mean_ayanamsa(time.tt) + e_tilt(time).dpsi / 3600.0
}

/// Offset to subtract from the engine's true-equinoctial tropical longitudes.
/// Supported from 1800-01-01 through 2200-01-01 TT (inclusive); chart coverage
/// is additionally limited by the caller's ephemeris. The underlying mean
/// reference is B1950, 24°02′31.36″, with the published 0.41256″
/// modern-precession correction and IAU 2006 precession.
pub fn fagan_bradley_ayanamsa(epoch: Epoch) -> Result<f64, CalculationError> {
    let time = AstroTime::from_tdb(epoch);
    if !(MIN_JD..=MAX_JD).contains(&(time.tt + J2000)) {
        return Err(CalculationError::InvalidInput(
            "Fagan–Bradley is supported from 1800-01-01 through 2200-01-01 TT",
        ));
    }
    Ok(true_ayanamsa(time))
}

/// Match the engine's centered one-day speed interval and time model.
pub(crate) fn ayanamsa_daily_rate(epoch: Epoch) -> f64 {
    let half_day = hifitime::Duration::from_days(0.5);
    true_ayanamsa(AstroTime::from_tdb(epoch + half_day))
        - true_ayanamsa(AstroTime::from_tdb(epoch - half_day))
}

// PAC Indian Astronomical Ephemeris 2027, p. 382 (formula since 2021), transcribed
// literally. The printed T⁴ coefficient, −0.00023857, is ten times the value in the
// source the IAE cites (Capitaine et al. 2003, p_A: −0.000023857). The IAE value is
// kept deliberately; the difference is at most 0.0034″ over 1800–2200. See
// docs/plans/2026-11-12-lahiri-full-nutation.md, D6.
fn lahiri_mean(tt: f64) -> f64 {
    let t = tt / 36525.0;
    23.0 + 51.0 / 60.0
        + 25.53 / 3600.0
        + t * (5028.796195
            + t * (1.1054348 + t * (0.00007964 + t * (-0.00023857 - t * 0.0000000383))))
            / 3600.0
}

/// Projection offset: IAE mean plus the engine's five-term nutation, matching
/// the nutation in the dataset's tropical longitudes so that it cancels.
fn lahiri_projection(time: AstroTime) -> f64 {
    lahiri_mean(time.tt) + e_tilt(time).dpsi / 3600.0
}

/// Reported true ayanamsa: IAE mean plus the complete IAU 2000B nutation in
/// longitude (IAE 2027 p. 423: true = mean + nutation in longitude).
fn lahiri_reported(time: AstroTime) -> f64 {
    lahiri_mean(time.tt) + iau2000b_full(time).dpsi / 3600.0
}

fn lahiri_time(epoch: Epoch) -> Result<AstroTime, CalculationError> {
    let time = AstroTime::from_tdb(epoch);
    if !(MIN_JD..=MAX_JD).contains(&(time.tt + J2000)) {
        return Err(CalculationError::InvalidInput(
            "Lahiri is supported from 1800-01-01 through 2200-01-01 TT",
        ));
    }
    Ok(time)
}

/// Lahiri true ayanamsa in degrees, as published by the Positional Astronomy
/// Centre: the IAE mean formula (IAE 2027 p. 382) plus the complete IAU 2000B
/// nutation in longitude. Supported from 1800-01-01 through 2200-01-01 TT
/// (inclusive); no ephemeris is needed.
///
/// This is the value charts report as `reference_offset_degrees`. Lahiri
/// sidereal longitudes are projected with a slightly different offset so that
/// nutation cancels (see the module documentation); as a result, for Lahiri
/// `tropical − sidereal` differs from this value by up to 0.27″.
pub fn lahiri_ayanamsa(epoch: Epoch) -> Result<f64, CalculationError> {
    Ok(lahiri_reported(lahiri_time(epoch)?))
}

/// Offsets a Lahiri chart needs, all in degrees.
pub(crate) struct LahiriOffsets {
    /// Reported true ayanamsa ([`lahiri_ayanamsa`]).
    pub reported: f64,
    /// Subtracted from tropical longitudes.
    pub projection: f64,
    /// Centered one-day change of the projection offset, applied to speeds.
    pub rate: f64,
}

pub(crate) fn lahiri_offsets(epoch: Epoch) -> Result<LahiriOffsets, CalculationError> {
    let time = lahiri_time(epoch)?;
    let half_day = hifitime::Duration::from_days(0.5);
    Ok(LahiriOffsets {
        reported: lahiri_reported(time),
        projection: lahiri_projection(time),
        rate: lahiri_projection(AstroTime::from_tdb(epoch + half_day))
            - lahiri_projection(AstroTime::from_tdb(epoch - half_day)),
    })
}

#[cfg(test)]
mod lahiri_tests {
    use super::*;
    #[test]
    fn official_mean_anchor() {
        assert!((lahiri_mean(0.0) - (23.0 + 51.0 / 60.0 + 25.53 / 3600.0)).abs() < 1e-12);
    }
}

#[cfg(test)]
mod lahiri_range_tests {
    use super::*;
    #[test]
    fn range_and_rate() {
        for jd in [MIN_JD, MAX_JD] {
            // Construct UTC instants well inside/outside each boundary; the
            // engine derives TT using its existing delta-T convention.
            let year = if jd == MIN_JD { "1800" } else { "2199" };
            let epoch: Epoch = format!("{year}-06-01T00:00:00Z").parse().unwrap();
            let offsets = lahiri_offsets(epoch).unwrap();
            assert!(offsets.reported.is_finite() && offsets.projection.is_finite());
            assert!(offsets.rate.is_finite() && offsets.rate.abs() < 0.001);
            assert_eq!(lahiri_ayanamsa(epoch).unwrap(), offsets.reported);
        }
        for utc in ["1799-01-01T00:00:00Z", "2201-01-01T00:00:00Z"] {
            assert!(lahiri_offsets(utc.parse().unwrap()).is_err());
            assert!(lahiri_ayanamsa(utc.parse().unwrap()).is_err());
        }
    }
}

#[cfg(test)]
mod lahiri_nutation_tests {
    use super::*;

    /// The two offsets differ only in the nutation series.
    #[test]
    fn reported_minus_projection_is_the_dropped_nutation() {
        for utc in [
            "1800-06-01T00:00:00Z",
            "1956-03-21T00:00:00Z",
            "2000-01-01T12:00:00Z",
            "2027-01-01T00:00:00Z",
            "2199-06-01T00:00:00Z",
        ] {
            let epoch: Epoch = utc.parse().unwrap();
            let time = AstroTime::from_tdb(epoch);
            let offsets = lahiri_offsets(epoch).unwrap();
            let expected = (iau2000b_full(time).dpsi - e_tilt(time).dpsi) / 3600.0;
            let difference = offsets.reported - offsets.projection;
            assert!(
                (difference - expected).abs() * 3600.0 < 1.0e-9,
                "{utc}: {difference} vs {expected}"
            );
            assert!(difference.abs() * 3600.0 < 0.3, "{utc}: {difference}");
        }
    }

    /// IAE 2027 p. 423 tabulates the true ayanamsa for 2027 January 1 (5h29m IST,
    /// about 0h TT) as 24°14′13″.9, rounded to 0.1″.
    #[test]
    fn reproduces_the_iae_2027_table_entry_for_january_1() {
        let epoch: Epoch = "2026-12-31T23:58:51Z".parse().unwrap();
        let arcsec = (lahiri_ayanamsa(epoch).unwrap() - 24.0) * 3600.0 - 14.0 * 60.0;
        assert!((arcsec - 13.9).abs() <= 0.05, "{arcsec}");
    }
}
