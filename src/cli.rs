use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum View {
    Unified,
    SideBySide,
}

#[derive(Parser, Debug)]
#[command(name = "arxml-diff")]
pub struct Args {
    pub left: String,
    pub right: String,

    #[arg(long, short)]
    pub interactive: bool,

    #[arg(long, value_enum, default_value_t = View::Unified)]
    pub view: View,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
