type CustomState = (usize, bool);
type CustomDirEnt = jwalk::DirEntry<CustomState>;

// TODO: use regexes for matching a whole pannel of files
// produced files: /.+\.(a,out,so)/
// temporary files: /.+\.(?:o|gch)/

const ARTEFACTS_PRODUCED: &str = r".+\.(a|out|so)$";
static RE_ARTEFACTS_PRODUCED: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(ARTEFACTS_PRODUCED).unwrap());

const ARTEFACTS_TEMP: &str = r".+\.(o|gch)$";
static RE_ARTEFACTS_TEMP: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(ARTEFACTS_TEMP).unwrap());

// INFO: we need to retain .o files and all folders
fn is_to_retain(dir_entry: &CustomDirEnt) -> bool {
    let Some(file_name) = dir_entry.file_name().to_str() else {
        return false;
    };
    dir_entry.file_type().is_dir() || RE_ARTEFACTS_TEMP.is_match(file_name)
}

pub fn id_temporary_files(
    dir: std::path::PathBuf,
) -> impl Iterator<Item = Result<CustomDirEnt, jwalk::Error>> {
    let walk_dir = jwalk::WalkDirGeneric::<CustomState>::new(dir).process_read_dir(
        |_depth, _path, _read_dir_state, children| {
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
