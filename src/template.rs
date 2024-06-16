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
                if list_of_files.len() != 0 {
                    self.cmd_apply_from_template_with_list(template_name, list_of_files);
                } else {
                    self.cmd_apply_from_template_ask(template_name);
                }
            }
            _ => {}
        }
    }
    fn cmd_apply_from_template_with_list(&self, template_name: String, list_of_files: Vec<String>) {
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

    fn cmd_apply_from_template_ask(&self, template_name: String) {
        let mut template_folder = self.base_path.clone();
        template_folder.push(&template_name);
        if !template_folder.exists() {
            eprintln!("the template doesn't exist, you can create it by adding files to it usingtemplate add-files");
            return;
        }
        let walker = jwalk::WalkDir::new(&template_folder)
            .skip_hidden(false)
            .sort(true);
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
        let ans = inquire::MultiSelect::new("Files to apply", files)
            .with_all_selected_by_default()
            .prompt();
        if let Ok(ans) = ans {
            ans.iter().for_each(|file_str| {
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
