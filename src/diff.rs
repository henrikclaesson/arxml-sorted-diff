use crate::parse::Element;
use similar::{capture_diff_slices, Algorithm};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeStatus {
    Unchanged,
    Added,
    Removed,
    Changed,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiffNode {
    pub status: NodeStatus,
    pub tag: String,
    pub key: Option<String>,
    pub children: Vec<DiffNode>,
}

fn key_of(e: &Element) -> String {
    if let Some(sn) = &e.short_name { return sn.clone(); }
    if let Some(u) = &e.uuid { return u.clone(); }
    e.tag.clone()
}

fn diff_elements(left: Option<&Element>, right: Option<&Element>) -> DiffNode {
    match (left, right) {
        (Some(l), None) => {
            // Entire subtree removed
            let mut children = Vec::new();
            for c in &l.children {
                children.push(diff_elements(Some(c), None));
            }
            DiffNode { status: NodeStatus::Removed, tag: l.tag.clone(), key: l.short_name.clone().or(l.uuid.clone()), children }
        }
        (None, Some(r)) => {
            // Entire subtree added
            let mut children = Vec::new();
            for c in &r.children {
                children.push(diff_elements(None, Some(c)));
            }
            DiffNode { status: NodeStatus::Added, tag: r.tag.clone(), key: r.short_name.clone().or(r.uuid.clone()), children }
        }
        (Some(l), Some(r)) => {
            let key_l = key_of(l);
            let key_r = key_of(r);
            if key_l != key_r || l.tag != r.tag {
                // Different nodes in same position: represent as removed + added
                return DiffNode {
                    status: NodeStatus::Changed,
                    tag: format!("{} -> {}", l.tag, r.tag),
                    key: None,
                    children: vec![diff_elements(Some(l), None), diff_elements(None, Some(r))],
                };
            }

            // Same key/tag: compare attributes/text and children
            let mut node = DiffNode { status: NodeStatus::Unchanged, tag: l.tag.clone(), key: l.short_name.clone().or(l.uuid.clone()), children: Vec::new() };

            // Quick content check (attributes/text)
            if l.text != r.text || l.attributes != r.attributes {
                node.status = NodeStatus::Changed;
            }

            // Diff children by keys using similar
            let left_keys: Vec<String> = l.children.iter().map(|c| key_of(c)).collect();
            let right_keys: Vec<String> = r.children.iter().map(|c| key_of(c)).collect();

            let ops = capture_diff_slices(Algorithm::Myers, &left_keys, &right_keys);

            for op in ops.iter() {
                use similar::DiffOp as SOp;
                match op {
                    SOp::Equal { old_index, new_index, len } => {
                        for k in 0..*len {
                            let li = old_index + k;
                            let ri = new_index + k;
                            let child = diff_elements(Some(&l.children[li]), Some(&r.children[ri]));
                            if child.status != NodeStatus::Unchanged { node.status = NodeStatus::Changed; }
                            node.children.push(child);
                        }
                    }
                    SOp::Delete { old_index, old_len, .. } => {
                        for k in 0..*old_len {
                            let li = old_index + k;
                            let child = diff_elements(Some(&l.children[li]), None);
                            node.status = NodeStatus::Changed;
                            node.children.push(child);
                        }
                    }
                    SOp::Insert { new_index, new_len, .. } => {
                        for k in 0..*new_len {
                            let ri = new_index + k;
                            let child = diff_elements(None, Some(&r.children[ri]));
                            node.status = NodeStatus::Changed;
                            node.children.push(child);
                        }
                    }
                    SOp::Replace { old_index, old_len, new_index, new_len } => {
                        for k in 0..*old_len {
                            let li = old_index + k;
                            let child = diff_elements(Some(&l.children[li]), None);
                            node.status = NodeStatus::Changed;
                            node.children.push(child);
                        }
                        for k in 0..*new_len {
                            let ri = new_index + k;
                            let child = diff_elements(None, Some(&r.children[ri]));
                            node.status = NodeStatus::Changed;
                            node.children.push(child);
                        }
                    }
                }
            }

            node
        }
        (None, None) => panic!("diff_elements called with None, None"),
    }
}

pub fn compute_tree_diff(left: &Element, right: &Element) -> DiffNode {
    diff_elements(Some(left), Some(right))
}
