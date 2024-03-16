use std::{cmp::Ordering, ffi::OsStr, usize};

fn main() {
    for arg in std::env::args().skip(1) {
        walkgeneric_test(&arg);
    }
}

type CustomState = (usize, bool);
type CustomDirEnt = jwalk::DirEntry<CustomState>;

// TODO: sort folders first?
fn sort_predicate(
    lhs: &jwalk::Result<CustomDirEnt>,
    rhs: &jwalk::Result<CustomDirEnt>,
) -> std::cmp::Ordering {
    match (lhs, rhs) {
        (Ok(a), Ok(b)) => a.file_name.cmp(&b.file_name),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => Ordering::Equal,
    }
}

const PROJ_CONFIG_FILESS: &'static [&'static str] = &["package.json", "cargo.toml", "Makefile"];

fn is_project_config_file(filename: &std::ffi::OsStr) -> bool {
    if let Ok(filename) = filename.try_into() {
        PROJ_CONFIG_FILESS.contains(&filename)
    } else {
        false
    }
}

#[cfg(test)]
mod test_is_project_config_file {
    #[test]
    fn case_a() {
        let os_filename = std::ffi::OsString::from("package.json");
        assert_eq!(super::is_project_config_file(&os_filename), true);
    }
}

fn walkgeneric_test(dir: &str) {
    let walk_dir = jwalk::WalkDirGeneric::<CustomState>::new(dir).process_read_dir(
        |_depth, _path, _read_dir_state, children| {
            // INFO: base usage for this callback
            // https://docs.rs/jwalk/0.8.1/jwalk/index.html#extended-example

            children.sort_by(sort_predicate);

            let mut count: usize = 0;
            children
                .iter_mut()
                .filter_map(|entry| match entry {
                    Ok(entry) if is_project_config_file(entry.file_name()) => {
                        count += 1;
                        Some(entry)
                    }
                    _ => None,
                })
                .for_each(|entry| entry.client_state = true);
            if count != 0 {
                children
                    .iter_mut()
                    .filter_map(|entry| match entry {
                        Ok(entry) if entry.read_children_path != None => Some(entry),
                        _ => None,
                    })
                    .for_each(|entry| entry.read_children_path = None);
            }
        },
    );

    // TODO: do something with found projects
    // ie. for rust, check if there is a "target" dir, and propose cleaning
    // it.
    walk_dir
        .into_iter()
        .filter_map(|item| match item {
            Ok(item) if item.client_state => Some(item),
            _ => None,
        })
        .for_each(|item| println!("{}", item.path().display()));
}
