use regex::Regex;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum FileKind {
    Source,
    Deliverable,
    Temporary,
    Other,
}

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
        };
        f.write_str(to_write)
    }
}

type CustomState = (usize, FileKind);
type CustomDirEnt = jwalk::DirEntry<CustomState>;

// TODO: use regexes for matching a whole pannel of files
// produced files: /.+\.(a,out,so)/
// temporary files: /.+\.(?:o|gch)/

struct FileMatcher2 {
    regex: Regex,
    kind: FileKind,
}

// INFO: we need to retain .o files and all folders
fn is_to_retain(dir_entry: &CustomDirEnt) -> bool {
    dir_entry.file_type().is_dir() || dir_entry.client_state != FileKind::Other
}

static MATCHERS: once_cell::sync::Lazy<[FileMatcher2; 3]> = once_cell::sync::Lazy::new(|| {
    [
        FileMatcher2 {
            regex: Regex::new(r".+\.(a|out|so)$").unwrap(),
            kind: FileKind::Deliverable,
        },
        FileMatcher2 {
            regex: Regex::new(r".+\.(o|gch)$").unwrap(),
            kind: FileKind::Temporary,
        },
        FileMatcher2 {
            regex: Regex::new(r".+\.(c|h|cpp|hpp|cc|hh)$").unwrap(),
            kind: FileKind::Other,
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
        _ => {}
    };
    // if RE_ARTEFACTS_TEMP.is_match(file_name) {
    //     dir_entry.client_state = FileKind::Temporary;
    // } else if RE_ARTEFACTS_PRODUCED.is_match(file_name) {
    //     dir_entry.client_state = FileKind::Deliverable;
    // } else if RE_SOURCE.is_match(file_name) {
    //     dir_entry.client_state = FileKind::Source;
    // }
}

pub fn id_temporary_files(
    dir: std::path::PathBuf,
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

    // INFO: afterwards, we can filter out the folder
    walk_dir.into_iter().filter(|elem| {
        elem.as_ref()
            .map(|dir_entry| dir_entry.file_type().is_file())
            .unwrap_or(false)
    })
}
