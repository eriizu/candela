use std::{borrow::BorrowMut, cell::RefCell};

use c_project::FileKind;

mod c_project;
mod find_project_files;

fn find_all_ccpp_projects(
    base_path: &str,
) -> impl Iterator<Item = find_project_files::CustomDirEnt> {
    find_project_files::iter(&base_path)
        .filter_map(|dir_entry| dir_entry.ok())
        .filter(|dir_entry| {
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

fn find_temp_and_deliverable_files(base_path: &std::path::Path) {
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
    // proj.print_temp_and_deliverables();
}

fn main() {
    for arg in std::env::args().skip(1) {
        // TODO: do something with found projects
        // ie. for rust, check if there is a "target" dir, and propose cleaning
        // it.
        find_all_ccpp_projects(&arg).for_each(|item| {
            let mut path = item.path().clone();
            path.pop();
            find_temp_and_deliverable_files(&path);
        });
    }
}
