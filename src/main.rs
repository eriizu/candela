mod flattener;
mod recursive_cleaner;

use clap::Parser;

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
        #[arg(short, long)]
        force: bool,

        base_dirs: Vec<String>,
    },
    Flatten(flattener::Cli),
    Version,
    Templates(template::TemplateCli),
}

mod template {
    use std::path::PathBuf;

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

    // TODO: maybe merge executor and templatecli structs?
    pub struct Executor {
        base_path: std::path::PathBuf,
    }

    impl Executor {
        pub fn new() -> Self {
            let home = std::env::var("HOME").expect("user should have a HOME");
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
                    self.cmd_add_files(template_name, list_of_files);
                }
                Commands::Apply {
                    template_name,
                    list_of_files,
                } => {
                    self.cmd_apply_from_template(template_name, list_of_files);
                }
                _ => {}
            }
        }

        fn cmd_apply_from_template(&self, template_name: String, list_of_files: Vec<String>) {
            let mut template_folder = self.base_path.clone();
            template_folder.push(&template_name);
            if !template_folder.exists() {
                eprintln!("the template doesn't exist, you can create it by adding files to it usingtemplate add-files");
                return;
            }
            let walker = jwalk::WalkDir::new(&template_folder)
                .skip_hidden(false)
                .sort(true);
            // let files: Vec<String> = walker
            //     .into_iter()
            //     .filter_map(|pot_direntry| pot_direntry.ok())
            //     .filter(|dirent| !dirent.path().is_dir())
            //     .filter_map(|dirent| dirent.path().to_str().map(|s| s.to_owned()))
            //     .collect();
            let paths: Vec<PathBuf> = walker
                .into_iter()
                .filter_map(|pot_direntry| pot_direntry.ok())
                .filter(|dirent| !dirent.path().is_dir())
                .map(|dirent| dirent.path())
                .collect();
            let files: Vec<String> = paths
                .iter()
                .map(|path| {
                    if let Some(relative_comps) =
                        crate::flattener::comps_after_root(&path, &template_folder)
                    {
                        let relative_path: PathBuf = relative_comps.collect();
                        return relative_path.to_str().unwrap().to_owned();
                    }
                    return path.to_str().unwrap().to_owned();
                })
                .collect();

            let map = {
                let mut map: std::collections::HashMap<String, PathBuf> =
                    std::collections::HashMap::new();
                files.iter().zip(paths.iter()).for_each(|(key, value)| {
                    map.insert(key.clone(), value.clone());
                });
                map
            };
            // let ans: Result<&str, InquireError> = Select::new("Depart from?", stops).prompt();
            // let formatter: inquire::formatter::MultiOptionFormatter<'_, &str> =
            //     &|a: std::path::PathBuf| {
            //         let mut refpath = a.as_path();
            //         if let Some(components) =
            //             crate::flattener::comps_after_root(refpath, &template_folder)
            //         {
            //             let relative_path: PathBuf = components.collect();
            //             relative_path.to_str().unwrap()
            //         } else {
            //             refpath.to_str().unwrap()
            //         }
            //         // refpath
            //         // format!("{} different fruits", a.len())
            //     };
            let ans = inquire::MultiSelect::new("Files to apply", files).prompt();
            if let Ok(ans) = ans {
                ans.iter().for_each(|file_str| {
                    // let mut in_template_file = template_folder.clone();
                    // in_template_file.push(file.as_str());
                    let dest = std::path::PathBuf::from(file_str);
                    let src = map.get(file_str).unwrap();
                    if let Err(err) = std::fs::copy(src, &dest) {
                        eprintln!("{err}");
                    } else {
                        println!("{} -> {}", &src.display(), &dest.display());
                    }
                });
            }
        }

        fn cmd_add_files(&self, template_name: String, list_of_files: Vec<String>) {
            let mut template_folder = self.base_path.clone();
            template_folder.push(&template_name);
            if !template_folder.exists() {
                if let Err(err) = std::fs::create_dir_all(&template_folder) {
                    eprintln!("{err}");
                    return;
                }
            }
            list_of_files.iter().for_each(|file| {
                let mut in_template_file = template_folder.clone();
                in_template_file.push(file.as_str());
                if let Err(err) = std::fs::copy(file, &in_template_file) {
                    eprintln!("{err}");
                }
            });
        }
    }

    fn remove_common_parts<'a>(
        path1: &'a std::path::Path,
        path2: &'a std::path::Path,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let mut components1 = path1.components();
        let mut components2 = path2.components();

        let mut common_components = vec![];

        while let (Some(comp1), Some(comp2)) = (components1.next(), components2.next()) {
            if comp1 == comp2 {
                common_components.push(comp1);
            } else {
                break;
            }
        }

        // Reconstruct the paths without the common prefix
        let remaining_path1: std::path::PathBuf = components1.collect();
        let remaining_path2: std::path::PathBuf = components2.collect();

        (remaining_path1, remaining_path2)
    }
}
