pub mod disk_usage;
mod duplicates;
mod flattener;
mod recursive_cleaner;
mod template;

use clap::{CommandFactory, Parser};

fn main() {
    let opt = Cli::parse();
    match opt.command {
        Commands::Clean {
            mut base_dirs,
            force,
        } => {
            recursive_cleaner::RecursiveCleaner::new(force).run(base_dirs.drain(..));
        }
        Commands::Flatten(flatten_opt) => flattener::run(flatten_opt),
        Commands::Version => {
            print!("{CANDLE_ASCII_ART}\n\n");
            println!("candela v{}\nA personnal qol helper.", VERSION);
        }
        Commands::Template(opt) => {
            template::Executor::new().run(opt.command);
        }
        Commands::Completions { shell } => {
            shell.generate(&mut Cli::command(), &mut std::io::stdout());
        }
        Commands::Duplicates(cli) => cli.run(),
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
// INFO: ASCII candle copyright of Joan Stack
const CANDLE_ASCII_ART: &str = r#"
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
        #[arg(short, long)]
        force: bool,

        #[arg(value_hint = clap::ValueHint::DirPath)]
        base_dirs: Vec<String>,
    },
    Flatten(flattener::Cli),
    Version,
    Template(template::TemplateCli),
    Completions {
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
    Duplicates(duplicates::Cli),
}
