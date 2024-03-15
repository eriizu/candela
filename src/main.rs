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
            // TODO: set read_children_path to none for all children here if
            // we found a project file
            if count != 0 {
                children
                    .iter_mut()
                    .filter_map(|entry| match entry {
                        Ok(entry) if entry.read_children_path != None => Some(entry),
                        _ => None,
                    })
                    .for_each(|entry| entry.read_children_path = None);
            }

            // children.iter_mut().for_each(|dir_entry_result| {
            //     if let Ok(dir_entry) = dir_entry_result {
            //         if is_project_config_file(dir_entry.file_name()) {
            //             dir_entry.client_state = true;
            //         }
            //     }
            // });
        },
    );

    walk_dir
        .into_iter()
        .filter_map(|item| match item {
            Ok(item) if item.client_state => Some(item),
            _ => None,
        })
        .for_each(|item| println!("{}", item.path().display()));
    return;
    for entry in walk_dir {
        if let Ok(entry) = entry {
            println!(
                "{} {}",
                if entry.client_state {
                    "project file "
                } else {
                    ""
                },
                //entry.client_state,
                entry.path().display(),
            );
        }
    }
}

fn walktest() {
    for arg in std::env::args().skip(1) {
        println!("{}:", arg);
        for entry in jwalk::WalkDir::new(arg).sort(true) {
            match entry {
                Ok(dirent) => {
                    println!("{}", dirent.path().display());
                }
                Ok(dirent) if dirent.file_type().is_dir() => {
                    println!("\ndir {}:", dirent.path().display());
                }
                Ok(dirent) => {
                    print!("{} ", dirent.path().file_name().unwrap().to_str().unwrap());
                }
                _ => {}
            }
            //println!("{}", entry.unwrap().path().display())
        }
    }
}
