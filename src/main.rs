mod flattener;
mod recursive_cleaner;

use clap::Parser;

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
        Commands::Templates(opt) => {
            template::Executor::new().run(opt.command);
        }
    }
}

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
    Templates(template::TemplateCli),
}

mod template {
    use clap::Parser;
    #[derive(Debug, Parser)]
    #[command()]
    pub struct TemplateCli {
        #[command(subcommand)]
        pub command: Commands,
    }

    #[derive(Debug, Parser)]
    pub enum Commands {
        Apply {
            template_name: String,
            list_of_files: Vec<String>,
        },
        AddFiles {
            template_name: String,
            list_of_files: Vec<String>,
        },
        Cp {
            source_template: String,
            dest_template: String,
        },
        Mv {
            source_template: String,
            dest_template: String,
        },
        Rm {
            template_name: String,
        },
    }

    pub struct Executor {
        base_path: std::path::PathBuf,
    }

    impl Executor {
        pub fn new() -> Self {
            let home = std::env::var("HOME").unwrap();
            let mut base_path = std::path::PathBuf::from(home);
            base_path.push(".config");
            base_path.push("candela");
            base_path.push("templates");
            Self { base_path }
        }

        pub fn run(&self, command: Commands) {
            match command {
                Commands::AddFiles {
                    template_name,
                    list_of_files,
                } => {
                    let mut template_folder = self.base_path.clone();
                    template_folder.push(&template_name);
                    if !template_folder.exists() {
                        std::fs::create_dir_all(&template_folder).unwrap();
                    }
                    list_of_files.iter().for_each(|file| {
                        let mut in_template_file = template_folder.clone();
                        in_template_file.push(file.as_str());
                        if let Err(err) = std::fs::copy(file, &in_template_file) {
                            eprintln!("{err}");
                        }
                    });
                }
                Commands::Apply {
                    template_name,
                    list_of_files,
                } => {
                    let mut template_folder = self.base_path.clone();
                    template_folder.push(&template_name);
                    if !template_folder.exists() {
                        eprintln!("the template doesn't exist, you can create it by adding files to it usingtemplate add-files");
                        return;
                    }
                    list_of_files.iter().for_each(|file| {
                        let mut in_template_file = template_folder.clone();
                        in_template_file.push(file.as_str());
                        if let Err(err) = std::fs::copy(in_template_file, &file) {
                            eprintln!("{err}");
                        }
                    });
                }
                _ => {}
            }
        }
    }
}
