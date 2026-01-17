pub mod cli;
pub mod parse;
pub mod diff;
pub mod render;
pub mod tui;

use anyhow::Result;
use crate::cli::Args;

pub fn run(args: Args) -> Result<()> {
    let left = parse::parse_file(&args.left)?;
    let right = parse::parse_file(&args.right)?;

    let diff_root = diff::compute_tree_diff(&left, &right);

    if args.interactive {
        // launch the interactive TUI
        crate::tui::run_tui(&diff_root)?;
        return Ok(())
    }

    match args.view {
        crate::cli::View::Unified => render::render_tree(&diff_root)?,
        crate::cli::View::SideBySide => render::render_side_by_side(&diff_root)?,
    }

    Ok(())
}
