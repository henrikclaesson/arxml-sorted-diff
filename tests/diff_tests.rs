use arxml_diff::parse::parse_file;
use arxml_diff::diff::{compute_tree_diff, NodeStatus};

#[test]
fn sibling_diff_detects_add_remove_common() {
    let left = parse_file("tests/fixtures/sample-left.arxml").expect("parse left");
    let right = parse_file("tests/fixtures/sample-right.arxml").expect("parse right");

    let ecu_l = left.children.iter().find(|c| c.tag.eq_ignore_ascii_case("ECU")).expect("ECU");
    let comps_l = ecu_l.children.iter().find(|c| c.tag.eq_ignore_ascii_case("COMPONENTS")).expect("COMPONENTS");

    let ecu_r = right.children.iter().find(|c| c.tag.eq_ignore_ascii_case("ECU")).expect("ECU");
    let comps_r = ecu_r.children.iter().find(|c| c.tag.eq_ignore_ascii_case("COMPONENTS")).expect("COMPONENTS");

    let diff = compute_tree_diff(comps_l, comps_r);

    // Expect three children: Removed CompA, Common CompB, Added CompC
    assert_eq!(diff.children.len(), 3);
    assert_eq!(diff.children[0].status, NodeStatus::Removed);
    assert_eq!(diff.children[0].key.as_deref(), Some("CompA"));
    assert_eq!(diff.children[1].status, NodeStatus::Unchanged);
    assert_eq!(diff.children[1].key.as_deref(), Some("CompB"));
    assert_eq!(diff.children[2].status, NodeStatus::Added);
    assert_eq!(diff.children[2].key.as_deref(), Some("CompC"));
}
