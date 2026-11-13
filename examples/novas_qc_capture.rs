// Copyright (c) Jack Asher
// SPDX-License-Identifier: MIT

#[cfg(feature = "generation")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use astrology_engine::tooling::{nutation::iau2000b_full, time::AstroTime};
    use astrology_engine::{
        calculate_chart, lahiri_ayanamsa, ChartInput, Ephemeris, Epoch, HouseSystem, Location,
        ZodiacConfiguration,
    };
    use serde::Deserialize;
    use serde_json::json;
    use std::io::{self, Read};
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Case {
        id: String,
        utc: String,
        zodiac: String,
        chart: bool,
        latitude: f64,
        longitude: f64,
        #[serde(rename = "houseSystem")]
        house_system: HouseSystem,
    }
    let data = std::env::args().nth(1).map(std::fs::read).transpose()?;
    let ephemeris = data
        .map(Ephemeris::parse)
        .transpose()
        .map_err(|e| format!("Dataset: {e:?}"))?;
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let cases: Vec<Case> = serde_json::from_str(&input)?;
    let mut output = Vec::with_capacity(cases.len());
    for case in cases {
        let epoch: Epoch = case.utc.parse()?;
        let time = AstroTime::from_tdb(epoch);
        let nutation = iau2000b_full(time);
        let reported = lahiri_ayanamsa(epoch)?;
        let chart = if case.chart {
            let eph = ephemeris
                .as_ref()
                .ok_or("Chart requested without dataset")?;
            let configuration: ZodiacConfiguration = serde_json::from_value(
                json!({"reference":case.zodiac,"divisions":"equal","ophiuchus":null}),
            )?;
            Some(
                match calculate_chart(
                    eph,
                    ChartInput {
                        epoch,
                        location: Location {
                            latitude: case.latitude,
                            longitude: case.longitude,
                        },
                        house_system: case.house_system,
                        zodiac: configuration,
                    },
                ) {
                    Ok(value) => json!({"value":value}),
                    Err(error) => json!({"error":error.to_string()}),
                },
            )
        } else {
            None
        };
        output.push(json!({"id":case.id,"utc":case.utc,"zodiac":case.zodiac,
            "effectiveJdTT":time.tt+2451545.0,"effectiveJdUT1":time.ut+2451545.0,
            "physicalJdTT":epoch.to_jde_tt_days(),"etSeconds":epoch.to_et_seconds(),
            "reportedLahiri":reported,"nutationArcseconds":{"longitude":nutation.dpsi,"obliquity":nutation.deps},"chart":chart}));
    }
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
#[cfg(not(feature = "generation"))]
fn main() {
    eprintln!("Use --features generation for the QC capture adapter");
    std::process::exit(2);
}
