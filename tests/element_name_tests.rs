use std::str::FromStr;

#[cfg(feature = "autosar")]
use autosar_data::ElementName;

#[test]
fn parse_common_element_names() {
    // Verify specific element name variants exist and round-trip
    let e1 = ElementName::from_str("IMPLEMENTATION-DATA-TYPE").expect("parse IMPLEMENTATION-DATA-TYPE");
    assert_eq!(e1.to_str(), "IMPLEMENTATION-DATA-TYPE");

    let e2 = ElementName::from_str("RUNNABLE-ENTITY").expect("parse RUNNABLE-ENTITY");
    assert_eq!(e2.to_str(), "RUNNABLE-ENTITY");

    let e3 = ElementName::from_str("AR-PACKAGES").expect("parse AR-PACKAGES");
    assert_eq!(e3.to_str(), "AR-PACKAGES");
}

#[test]
fn parse_invalid_element_name() {
    // Ensure invalid names return an error
    let err = ElementName::from_str("NOT-A-REAL-NAME").err();
    assert!(err.is_some(), "invalid element name should error");
}
