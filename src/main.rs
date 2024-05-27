use std::{cell::RefCell, ffi::OsStr};

mod find_project_files;
mod project;

use project::Project;

fn clean_project_at_path(mut path: std::path::PathBuf, state: find_project_files::ProjectLang) {
    println!("\n:: in {}", path.display());
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
            let project = Project::from_c_project_path(path.as_ref());
            project.pretty_print();
            let to_remove: Vec<_> = project
                .files
                .iter()
                .filter(|direntry| direntry.client_state == project::FileKind::Temporary)
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
                let mut path = direntry.path();
                path.pop();
                clean_project_at_path(path, state)
            });
    }
}
