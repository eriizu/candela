use super::*;

/// Sig. of a function that can generate the destination path
/// of a file.
type GenDestFn = fn(
    file: &std::path::Path,
    root_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Option<std::path::PathBuf>;

/// Copies files to a destination subdir with or without preserving path
/// components. When they are kept, they are inserted into the destination
/// filename, separated by dots.
///
/// example when not keeping:
/// "root/tata/src/stacks/create.c" becomes
/// "dest/tata/create.c"
///
/// example when keeping:
/// "root/tata/src/stacks/create.c" becomes
/// "dest/tata/src.stacks.create.c"
pub fn flatten<I>(
    source_dir: &std::ffi::OsStr,
    destination_dir: &std::ffi::OsStr,
    files: I,
    keep_components: bool,
) where
    I: Iterator<Item = std::ffi::OsString>,
{
    let source_dir = std::path::Path::new(source_dir);
    let destination_dir = std::path::Path::new(destination_dir);
    let gen_dest_fn: GenDestFn = if keep_components {
        path::gen_destination2
    } else {
        path::gen_destination
    };
    for file_path_str in files {
        process_one_file(file_path_str, source_dir, destination_dir, gen_dest_fn);
    }
}

fn process_one_file(
    file_path_str: std::ffi::OsString,
    root: &std::path::Path,
    output_dir: &std::path::Path,
    gen_dest: GenDestFn,
) {
    let file_path = std::path::Path::new(&file_path_str);
    let Some(dest) = gen_dest(file_path, root, output_dir) else {
        eprintln!("path generation failed for {:?}", file_path_str);
        return;
    };
    if let Err(err) = create_dir_for(&dest) {
        eprintln!("{}", err);
        return;
    };
    match std::fs::copy(file_path, &dest) {
        Ok(_) => println!("{:?} -> {:?}", file_path, dest),
        Err(error) => eprintln!("during copy {:?}", error),
    }
}

fn create_dir_for(file_path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}
