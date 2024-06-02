mod flattener;
mod recursive_cleaner;

use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");
// INFO: ASCII candle copyright of Joan Stack
const CANDLE_ASCII_ART: &'static str = r#"
            )
           (_)
          .-'-.
          |   |
          |   |
          |   |
          |   |
        __|   |__   .-.
     .-'  |   |  `-:   :
    :     `---'     :-'
jgs  `-._       _.-'
         '""""""
"#;

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
    Clean {
        base_dirs: Vec<String>,
    },
    Flatten(flattener::Cli),
    Version,
}

fn main() {
    let opt = Cli::parse();
    match opt.command {
        Commands::Clean { mut base_dirs } => {
            recursive_cleaner::RecursiveCleaner::new().run(base_dirs.drain(..));
        }
        Commands::Flatten(flatten_opt) => flattener::run(flatten_opt),
        Commands::Version => {
            print!("{CANDLE_ASCII_ART}\n\n");
            println!("candela v{}\nA personnal qol helper.", VERSION);
        }
    }
}
