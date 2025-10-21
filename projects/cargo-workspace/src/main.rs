use cargo_workspace::{
    CargoError, CargoWorkspaceCommand
    ,
};
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), CargoError> {
    // Initialize logger
    tracing_subscriber::fmt().init();
    // Parse command line arguments
    CargoWorkspaceCommand::parse().run().await
}
