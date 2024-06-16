use std::cell::RefCell;

mod ccpp;

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub enum FileKind {
    Source,
    Deliverable,
    Temporary,
    OtherElf,
    #[default]
    Other,
}

pub struct Project {
    pub path: std::path::PathBuf,
    pub files: Vec<ccpp::CustomDirEnt>,
    pub artefacts_sizes: std::cell::RefCell<Option<u64>>,
}

impl Project {
    pub fn from_c_project_path(base_path: &std::path::Path) -> Self {
        let files: Vec<ccpp::CustomDirEnt> = ccpp::id_temporary_files(base_path)
            .filter_map(|file| match file {
                Ok(file) => Some(file),
                _ => None,
            })
            .collect();

        Project {
            path: base_path.to_owned(),
            files,
            artefacts_sizes: RefCell::new(None),
        }
    }

    fn compute_artefacts_sizes(&self) {
        let sum: u64 = self
            .files
            .iter()
            .filter(|entry| matches!(entry.client_state, FileKind::Temporary | FileKind::OtherElf))
            .map(|file| file.path())
            .filter_map(|path| path.metadata().ok())
            .map(|meta| meta.len())
            .sum();
        *self.artefacts_sizes.borrow_mut() = Some(sum);
    }

    fn get_or_compute_artefact_sizes(&self) -> u64 {
        let potential_value = *self.artefacts_sizes.borrow();
        if let Some(val) = potential_value {
            val
        } else {
            self.compute_artefacts_sizes();
            self.get_or_compute_artefact_sizes()
        }
    }

    pub fn pretty_print(&self) {
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
        let n_elf = self
            .files
            .iter()
            .filter(|file| file.client_state == FileKind::OtherElf)
            .count();

        println!("- Project {}", self.path.display());
        println!("    - {} temporary files", n_temporary);
        println!("    - {} deliverable files", n_deliverable);
        println!("    - {} ELF files", n_elf);
        println!(
            "    - size of artefacts {} ",
            size::Size::from_bytes(self.get_or_compute_artefact_sizes())
        );
    }
}
