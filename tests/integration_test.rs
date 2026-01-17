use arxml_diff::parse::parse_file;

#[test]
fn parse_sample_files() {
    let left = parse_file("tests/fixtures/sample-left.arxml").expect("parse left");
    let right = parse_file("tests/fixtures/sample-right.arxml").expect("parse right");
    assert_eq!(left.tag, "ARXML");
    assert_eq!(right.tag, "ARXML");
}

#[test]
fn short_name_extraction() {
    let left = parse_file("tests/fixtures/sample-left.arxml").expect("parse left");
    // root -> ECU
    let ecu = left.children.iter().find(|c| c.tag.eq_ignore_ascii_case("ECU")).expect("ECU");
    assert_eq!(ecu.short_name.as_deref(), Some("MyEcu"));
    let components = ecu.children.iter().find(|c| c.tag.eq_ignore_ascii_case("COMPONENTS")).expect("COMPONENTS");
    let comp_names: Vec<_> = components.children.iter().filter_map(|c| c.short_name.clone()).collect();
    assert_eq!(comp_names, vec!["CompA".to_string(), "CompB".to_string()]);

    let right = parse_file("tests/fixtures/sample-right.arxml").expect("parse right");
    let ecu_r = right.children.iter().find(|c| c.tag.eq_ignore_ascii_case("ECU")).expect("ECU");
    let components_r = ecu_r.children.iter().find(|c| c.tag.eq_ignore_ascii_case("COMPONENTS")).expect("COMPONENTS");
    let comp_names_r: Vec<_> = components_r.children.iter().filter_map(|c| c.short_name.clone()).collect();
    assert_eq!(comp_names_r, vec!["CompB".to_string(), "CompC".to_string()]);
}
