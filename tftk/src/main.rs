mod cli;
mod compose;
mod convert;
mod utils;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Convert(cli) => crate::convert::convert(cli)?,
        Cli::Compose(cli) => crate::compose::compose(cli)?,
    }

    Ok(())
}
