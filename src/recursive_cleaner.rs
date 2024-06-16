mod find_project_files;
mod is_elf;
mod project;

use spinoff::{spinners, Spinner};

pub struct RecursiveCleaner {
    spinner: spinoff::Spinner,
    n_processed: u32,
    n_cleaned: u32,
    force: bool,
}

impl RecursiveCleaner {
    pub fn new(force: bool) -> Self {
        Self {
            spinner: Spinner::new(spinners::Dots, "Scaning and deleting...", None),
            n_processed: 0,
            n_cleaned: 0,
            force,
        }
    }
    fn restart_spinner(&mut self) {
        self.spinner = Spinner::new(spinners::Dots, "Scaning and deleting...", None);
    }

    pub fn run<T>(&mut self, paths_to_search: T)
    where
        T: Iterator<Item = String>,
    {
        for arg in paths_to_search {
            find_project_files::iter(&arg)
                .filter_map(|direntry| direntry.client_state.map(|state| (direntry, state)))
                .for_each(|(direntry, state)| {
                    let mut path = direntry.path();
                    path.pop();
                    if self.clean_project_at_path(path, state) {
                        self.n_cleaned += 1;
                    }
                    self.n_processed += 1;
                });
        }
        self.spinner_success();
    }

    fn spinner_success(&mut self) {
        if self.n_cleaned != 0 {
            self.spinner.success(
                format!(
                    "Cleaned {} out of {} project. Thanks for using me!",
                    self.n_cleaned, self.n_processed
                )
                .as_ref(),
            );
        } else {
            self.spinner.success(
                format!(
                    "Scanned {} project folders, nothing to clean. Thanks for using me!",
                    self.n_processed
                )
                .as_ref(),
            );
        }
    }

    fn clean_project_at_path(
        &mut self,
        mut path: std::path::PathBuf,
        state: find_project_files::ProjectLang,
    ) -> bool {
        let mut has_cleaned_something = false;
        match state {
            find_project_files::ProjectLang::Yarn => {
                let node_modules_path = {
                    let mut tmp = path.to_owned();
                    tmp.push("node_modules");
                    tmp
                };
                let yarn_install_state_path = {
                    let mut tmp = path.to_owned();
                    tmp.push(".yarn");
                    tmp.push("install-state.gz");
                    tmp
                };
                let yarn_unpluged_path = {
                    let mut tmp = path.to_owned();
                    tmp.push(".yarn");
                    tmp.push("unplugged");
                    tmp
                };
                let mut cmd = std::process::Command::new("yarn");
                cmd.arg("cache").arg("clean").arg("--all").current_dir(path);
                self.spawn_and_wait_command(cmd);
                if yarn_install_state_path.exists() {
                    if let Err(err) = std::fs::remove_file(yarn_install_state_path) {
                        eprintln!("{}", err);
                    }
                    has_cleaned_something = true;
                }
                if yarn_unpluged_path.exists() {
                    if let Err(err) = std::fs::remove_dir_all(yarn_unpluged_path) {
                        eprintln!("{}", err);
                    }
                    has_cleaned_something = true;
                }
                if node_modules_path.exists() {
                    if let Err(err) = std::fs::remove_dir_all(node_modules_path) {
                        eprintln!("{}", err);
                    }
                    has_cleaned_something = true;
                }
            }
            find_project_files::ProjectLang::Npm => {
                path.push("node_modules");
                if path.exists() {
                    if let Err(err) = std::fs::remove_dir_all(path) {
                        eprintln!("{}", err);
                    }
                    has_cleaned_something = true;
                }
            }
            find_project_files::ProjectLang::Rust => {
                let target_path = {
                    let mut tmp = path.to_owned();
                    tmp.push("target");
                    tmp
                };
                if target_path.exists() {
                    let mut cmd = std::process::Command::new("cargo");
                    cmd.arg("clean").current_dir(path);
                    self.spawn_and_wait_command(cmd);
                    has_cleaned_something = true;
                }
            }
            find_project_files::ProjectLang::CCpp => {
                has_cleaned_something = self.process_ccpp(path);
            }
        }
        has_cleaned_something
    }

    fn process_ccpp(&mut self, path: std::path::PathBuf) -> bool {
        let project = project::Project::from_c_project_path(path.as_ref());
        let to_remove: Vec<_> = project
            .files
            .iter()
            .filter(|entry| {
                matches!(
                    entry.client_state,
                    project::FileKind::Temporary | project::FileKind::OtherElf
                )
            })
            .map(|direntry| direntry.path())
            .collect();
        if !to_remove.is_empty() {
            self.spinner.stop();
            print!("\r");
            project.pretty_print();
            println!("about to remove:");
            to_remove.iter().for_each(|path| {
                if let Some(os_file_name) = path.file_name() {
                    if let Some(file_name) = os_file_name.to_str() {
                        println!("{}", file_name);
                    } else {
                        eprintln!("Failed to convert one filename to str");
                    }
                } else {
                    eprintln!("Failed to extract a filename from path");
                }
            });
            let ans = if self.force {
                true
            } else {
                inquire::Confirm::new("proceed ?")
                    .with_default(true)
                    .prompt()
                    .unwrap()
            };
            self.restart_spinner();
            if ans {
                to_remove.iter().for_each(|path| {
                    if let Err(err) = std::fs::remove_file(path) {
                        eprintln!("\r{}", err);
                    }
                });
                return true;
            }
        }
        false
    }
    fn spawn_and_wait_command(&mut self, mut cmd: std::process::Command) {
        if let Ok(output) = cmd.output() {
            if !output.status.success() {
                self.spinner.fail("failed cleanup command");
                eprintln!(
                    "{} exited with status: {}",
                    cmd.get_program().to_str().unwrap_or("?"),
                    output.status
                );
                use std::io::Write;
                std::io::stderr().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();
                self.restart_spinner();
            }
        }
    }
}
