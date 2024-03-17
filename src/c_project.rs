type CustomState = (usize, bool);
type CustomDirEnt = jwalk::DirEntry<CustomState>;

// TODO: use regexes for matching a whole pannel of files
// produced files /.+\.(a,out,so)/
// temporary files /.+\.(?:o|gch)

// INFO: we need to retain .o files and all folders
fn is_to_retain(dir_entry: &CustomDirEnt) -> bool {
    dir_entry.file_type().is_dir()
        || dir_entry
            .file_name
            .to_str()
            .map(|s| s.ends_with(".o"))
            .unwrap_or(false)
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
