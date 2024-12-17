use rayon::prelude::*;
use spinoff::{spinners, Spinner};
use std::io::Read;

use multimap::MultiMap;

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let dir = std::path::PathBuf::try_from(dir)
        .unwrap()
        .canonicalize()
        .unwrap();

    // INFO: 1st step, walking and maping by filesize
    let mut spinner = Spinner::new(spinners::Dots, "Walking", None);
    let walk_dir = make_walkdir(dir);
    let mut file_by_sizes = map_path_by_filesize(walk_dir);
    spinner.success(&format!(
        "Done walking and mapping by filesize. {} files have a size equal to another file.",
        file_by_sizes.len()
    ));

    // INFO: 2nd step, keep groups that contain multiple files
    let sizes_with_multiple_files = get_sizes_with_multiple_files(&file_by_sizes);
    file_by_sizes.retain(|key, _| sizes_with_multiple_files.contains(key));

    // INFO: 3nd step, regroup files by the same contents
    let mut spinner = Spinner::new(spinners::Dots, "Mapping by content", None);
    let content_match_outer = get_files_with_same_content(&file_by_sizes);
    let total_files = content_match_outer.iter().flatten().count();
    spinner.success(&format!(
        "Done with {} matches ({} files total)",
        content_match_outer.len(),
        total_files
    ));
    // dbg!(content_match);
}

fn get_files_with_same_content<'a>(
    file_by_sizes: &'a MultiMap<u64, std::path::PathBuf>,
) -> Vec<Vec<&'a std::path::Path>> {
    file_by_sizes
        .iter_all()
        .par_bridge()
        .map(|(_, files)| {
            let mut content_match: Vec<Vec<&std::path::Path>> = vec![];

            let (same, mut not_same) =
                same_content_group_with_first(files.iter().map(|file| file.as_ref()));
            if same.len() > 1 {
                content_match.push(same);
                // println!("these files are the same {:?}", same);
            }
            while not_same.len() >= 2 {
                let (same, new_not_same) = same_content_group_with_first(not_same.iter().copied());
                not_same = new_not_same;
                if same.len() > 1 {
                    content_match.push(same);
                    // println!("these files are the same {:?}", same);
                }
            }
            content_match
        })
        .flatten()
        .collect()
}

/// Compare all file's contents to the first one in iterator. If they match they are returned in
/// the first vector (among with the first file) if they dont they are returned in the second
/// vector.
fn same_content_group_with_first<'a>(
    file_paths: impl Iterator<Item = &'a std::path::Path>,
) -> (Vec<&'a std::path::Path>, Vec<&'a std::path::Path>) {
    let mut buf1 = [0; 2048];
    let mut file_and_paths = file_paths
        .filter_map(|path| std::fs::File::open(path).map(|file| (path, file)).ok())
        .collect::<Vec<_>>();
    let (path_1, mut file_1) = file_and_paths.remove(0);

    let mut matching: Vec<&'a std::path::Path> = vec![path_1];
    let mut not_matching: Vec<&'a std::path::Path> = vec![];

    while !file_and_paths.is_empty() {
        let Ok(read1) = file_1.read(&mut buf1) else {
            return (
                matching,
                file_and_paths
                    .iter()
                    .map(|(path, _)| path)
                    .copied()
                    .collect(),
            );
        };
        file_and_paths.retain_mut(|(path, file)| {
            let mut buf2 = [0; 2048];
            let Ok(read2) = file.read(&mut buf2) else {
                not_matching.push(path);
                return false;
            };
            if read1 == 0 && read2 == 0 {
                matching.push(path);
                return false;
            }
            if read1 != read2 || buf1[..read1] != buf2[..read2] {
                not_matching.push(path);
                return false;
            }
            return true;
        });
    }
    (matching, not_matching)
}

/// Creates a set of all file size where multiple match exists for further checks.
fn get_sizes_with_multiple_files(
    patate: &MultiMap<u64, std::path::PathBuf>,
) -> std::collections::HashSet<u64> {
    patate
        .keys()
        .filter(|key| patate.is_vec(key))
        .copied()
        .collect::<std::collections::HashSet<_>>()
}

/// Consume walkdir and create a multimap that regroups files of the same size.
fn map_path_by_filesize(
    walk_dir: jwalk::WalkDirGeneric<(usize, u64)>,
) -> MultiMap<u64, std::path::PathBuf> {
    walk_dir
        .into_iter()
        .filter_map(|dir_ent| dir_ent.ok())
        .filter(|dir_ent| dir_ent.path().is_file())
        .map(|dir_ent| (dir_ent.client_state, dir_ent.path()))
        .collect()
}

/// Create a walkdir where dirent are associated with the file sizes.
fn make_walkdir(dir: std::path::PathBuf) -> jwalk::WalkDirGeneric<(usize, u64)> {
    jwalk::WalkDirGeneric::<(usize, u64)>::new(dir).process_read_dir(
        |_depth, _path, _rd_state, children| {
            {
                let stop_walking = children
                    .iter()
                    .filter_map(|dir_ent_res| dir_ent_res.as_ref().ok())
                    .filter(|dir_ent| std::fs::FileType::is_dir(&dir_ent.file_type()))
                    .any(|dir_ent| dir_ent.file_name() == ".git");
                if stop_walking {
                    children
                        .iter_mut()
                        .filter_map(|dir_ent_res| dir_ent_res.as_mut().ok())
                        .for_each(|dir_ent| dir_ent.read_children_path = None);
                }
            }
            children
                .iter_mut()
                .filter_map(|dir_ent_res| dir_ent_res.as_mut().ok())
                .filter(|dir_ent| std::fs::FileType::is_dir(&dir_ent.file_type()))
                .filter(|dir_ent| {
                    [
                        std::ffi::OsStr::new("node_modules"),
                        std::ffi::OsStr::new("delivery"),
                        std::ffi::OsStr::new(".git"),
                    ]
                    .contains(&dir_ent.file_name())
                })
                .for_each(|dir_ent| dir_ent.read_children_path = None);
            children.retain_mut(|candidate| {
                let Ok(candidate) = candidate else {
                    return false;
                };
                let retain = !hf::is_hidden(candidate.path()).unwrap_or(true);
                if retain {
                    if let Ok(meta) = candidate.metadata() {
                        candidate.client_state = meta.len();
                    }
                }
                retain
            });
        },
    )
}
