// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

//! IAU 2000B nutation: the complete 77-term luni-solar series.
//!
//! Model: McCarthy, D. D. & Luzum, B. J. (2003), "An abridged model of the
//! precession-nutation of the celestial pole", Cel. Mech. Dyn. Astron. 85, 37-49;
//! IERS Conventions (2003), Chapter 5. Fundamental arguments from Simon et al. (1994).
//!
//! The coefficients and the fixed planetary-bias terms (Luzum 2001) are
//! transcribed mechanically from NOVAS C 3.1 `nutation.c`, U.S. Naval Observatory,
//! which is public domain and carries no licence conditions. The same table appears,
//! value for value, in ERFA `nut00b.c`; the benchmark checks this implementation
//! against NOVAS directly.
//!
//! [`super::frames::iau2000b`] keeps only the first five rows of this table and is
//! unchanged. See `docs/plans/2026-11-12-lahiri-full-nutation.md`.
//!
//! Source table: NOVAS C 3.1 `nutation.c`, SHA-256 `49d193c9e0d6dfb14650ed0b2603e3954ef02ab51f83d66869a769e892e471bb`.

use super::frames::{ASEC2RAD, ASEC360};
use super::time::{AstroTime, DAYS_PER_CENTURY};

/// One luni-solar term: Delaunay multipliers for l, l', F, D and Omega, then
/// longitude (sin, t*sin, cos) and obliquity (cos, t*cos, sin) coefficients in
/// units of 1e-7 arcseconds.
#[derive(Clone, Copy)]
pub(crate) struct Term {
    pub multipliers: [i8; 5],
    pub longitude: [f64; 3],
    pub obliquity: [f64; 3],
}

/// Nutation angles in arcseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nutation {
    pub dpsi: f64,
    pub deps: f64,
}

/// Fixed "planetary bias" terms, arcseconds (Luzum 2001, as used by NOVAS and ERFA).
pub(crate) const DPPLAN: f64 = -0.000135;
pub(crate) const DEPLAN: f64 = 0.000388;

/// Sum `terms` at `t` Julian centuries of TT from J2000, in 1e-7 arcseconds.
/// Summed in reverse order, smallest terms first, as NOVAS does.
pub(crate) fn accumulate(terms: &[Term], t: f64) -> (f64, f64) {
    let turns = |x: f64| (x % ASEC360) * ASEC2RAD;
    let arguments = [
        turns(485_868.249036 + t * 1_717_915_923.2178), // l,  Moon's mean anomaly
        turns(1_287_104.79305 + t * 129_596_581.0481),  // l', Sun's mean anomaly
        turns(335_779.526232 + t * 1_739_527_262.8478), // F,  Moon's mean argument of latitude
        turns(1_072_260.70369 + t * 1_602_961_601.2090), // D,  Moon's mean elongation from the Sun
        turns(450_160.398036 - t * 6_962_890.5431),     // Omega, Moon's mean ascending node
    ];
    let (mut dpsi, mut deps) = (0.0, 0.0);
    for term in terms.iter().rev() {
        let argument: f64 = term
            .multipliers
            .iter()
            .zip(arguments)
            .map(|(multiplier, value)| f64::from(*multiplier) * value)
            .sum::<f64>()
            % core::f64::consts::TAU;
        let (sin_argument, cos_argument) = argument.sin_cos();
        dpsi += (term.longitude[0] + term.longitude[1] * t) * sin_argument
            + term.longitude[2] * cos_argument;
        deps += (term.obliquity[0] + term.obliquity[1] * t) * cos_argument
            + term.obliquity[2] * sin_argument;
    }
    (dpsi, deps)
}

/// Nutation in longitude and obliquity, arcseconds, from the full series.
#[cfg_attr(not(feature = "generation"), allow(dead_code))]
pub fn iau2000b_full(time: AstroTime) -> Nutation {
    let (dpsi, deps) = accumulate(&TERMS, time.tt / DAYS_PER_CENTURY);
    Nutation {
        dpsi: DPPLAN + dpsi * 1.0e-7,
        deps: DEPLAN + deps * 1.0e-7,
    }
}

