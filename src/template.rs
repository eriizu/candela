use std::{io::Write, path::PathBuf};

use clap::Parser;
#[derive(Debug, Parser)]
#[command()]
pub struct TemplateCli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    Apply {
        template_name: String,
        list_of_files: Vec<String>,
    },
    AddFiles {
        template_name: String,
        #[arg(value_hint = clap::ValueHint::FilePath)]
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
    Ls,
}

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

    pub fn run(&self, command: Command) {
        match command {
            Command::AddFiles {
                template_name,
                list_of_files,
            } => {
                self.cmd_add_files(template_name, list_of_files);
            }
            Command::Apply {
                template_name,
                list_of_files,
            } => {
                if let Some(template_folder) = self.get_template_folder(template_name) {
                    if !list_of_files.is_empty() {
                        self.cmd_apply_from_template_with_list(template_folder, list_of_files);
                    } else {
                        self.cmd_apply_from_template_ask(template_folder);
                    }
                }
            }
            Command::Ls => {
                let Ok(read_dir) = std::fs::read_dir(&self.base_path) else {
                    return;
                };
                let mut paths = read_dir
                    .into_iter()
                    .filter_map(|dir_ent_res| dir_ent_res.ok())
                    .filter_map(|dir_ent| {
                        dir_ent
                            .path()
                            .file_name()
                            .map(|file_name| file_name.to_os_string())
                    })
                    .filter(|file_name| file_name.as_encoded_bytes()[0] != b'.')
                    .collect::<Vec<_>>();
                paths.sort();
                paths.iter().for_each(|item| {
                    let _ = std::io::stdout().write(item.as_encoded_bytes());
                    let _ = std::io::stdout().write(b"\n").unwrap();
                });
            }
            _ => {
                todo!();
            }
        }
    }

    fn get_template_folder(&self, template_name: String) -> Option<PathBuf> {
        let mut template_folder = self.base_path.clone();
        let comp_number = template_folder.components().count();
        template_folder.push(&template_name);
        template_folder = template_folder.canonicalize().unwrap();
        assert_eq!(comp_number + 1, template_folder.components().count());
        if !template_folder.exists() {
            eprintln!("the template doesn't exist, you can create it by adding files to it using template add-files");
            None
        } else {
            Some(template_folder)
        }
    }

    fn cmd_apply_from_template_with_list(
        &self,
        template_folder: PathBuf,
        list_of_files: Vec<String>,
    ) {
        list_of_files.iter().for_each(|file| {
            let mut in_template_file = template_folder.clone();
            in_template_file.push(file.as_str());
            if let Err(err) = std::fs::copy(in_template_file, file) {
                eprintln!("{err}");
            }
        });
    }

    fn cmd_apply_from_template_ask(&self, template_folder: PathBuf) {
        let walker = jwalk::WalkDir::new(&template_folder)
            .skip_hidden(false)
            .sort(true);
        let paths: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|pot_direntry| pot_direntry.ok())
            .filter(|dirent| !dirent.path().is_dir())
            .map(|dirent| dirent.path())
            .collect();
        let rel_file_string: Vec<String> = paths
            .iter()
            .map(|path| {
                if let Some(relative_comps) =
                    crate::flattener::comps_after_root(path, &template_folder)
                {
                    let relative_path: PathBuf = relative_comps.collect();
                    return relative_path.to_str().unwrap().to_owned();
                }
                path.to_str().unwrap().to_owned()
            })
            .collect();

        let map = {
            let mut map: std::collections::HashMap<String, PathBuf> =
                std::collections::HashMap::new();
            rel_file_string
                .iter()
                .zip(paths.iter())
                .for_each(|(key, value)| {
                    map.insert(key.clone(), value.clone());
                });
            map
        };
        let ans = inquire::MultiSelect::new("Files to apply", rel_file_string)
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
