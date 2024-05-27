use std::{cell::RefCell, ffi::OsStr};

use c_project::FileKind;

mod c_project;
mod find_project_files;

fn clean_project_at_path(mut path: std::path::PathBuf, state: find_project_files::ProjectLang) {
    match state {
        find_project_files::ProjectLang::Yarn => {
            std::process::Command::new("yarn")
                .arg("cache")
                .arg("clean")
                .current_dir(path)
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
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
            std::process::Command::new("cargo")
                .arg("clean")
                .current_dir(path)
                .spawn()
                .unwrap()
                .wait_with_output()
                .unwrap();
        }
        find_project_files::ProjectLang::CCpp => {
            let project = find_temp_and_deliverable_files(path.as_ref());
            let to_remove: Vec<_> = project
                .files
                .iter()
                .filter(|direntry| direntry.client_state == c_project::FileKind::Temporary)
                .map(|direntry| direntry.path())
                .collect();
            if to_remove.len() != 0 {
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
                    to_remove.iter().for_each(|path| {
                        if let Err(err) = std::fs::remove_file(path) {
                            eprintln!("{}", err);
                        }
                    });
                }
            }
        }
    }
}

fn main() {
    for arg in std::env::args().skip(1) {
        find_project_files::iter(&arg)
            .filter_map(|direntry| match direntry.client_state {
                // Some(state) if state != find_project_files::ProjectLang::CCpp => {
                Some(state) => Some((direntry, state)),
                _ => None,
            })
            .for_each(|(direntry, state)| {
                dbg!(&direntry);
                dbg!(state);
                let mut path = direntry.path();
                path.pop();

                clean_project_at_path(path, state)
                // let ans =
                //     inquire::Confirm::new(&format!("Do you want to clean {}", path.display()))
                //         .with_default(true)
                //         .prompt()
                //         .unwrap();
                // if ans {
                //     clean_project_at_path(path, state)
                // }
            });
        // TODO: do something with found projects
        // ie. for rust, check if there is a "target" dir, and propose cleaning
        // it.

        // find_all_ccpp_projects(&arg).for_each(|item| {
        //     let mut path = item.path().clone();
        //     path.pop();
        //     find_temp_and_deliverable_files(&path);
        // });
    }
}

fn find_all_ccpp_projects(
    base_path: &str,
) -> impl Iterator<Item = find_project_files::CustomDirEnt> {
    find_project_files::iter(&base_path).filter(|dir_entry| {
        if let Some(find_project_files::ProjectLang::CCpp) = dir_entry.client_state {
            return true;
        }
        return false;
    })
}

struct Project {
    path: std::path::PathBuf,
    files: Vec<c_project::CustomDirEnt>,
    artefacts_sizes: RefCell<Option<u64>>,
}

impl Project {
    fn compute_artefacts_sizes(&self) {
        let sum: u64 = self
            .files
            .iter()
            .map(|file| file.path())
            .filter_map(|path| path.metadata().ok())
            .map(|meta| meta.len())
            .sum();
        *self.artefacts_sizes.borrow_mut() = Some(sum);
    }

    fn get_or_compute_artefact_sizes(&self) -> u64 {
        let potential_value = *self.artefacts_sizes.borrow();
        if let Some(val) = potential_value {
            return val;
        } else {
            self.compute_artefacts_sizes();
            return self.get_or_compute_artefact_sizes();
        }
    }
    fn print_temp_and_deliverables(&self) {
        self.files
            .iter()
            .filter(|file| match file.client_state {
                FileKind::Temporary | FileKind::Deliverable => true,
                _ => false,
            })
            .for_each(|file| {
                if let Ok(tmp) = file.path().strip_prefix(&self.path) {
                    println!("- {} {}", tmp.display(), file.client_state)
                }
            });
    }

    fn pretty_print(&self) {
        let n_temporary = self
            .files
            .iter()
            .filter(|file| file.client_state == FileKind::Temporary)
            .count();
        let n_deliverable = self
            .files
            .iter()
            .filter(|file| file.client_state == FileKind::Deliverable)
            .count();

        println!("- Project {}", self.path.display());
        println!("    - {} temporary files", n_temporary);
        println!("    - {} deliverable files", n_deliverable);
        println!(
            "    - size of artefacts {} ",
            self.get_or_compute_artefact_sizes()
        );
    }
}

fn find_temp_and_deliverable_files(base_path: &std::path::Path) -> Project {
    // println!("\n:: in project {}", base_path.to_str().unwrap());
    let files: Vec<c_project::CustomDirEnt> = c_project::id_temporary_files(base_path)
        .filter_map(|file| match file {
            Ok(file) => Some(file),
            _ => None,
        })
        .collect();

    let proj = Project {
        path: base_path.to_owned(),
        files,
        artefacts_sizes: RefCell::new(None),
    };
    proj.pretty_print();
    proj
    // let ans = inquire::Confirm::new("Do you want to delete temporary files?")
    //     .with_default(true)
    //     .prompt();
    // match ans {
    //     Ok(true) => println!("i can't delete them for now, but someday i will"),
    //     Ok(false) => println!("not deleting"),
    //     Err(inquire::InquireError::OperationCanceled) => {
    //         std::process::exit(0);
    //     }
    //     Err(inquire::InquireError::OperationInterrupted) => {
    //         std::process::exit(1);
    //     }
    //     _ => eprintln!("inquiry error that wasn't expected, moving on..."),
    // }

    // proj.print_temp_and_deliverables();
}