pub(crate) const TERMS: [Term; 77] = [
    // 1
    Term {
        multipliers: [0, 0, 0, 0, 1],
        longitude: [-172064161.0, -174666.0, 33386.0],
        obliquity: [92052331.0, 9086.0, 15377.0],
    },
    // 2
    Term {
        multipliers: [0, 0, 2, -2, 2],
        longitude: [-13170906.0, -1675.0, -13696.0],
        obliquity: [5730336.0, -3015.0, -4587.0],
    },
    // 3
    Term {
        multipliers: [0, 0, 2, 0, 2],
        longitude: [-2276413.0, -234.0, 2796.0],
        obliquity: [978459.0, -485.0, 1374.0],
    },
    // 4
    Term {
        multipliers: [0, 0, 0, 0, 2],
        longitude: [2074554.0, 207.0, -698.0],
        obliquity: [-897492.0, 470.0, -291.0],
    },
    // 5
    Term {
        multipliers: [0, 1, 0, 0, 0],
        longitude: [1475877.0, -3633.0, 11817.0],
        obliquity: [73871.0, -184.0, -1924.0],
    },
    // 6
    Term {
        multipliers: [0, 1, 2, -2, 2],
        longitude: [-516821.0, 1226.0, -524.0],
        obliquity: [224386.0, -677.0, -174.0],
    },
    // 7
    Term {
        multipliers: [1, 0, 0, 0, 0],
        longitude: [711159.0, 73.0, -872.0],
        obliquity: [-6750.0, 0.0, 358.0],
    },
    // 8
    Term {
        multipliers: [0, 0, 2, 0, 1],
        longitude: [-387298.0, -367.0, 380.0],
        obliquity: [200728.0, 18.0, 318.0],
    },
    // 9
    Term {
        multipliers: [1, 0, 2, 0, 2],
        longitude: [-301461.0, -36.0, 816.0],
        obliquity: [129025.0, -63.0, 367.0],
    },
    // 10
    Term {
        multipliers: [0, -1, 2, -2, 2],
        longitude: [215829.0, -494.0, 111.0],
        obliquity: [-95929.0, 299.0, 132.0],
    },
    // 11
    Term {
        multipliers: [0, 0, 2, -2, 1],
        longitude: [128227.0, 137.0, 181.0],
        obliquity: [-68982.0, -9.0, 39.0],
    },
    // 12
    Term {
        multipliers: [-1, 0, 2, 0, 2],
        longitude: [123457.0, 11.0, 19.0],
        obliquity: [-53311.0, 32.0, -4.0],
    },
    // 13
    Term {
        multipliers: [-1, 0, 0, 2, 0],
        longitude: [156994.0, 10.0, -168.0],
        obliquity: [-1235.0, 0.0, 82.0],
    },
    // 14
    Term {
        multipliers: [1, 0, 0, 0, 1],
        longitude: [63110.0, 63.0, 27.0],
        obliquity: [-33228.0, 0.0, -9.0],
    },
    // 15
    Term {
        multipliers: [-1, 0, 0, 0, 1],
        longitude: [-57976.0, -63.0, -189.0],
        obliquity: [31429.0, 0.0, -75.0],
    },
    // 16
    Term {
        multipliers: [-1, 0, 2, 2, 2],
        longitude: [-59641.0, -11.0, 149.0],
        obliquity: [25543.0, -11.0, 66.0],
    },
    // 17
    Term {
        multipliers: [1, 0, 2, 0, 1],
        longitude: [-51613.0, -42.0, 129.0],
        obliquity: [26366.0, 0.0, 78.0],
    },
    // 18
    Term {
        multipliers: [-2, 0, 2, 0, 1],
        longitude: [45893.0, 50.0, 31.0],
        obliquity: [-24236.0, -10.0, 20.0],
    },
    // 19
    Term {
        multipliers: [0, 0, 0, 2, 0],
        longitude: [63384.0, 11.0, -150.0],
        obliquity: [-1220.0, 0.0, 29.0],
    },
    // 20
    Term {
        multipliers: [0, 0, 2, 2, 2],
        longitude: [-38571.0, -1.0, 158.0],
        obliquity: [16452.0, -11.0, 68.0],
    },
    // 21
    Term {
        multipliers: [0, -2, 2, -2, 2],
        longitude: [32481.0, 0.0, 0.0],
        obliquity: [-13870.0, 0.0, 0.0],
    },
    // 22
    Term {
        multipliers: [-2, 0, 0, 2, 0],
        longitude: [-47722.0, 0.0, -18.0],
        obliquity: [477.0, 0.0, -25.0],
    },
    // 23
    Term {
        multipliers: [2, 0, 2, 0, 2],
        longitude: [-31046.0, -1.0, 131.0],
        obliquity: [13238.0, -11.0, 59.0],
    },
    // 24
    Term {
        multipliers: [1, 0, 2, -2, 2],
        longitude: [28593.0, 0.0, -1.0],
        obliquity: [-12338.0, 10.0, -3.0],
    },
    // 25
    Term {
        multipliers: [-1, 0, 2, 0, 1],
        longitude: [20441.0, 21.0, 10.0],
        obliquity: [-10758.0, 0.0, -3.0],
    },
    // 26
    Term {
        multipliers: [2, 0, 0, 0, 0],
        longitude: [29243.0, 0.0, -74.0],
        obliquity: [-609.0, 0.0, 13.0],
    },
    // 27
    Term {
        multipliers: [0, 0, 2, 0, 0],
        longitude: [25887.0, 0.0, -66.0],
        obliquity: [-550.0, 0.0, 11.0],
    },
    // 28
    Term {
        multipliers: [0, 1, 0, 0, 1],
        longitude: [-14053.0, -25.0, 79.0],
        obliquity: [8551.0, -2.0, -45.0],
    },
    // 29
    Term {
        multipliers: [-1, 0, 0, 2, 1],
        longitude: [15164.0, 10.0, 11.0],
        obliquity: [-8001.0, 0.0, -1.0],
    },
    // 30
    Term {
        multipliers: [0, 2, 2, -2, 2],
        longitude: [-15794.0, 72.0, -16.0],
        obliquity: [6850.0, -42.0, -5.0],
    },
    // 31
    Term {
        multipliers: [0, 0, -2, 2, 0],
        longitude: [21783.0, 0.0, 13.0],
        obliquity: [-167.0, 0.0, 13.0],
    },
    // 32
    Term {
        multipliers: [1, 0, 0, -2, 1],
        longitude: [-12873.0, -10.0, -37.0],
        obliquity: [6953.0, 0.0, -14.0],
    },
    // 33
    Term {
        multipliers: [0, -1, 0, 0, 1],
        longitude: [-12654.0, 11.0, 63.0],
        obliquity: [6415.0, 0.0, 26.0],
    },
    // 34
    Term {
        multipliers: [-1, 0, 2, 2, 1],
        longitude: [-10204.0, 0.0, 25.0],
        obliquity: [5222.0, 0.0, 15.0],
    },
    // 35
    Term {
        multipliers: [0, 2, 0, 0, 0],
        longitude: [16707.0, -85.0, -10.0],
        obliquity: [168.0, -1.0, 10.0],
    },
    // 36
    Term {
        multipliers: [1, 0, 2, 2, 2],
        longitude: [-7691.0, 0.0, 44.0],
        obliquity: [3268.0, 0.0, 19.0],
    },
    // 37
    Term {
        multipliers: [-2, 0, 2, 0, 0],
        longitude: [-11024.0, 0.0, -14.0],
        obliquity: [104.0, 0.0, 2.0],
    },
    // 38
    Term {
        multipliers: [0, 1, 2, 0, 2],
        longitude: [7566.0, -21.0, -11.0],
        obliquity: [-3250.0, 0.0, -5.0],
    },
    // 39
    Term {
        multipliers: [0, 0, 2, 2, 1],
        longitude: [-6637.0, -11.0, 25.0],
        obliquity: [3353.0, 0.0, 14.0],
    },
    // 40
    Term {
        multipliers: [0, -1, 2, 0, 2],
        longitude: [-7141.0, 21.0, 8.0],
        obliquity: [3070.0, 0.0, 4.0],
    },
    // 41
    Term {
        multipliers: [0, 0, 0, 2, 1],
        longitude: [-6302.0, -11.0, 2.0],
        obliquity: [3272.0, 0.0, 4.0],
    },
    // 42
    Term {
        multipliers: [1, 0, 2, -2, 1],
        longitude: [5800.0, 10.0, 2.0],
        obliquity: [-3045.0, 0.0, -1.0],
    },
    // 43
    Term {
        multipliers: [2, 0, 2, -2, 2],
        longitude: [6443.0, 0.0, -7.0],
        obliquity: [-2768.0, 0.0, -4.0],
    },
    // 44
    Term {
        multipliers: [-2, 0, 0, 2, 1],
        longitude: [-5774.0, -11.0, -15.0],
        obliquity: [3041.0, 0.0, -5.0],
    },
    // 45
    Term {
        multipliers: [2, 0, 2, 0, 1],
        longitude: [-5350.0, 0.0, 21.0],
        obliquity: [2695.0, 0.0, 12.0],
    },
    // 46
    Term {
        multipliers: [0, -1, 2, -2, 1],
        longitude: [-4752.0, -11.0, -3.0],
        obliquity: [2719.0, 0.0, -3.0],
    },
    // 47
    Term {
        multipliers: [0, 0, 0, -2, 1],
        longitude: [-4940.0, -11.0, -21.0],
        obliquity: [2720.0, 0.0, -9.0],
    },
    // 48
    Term {
        multipliers: [-1, -1, 0, 2, 0],
        longitude: [7350.0, 0.0, -8.0],
        obliquity: [-51.0, 0.0, 4.0],
    },
    // 49
    Term {
        multipliers: [2, 0, 0, -2, 1],
        longitude: [4065.0, 0.0, 6.0],
        obliquity: [-2206.0, 0.0, 1.0],
    },
    // 50
    Term {
        multipliers: [1, 0, 0, 2, 0],
        longitude: [6579.0, 0.0, -24.0],
        obliquity: [-199.0, 0.0, 2.0],
    },
    // 51
    Term {
        multipliers: [0, 1, 2, -2, 1],
        longitude: [3579.0, 0.0, 5.0],
        obliquity: [-1900.0, 0.0, 1.0],
    },
    // 52
    Term {
        multipliers: [1, -1, 0, 0, 0],
        longitude: [4725.0, 0.0, -6.0],
        obliquity: [-41.0, 0.0, 3.0],
    },
    // 53
    Term {
        multipliers: [-2, 0, 2, 0, 2],
        longitude: [-3075.0, 0.0, -2.0],
        obliquity: [1313.0, 0.0, -1.0],
    },
    // 54
    Term {
        multipliers: [3, 0, 2, 0, 2],
        longitude: [-2904.0, 0.0, 15.0],
        obliquity: [1233.0, 0.0, 7.0],
    },
    // 55
    Term {
        multipliers: [0, -1, 0, 2, 0],
        longitude: [4348.0, 0.0, -10.0],
        obliquity: [-81.0, 0.0, 2.0],
    },
    // 56
    Term {
        multipliers: [1, -1, 2, 0, 2],
        longitude: [-2878.0, 0.0, 8.0],
        obliquity: [1232.0, 0.0, 4.0],
    },
    // 57
    Term {
        multipliers: [0, 0, 0, 1, 0],
        longitude: [-4230.0, 0.0, 5.0],
        obliquity: [-20.0, 0.0, -2.0],
    },
    // 58
    Term {
        multipliers: [-1, -1, 2, 2, 2],
        longitude: [-2819.0, 0.0, 7.0],
        obliquity: [1207.0, 0.0, 3.0],
    },
    // 59
    Term {
        multipliers: [-1, 0, 2, 0, 0],
        longitude: [-4056.0, 0.0, 5.0],
        obliquity: [40.0, 0.0, -2.0],
    },
    // 60
    Term {
        multipliers: [0, -1, 2, 2, 2],
        longitude: [-2647.0, 0.0, 11.0],
        obliquity: [1129.0, 0.0, 5.0],
    },
    // 61
    Term {
        multipliers: [-2, 0, 0, 0, 1],
        longitude: [-2294.0, 0.0, -10.0],
        obliquity: [1266.0, 0.0, -4.0],
    },
    // 62
    Term {
        multipliers: [1, 1, 2, 0, 2],
        longitude: [2481.0, 0.0, -7.0],
        obliquity: [-1062.0, 0.0, -3.0],
    },
    // 63
    Term {
        multipliers: [2, 0, 0, 0, 1],
        longitude: [2179.0, 0.0, -2.0],
        obliquity: [-1129.0, 0.0, -2.0],
    },
    // 64
    Term {
        multipliers: [-1, 1, 0, 1, 0],
        longitude: [3276.0, 0.0, 1.0],
        obliquity: [-9.0, 0.0, 0.0],
    },
    // 65
    Term {
        multipliers: [1, 1, 0, 0, 0],
        longitude: [-3389.0, 0.0, 5.0],
        obliquity: [35.0, 0.0, -2.0],
    },
    // 66
    Term {
        multipliers: [1, 0, 2, 0, 0],
        longitude: [3339.0, 0.0, -13.0],
        obliquity: [-107.0, 0.0, 1.0],
    },
    // 67
    Term {
        multipliers: [-1, 0, 2, -2, 1],
        longitude: [-1987.0, 0.0, -6.0],
        obliquity: [1073.0, 0.0, -2.0],
    },
    // 68
    Term {
        multipliers: [1, 0, 0, 0, 2],
        longitude: [-1981.0, 0.0, 0.0],
        obliquity: [854.0, 0.0, 0.0],
    },
    // 69
    Term {
        multipliers: [-1, 0, 0, 1, 0],
        longitude: [4026.0, 0.0, -353.0],
        obliquity: [-553.0, 0.0, -139.0],
    },
    // 70
    Term {
        multipliers: [0, 0, 2, 1, 2],
        longitude: [1660.0, 0.0, -5.0],
        obliquity: [-710.0, 0.0, -2.0],
    },
    // 71
    Term {
        multipliers: [-1, 0, 2, 4, 2],
        longitude: [-1521.0, 0.0, 9.0],
        obliquity: [647.0, 0.0, 4.0],
    },
    // 72
    Term {
        multipliers: [-1, 1, 0, 1, 1],
        longitude: [1314.0, 0.0, 0.0],
        obliquity: [-700.0, 0.0, 0.0],
    },
    // 73
    Term {
        multipliers: [0, -2, 2, -2, 1],
        longitude: [-1283.0, 0.0, 0.0],
        obliquity: [672.0, 0.0, 0.0],
    },
    // 74
    Term {
        multipliers: [1, 0, 2, 2, 1],
        longitude: [-1331.0, 0.0, 8.0],
        obliquity: [663.0, 0.0, 4.0],
    },
    // 75
    Term {
        multipliers: [-2, 0, 2, 2, 2],
        longitude: [1383.0, 0.0, -2.0],
        obliquity: [-594.0, 0.0, -2.0],
    },
    // 76
    Term {
        multipliers: [-1, 0, 0, 0, 2],
        longitude: [1405.0, 0.0, 4.0],
        obliquity: [-610.0, 0.0, 2.0],
    },
    // 77
    Term {
        multipliers: [1, 1, 2, -2, 2],
        longitude: [1290.0, 0.0, 0.0],
        obliquity: [-556.0, 0.0, 0.0],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_shape() {
        assert_eq!(TERMS.len(), 77);
        for term in &TERMS {
            assert!(term.multipliers.iter().all(|m| (-2..=4).contains(m)));
            assert!(term
                .longitude
                .iter()
                .chain(&term.obliquity)
                .all(|c| c.is_finite()));
        }
        // Largest term first: the 18.6-year Omega term, ~17.2 arcseconds.
        assert_eq!(TERMS[0].multipliers, [0, 0, 0, 0, 1]);
        assert!((TERMS[0].longitude[0] * 1.0e-7 + 17.206_4).abs() < 1.0e-3);
    }

    /// Fixed values from NOVAS C 3.1 `iau2000b`, computed via the benchmark
    /// service (see the project README); dpsi/deps in arcseconds.
    #[test]
    fn matches_novas_reference_values() {
        const CASES: [(f64, f64, f64); 5] = [
            (2378496.5, -8.526217553076, 7.229612113575799),
            (2451545.0, -13.931663888969783, -5.769417077292847),
            (2460676.5, 0.19792331217020098, 8.503935144012724),
            (2461406.5, 10.55450240907523, 6.840171190637621),
            (2524593.5, 11.081349407140154, -8.01619377861145),
        ];
        for (jd_tt, dpsi, deps) in CASES {
            let time = AstroTime {
                ut: jd_tt - 2_451_545.0,
                tt: jd_tt - 2_451_545.0,
            };
            let got = iau2000b_full(time);
            assert!(
                (got.dpsi - dpsi).abs() < 1.0e-9,
                "dpsi at {jd_tt}: {} vs {dpsi}",
                got.dpsi
            );
            assert!(
                (got.deps - deps).abs() < 1.0e-9,
                "deps at {jd_tt}: {} vs {deps}",
                got.deps
            );
        }
    }
}
