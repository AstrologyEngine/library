use astrology_engine::{
    effective_configuration, SignDivisions, ZodiacConfiguration, ZodiacReference,
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
