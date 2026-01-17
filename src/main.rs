use anyhow::Result;
use arxml_diff::cli::Args;

fn main() -> Result<()> {
    let args = Args::parse_args();
    arxml_diff::run(args)
}
