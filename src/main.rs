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

fn find_temp_and_deliverable_files(base_path: &std::path::Path) {
    c_project::id_temporary_files(base_path)
        .filter_map(|file| match file {
            Ok(file) => Some(file),
            _ => None,
        })
        .filter(|file| match file.client_state {
            FileKind::Temporary | FileKind::Deliverable => true,
            _ => false,
        })
        .for_each(|file| println!("{} {}", file.path().display(), file.client_state));
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
