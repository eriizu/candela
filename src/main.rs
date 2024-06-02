mod find_project_files;
mod is_elf;
mod project;

use project::Project;

use spinoff::{spinners, Spinner};

struct JWalkCleaner {
    spinner: spinoff::Spinner,
    n_processed: u32,
}

impl JWalkCleaner {
    fn new() -> Self {
        Self {
            spinner: Spinner::new(spinners::Dots, "Scaning and deleting...", None),
            n_processed: 0,
        }
    }
    fn restart_spinner(&mut self) {
        self.spinner = Spinner::new(spinners::Dots, "Scaning and deleting...", None);
    }

    fn run<T>(&mut self, paths_to_search: T)
    where
        T: Iterator<Item = String>,
    {
        for arg in paths_to_search {
            find_project_files::iter(&arg)
                .filter_map(|direntry| match direntry.client_state {
                    // Some(state) if state != find_project_files::ProjectLang::CCpp => {
                    Some(state) => Some((direntry, state)),
                    _ => None,
                })
                .for_each(|(direntry, state)| {
                    let mut path = direntry.path();
                    path.pop();
                    // self.spinner
                    //     .update_text(format!("processing: {}", path.display()));
                    self.clean_project_at_path(path, state);
                    // self.spinner.update_text("scanning for next project...");
                    self.n_processed += 1;
                });
        }
        self.spinner.success(
            format!(
                "Processed {} project folders. Thanks for using me!",
                self.n_processed
            )
            .as_ref(),
        );
    }

    fn clean_project_at_path(
        &mut self,
        mut path: std::path::PathBuf,
        state: find_project_files::ProjectLang,
    ) {
        match state {
            find_project_files::ProjectLang::Yarn => {
                let mut cmd = std::process::Command::new("yarn");
                cmd.arg("cache").arg("clean").current_dir(path);
                self.spawn_and_wait_command(cmd);
            }
            find_project_files::ProjectLang::Npm => {
                path.push("node_modules");
                if path.exists() {
                    if let Err(err) = std::fs::remove_dir_all(path) {
                        eprintln!("{}", err);
                    }
                }
            }
            find_project_files::ProjectLang::Rust => {
                let mut cmd = std::process::Command::new("cargo");
                cmd.arg("clean").current_dir(path);
                self.spawn_and_wait_command(cmd);
            }
            find_project_files::ProjectLang::CCpp => {
                self.process_ccpp(path);
            }
        }
    }

    fn process_ccpp(&mut self, path: std::path::PathBuf) {
        let project = Project::from_c_project_path(path.as_ref());
        let to_remove: Vec<_> = project
            .files
            .iter()
            // .filter(|direntry| direntry.client_state == project::FileKind::Temporary)
            .filter(|entry| match entry.client_state {
                project::FileKind::Temporary | project::FileKind::OtherElf => true,
                _ => false,
            })
            .map(|direntry| direntry.path())
            .collect();
        if to_remove.len() != 0 {
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
            let ans = inquire::Confirm::new("proceed ?")
                .with_default(true)
                .prompt()
                .unwrap();
            if ans {
                self.restart_spinner();
                to_remove.iter().for_each(|path| {
                    if let Err(err) = std::fs::remove_file(path) {
                        eprintln!("\r{}", err);
                    }
                });
            } else {
                self.restart_spinner();
            }
        }
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

fn main() {
    JWalkCleaner::new().run(std::env::args().skip(1));
}
