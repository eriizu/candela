mod find_project_files;

fn main() {
    for arg in std::env::args().skip(1) {
        // TODO: do something with found projects
        // ie. for rust, check if there is a "target" dir, and propose cleaning
        // it.
        find_project_files::iter(&arg)
            .filter_map(|item| match item {
                Ok(item) if item.client_state => Some(item),
                _ => None,
            })
            .for_each(|item| println!("{}", item.path().display()));
    }
}
