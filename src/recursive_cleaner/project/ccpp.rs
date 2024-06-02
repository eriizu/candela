use regex::Regex;

use super::FileKind;

impl Default for FileKind {
    fn default() -> Self {
        FileKind::Other
    }
}

impl std::fmt::Display for FileKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let to_write = match self {
            Self::Source => "Source",
            Self::Deliverable => "Deliverable",
            Self::Temporary => "Temporary",
            Self::Other => "Other",
            Self::OtherElf => "Other Elf",
        };
        f.write_str(to_write)
    }
}

pub type CustomState = (usize, FileKind);
pub type CustomDirEnt = jwalk::DirEntry<CustomState>;

// INFO: regexes that we use:
// produced files: /.+\.(a,out,so)/
// temporary files: /.+\.(?:o|gch)/

struct FileMatcher {
    regex: Regex,
    kind: FileKind,
}

// INFO: we need to retain artefacts files and all folders
fn is_to_retain(dir_entry: &CustomDirEnt) -> bool {
    dir_entry.file_type().is_dir() || dir_entry.client_state != FileKind::Other
}

static MATCHERS: once_cell::sync::Lazy<[FileMatcher; 3]> = once_cell::sync::Lazy::new(|| {
    [
        FileMatcher {
            regex: Regex::new(r".+\.(a|out|so)$").unwrap(),
            kind: FileKind::Deliverable,
        },
        FileMatcher {
            regex: Regex::new(r".+\.(o|gch)$").unwrap(),
            kind: FileKind::Temporary,
        },
        FileMatcher {
            regex: Regex::new(r".+\.(c|h|cpp|hpp|cc|hh)$").unwrap(),
            kind: FileKind::Source,
        },
    ]
});

fn tag_file(dir_entry: &mut CustomDirEnt) {
    let Some(file_name) = dir_entry.file_name().to_str() else {
        return;
    };
    match MATCHERS.iter().find_map(|matcher| {
        if matcher.regex.is_match(file_name) {
            Some(matcher.kind)
        } else {
            None
        }
    }) {
        Some(kind) => dir_entry.client_state = kind,
        _ => {
            if super::super::is_elf::is_elf(dir_entry.path()).unwrap_or(false) {
                dir_entry.client_state = FileKind::OtherElf;
            }
        }
    };
}

pub fn id_temporary_files(
    dir: &std::path::Path,
) -> impl Iterator<Item = Result<CustomDirEnt, jwalk::Error>> {
    let walk_dir = jwalk::WalkDirGeneric::<CustomState>::new(dir).process_read_dir(
        |_depth, _path, _read_dir_state, children| {
            children
                .iter_mut()
                .filter_map(|dir_ent_result| match dir_ent_result {
                    Ok(dir_entry) if dir_entry.file_type().is_file() => Some(dir_entry),
                    _ => None,
                })
                .for_each(tag_file);
            children.retain(|dir_entry_result| {
                dir_entry_result.as_ref().map(is_to_retain).unwrap_or(false)
            });
        },
    );

    // INFO: afterwards, we can filter out the folders
    walk_dir.into_iter().filter(|elem| {
        elem.as_ref()
            .map(|dir_entry| dir_entry.file_type().is_file())
            .unwrap_or(false)
    })
}
