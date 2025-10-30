use std::cmp::Ordering;

// use crate::c_project::CustomDirEnt;
type CustomState = (usize, Option<ProjectLang>);
pub type CustomDirEnt = jwalk::DirEntry<CustomState>;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectLang {
    CCpp,
    Rust,
    Yarn,
    Npm,
}

struct ProjectMatcher {
    lang: ProjectLang,
    file: &'static str,
}

static MATCHERS: [ProjectMatcher; 5] = [
    ProjectMatcher {
        lang: ProjectLang::CCpp,
        file: "Makefile",
    },
    ProjectMatcher {
        lang: ProjectLang::Rust,
        file: "Cargo.toml",
    },
    ProjectMatcher {
        lang: ProjectLang::Yarn,
        file: "yarn.lock",
    },
    ProjectMatcher {
        lang: ProjectLang::Npm,
        file: "package-lock.json",
    },
    ProjectMatcher {
        lang: ProjectLang::Npm,
        file: "pnpm-lock.yaml",
    },
];

fn get_project_lang(file_name: &std::ffi::OsStr) -> Option<ProjectLang> {
    let file_name = file_name.to_str()?;
    MATCHERS.iter().find_map(|matcher| {
        if matcher.file == file_name {
            Some(matcher.lang)
        } else {
            None
        }
    })
}

pub fn iter(dir: &str) -> impl Iterator<Item = CustomDirEnt> {
    let walk_dir = jwalk::WalkDirGeneric::<CustomState>::new(dir).process_read_dir(
        |_depth, _path, _read_dir_state, children| {
            // INFO: base usage for this callback
            // https://docs.rs/jwalk/0.8.1/jwalk/index.html#extended-example

            children.sort_by(sort_predicate);

            let mut count: usize = 0;
            children.iter_mut().for_each(|dir_ent_result| {
                let _ = dir_ent_result.as_mut().map(|dir_entry| {
                    dir_entry.client_state = get_project_lang(dir_entry.file_name());
                    if dir_entry.client_state.is_some() {
                        count += 1;
                    }
                });
            });
            // INFO: if at least one child indicates to us that we are in a
            // codebase, don't bother reading subdirectories.
            if count != 0 {
                children.iter_mut().for_each(|dir_ent_res| {
                    let _ = dir_ent_res
                        .as_mut()
                        .map(|dir_entry| dir_entry.read_children_path = None);
                });
            }
        },
    );
    walk_dir.into_iter().filter_map(|dirent| dirent.ok())
}
