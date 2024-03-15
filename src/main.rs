use std::cmp::Ordering;

fn main() {
    for arg in std::env::args().skip(1) {
        walkgeneric_test(&arg);
    }
}

type CustomState = (usize, usize);
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

fn walkgeneric_test(dir: &str) {
    let walk_dir = jwalk::WalkDirGeneric::<CustomState>::new(dir)
        .process_read_dir(|_depth, _path, read_dir_state, children| {
            // 1. Custom sort
            children.sort_by(sort_predicate);
            // 2. Custom filter
            // children.retain(|dir_entry_result| {
            //     dir_entry_result
            //         .as_ref()
            //         .map(|dir_entry| {
            //             dir_entry
            //                 .file_name
            //                 .to_str()
            //                 .map(|s| !s.starts_with('.'))
            //                 .unwrap_or(false)
            //         })
            //         .unwrap_or(false)
            // });
            // 3. Custom skip
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    if dir_entry.depth == 2 {
                        dir_entry.read_children_path = None;
                    }
                }
            });
            // 4. Custom state
            // dbg!(*read_dir_state);
            *read_dir_state += 1;
            let a = *read_dir_state;
            children.iter_mut().for_each(|dir_entry_result| {
                if let Ok(dir_entry) = dir_entry_result {
                    dir_entry.client_state = a;
                }
            });
        })
        .root_read_dir_state(0);

    for entry in walk_dir {
        if let Ok(entry) = entry {
            println!(
                "{} {}",
                //if entry.client_state { "first " } else { "" },
                entry.client_state,
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
