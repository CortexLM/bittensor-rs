use bittensor_cli::Cli;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(cli.run())
}
