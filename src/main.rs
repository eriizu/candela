mod recursive_cleaner;

use clap::Parser;

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "candela")]
#[command(about = "A personal helper for managing code repos", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Clean { base_dirs: Vec<String> },
}

fn main() {
    let opt = Cli::parse();
    match opt.command {
        Commands::Clean { mut base_dirs } => {
            recursive_cleaner::RecursiveCleaner::new().run(base_dirs.drain(..));
        }
    }
}
