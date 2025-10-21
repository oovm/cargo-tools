use cargo_workspace::{Cargo, CargoError};
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), CargoError> {
    tracing_subscriber::fmt().init();
    let Cargo::Workspace(cmd) = Cargo::parse();
    cmd.run().await
}
