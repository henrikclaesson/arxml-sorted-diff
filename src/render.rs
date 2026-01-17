use crate::diff::{DiffNode, NodeStatus};
use crossterm::style::Stylize;
use anyhow::Result;

fn render_node(node: &DiffNode, indent: usize) {
    let prefix = match node.status {
        NodeStatus::Added => "+".green().to_string(),
        NodeStatus::Removed => "-".red().to_string(),
        NodeStatus::Changed => "~".yellow().to_string(),
        NodeStatus::Unchanged => " ".to_string(),
    };

    let name = node.key.as_deref().unwrap_or(&node.tag);
    let indent_str = "  ".repeat(indent);
    println!("{}{} {}", indent_str, prefix, name);

    for c in &node.children {
        render_node(c, indent + 1);
    }
}

pub fn render_tree(root: &DiffNode) -> Result<()> {
    render_node(root, 0);
    Ok(())
}

fn compute_max_width(node: &DiffNode) -> usize {
    let mut max = node.key.as_deref().unwrap_or(&node.tag).len();
    for c in &node.children {
        let child_max = compute_max_width(c);
        if child_max > max { max = child_max; }
    }
    max
}

fn line_values(node: &DiffNode) -> (String, String) {
    match node.status {
        NodeStatus::Added => ("".to_string(), node.key.as_deref().unwrap_or(&node.tag).to_string()),
        NodeStatus::Removed => (node.key.as_deref().unwrap_or(&node.tag).to_string(), "".to_string()),
        NodeStatus::Unchanged => {
            let k = node.key.as_deref().unwrap_or(&node.tag).to_string();
            (k.clone(), k)
        }
        NodeStatus::Changed => {
            if let Some(k) = &node.key { return (k.clone(), k.clone()); }
            // Try to extract a left and right candidate from children
            let mut left = String::new();
            let mut right = String::new();
            for c in &node.children {
                if left.is_empty() && c.status != NodeStatus::Added {
                    left = c.key.as_deref().unwrap_or(&c.tag).to_string();
                }
                if right.is_empty() && c.status != NodeStatus::Removed {
                    right = c.key.as_deref().unwrap_or(&c.tag).to_string();
                }
            }
            if left.is_empty() && right.is_empty() { (node.tag.clone(), node.tag.clone()) }
            else { (left, right) }
        }
    }
}

fn render_node_side(node: &DiffNode, indent: usize, left_width: usize) {
    let indent_str = "  ".repeat(indent);
    let (l, r) = line_values(node);

    // If node has no explicit key and its single child has the same key on both sides,
    // skip printing the parent row to avoid duplicate rows and render the child directly.
    if node.key.is_none() && node.children.len() == 1 {
        let child = &node.children[0];
        if let (Some(ck), Some(_)) = (child.key.as_deref(), Some(())) {
            if ck == l && ck == r {
                render_node_side(child, indent, left_width);
                return;
            }
        }
    }

    // Prepare left and right cell content with indentation
    let left_cell = if l.is_empty() { "".to_string() } else { format!("{}{}", indent_str, l) };
    let right_cell = if r.is_empty() { "".to_string() } else { format!("{}{}", indent_str, r) };

    // Apply colors: removed -> red (left), added -> green (right), changed -> yellow marker in middle
    let left_repr = if node.status == NodeStatus::Removed { left_cell.red().to_string() } else { left_cell.clone() };
    let right_repr = if node.status == NodeStatus::Added { right_cell.green().to_string() } else { right_cell.clone() };

    let mid = match node.status {
        NodeStatus::Changed => " ~ ".yellow().to_string(),
        NodeStatus::Added => "   ".to_string(),
        NodeStatus::Removed => "   ".to_string(),
        NodeStatus::Unchanged => "   ".to_string(),
    };

    println!("{:<width$}{}{}", left_repr, mid, right_repr, width = left_width + 2);

    for c in &node.children {
        render_node_side(c, indent + 1, left_width);
    }
}

pub fn render_side_by_side(root: &DiffNode) -> Result<()> {
    // compute a reasonable left column width
    let max = compute_max_width(root);
    let left_width = std::cmp::min(40, max) + 4; // add padding
    println!("{:<width$} | {}", "LEFT", "RIGHT", width = left_width + 2);
    println!("{:-<width$}-+-{:-<right$}", "", "", width = left_width + 2, right = 20);

    render_node_side(root, 0, left_width);
    Ok(())
}
