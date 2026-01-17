# arxml-diff

Terminal ARXML structural diff tool (Rust)

Goals:
- Parse AUTOSAR ARXML with `quick-xml` (streaming)
- Compute structure-aware diffs keyed by `SHORT-NAME`
- Provide colored CLI output (added=green, removed=red) and an interactive `ratatui` TUI

Quickstart:
- cargo build
- cargo run -- sample-left.arxml sample-right.arxml
- cargo run -- --interactive sample-left.arxml sample-right.arxml
