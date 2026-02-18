use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lum")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    Build,
    Run,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { dir } => lumina_part1_lum_cli::init_project(&dir)?,
        Commands::Build => {
            let _exe = lumina_part1_lum_cli::build_project(&PathBuf::from("."))?;
        }
        Commands::Run => lumina_part1_lum_cli::run_project(&PathBuf::from("."))?,
    }
    Ok(())
}
