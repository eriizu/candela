use multimap::MultiMap;
use rayon::prelude::*;
use spinoff::{spinners, Spinner};
use std::io::Read;

pub struct DuplicatesWalker {
    quiet: bool,
    spinner: Option<Spinner>,
}

impl DuplicatesWalker {
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet,
            spinner: None,
        }
    }

    pub fn make_filesize_map_for_paths<'a>(
        &mut self,
        paths: impl Iterator<Item = &'a std::path::Path>,
    ) -> MultiMap<u64, std::path::PathBuf> {
        if !self.quiet {
            self.spinner = Some(Spinner::new(spinners::Dots, "Walking", None));
        }
        let file_by_sizes: MultiMap<u64, std::path::PathBuf> = paths
            .flat_map(|path| {
                if let Some(spinner) = &mut self.spinner {
                    let msg = format!("Walking {}", path.display());
                    spinner.update_text(msg);
                }
                make_walkdir(path)
                    .into_iter()
                    .filter_map(|dir_ent_res| dir_ent_res.ok())
                    .filter(|dir_ent| dir_ent.path().is_file())
                    .map(|dir_ent| (dir_ent.client_state, dir_ent.path()))
                    .collect::<Vec<_>>()
            })
            .collect();
        if let Some(spinner) = &mut self.spinner {
            spinner.success(&format!(
                "Done walking and mapping by filesize. {} files have a size equal to another file.",
                file_by_sizes.len()
            ));
        }
        file_by_sizes
    }

    pub fn gen_matching_file_groups(
        &mut self,
        file_by_sizes: &MultiMap<u64, std::path::PathBuf>,
    ) -> MatchingFilesGroups {
        if !self.quiet {
            self.spinner = Some(Spinner::new(
                spinners::Dots,
                "Scanning content, making groups...",
                None,
            ));
        }
        let mut out = MatchingFilesGroups::from_size_groups(file_by_sizes);
        out.sort();
        if let Some(spinner) = &mut self.spinner {
            spinner.success(&format!(
                "Done with {} matches ({} files total)",
                out.len(),
                out.total_files()
            ));
        }
        out
    }
}

#[derive(serde::Serialize)]
pub struct MatchingFilesGroups {
    groups: Vec<Vec<std::path::PathBuf>>,
}

impl MatchingFilesGroups {
    pub fn from_size_groups(file_by_sizes: &MultiMap<u64, std::path::PathBuf>) -> Self {
        let out: Vec<Vec<std::path::PathBuf>> = file_by_sizes
            .iter_all()
            .par_bridge()
            .map(|(_, files)| {
                let mut same_content_groups: Vec<Vec<std::path::PathBuf>> = vec![];
                let mut not_checked: Vec<&std::path::Path> =
                    files.iter().map(|path| path.as_ref()).collect();

                loop {
                    let (mut same, not_same) = group_same_content(not_checked.iter().copied());
                    not_checked = not_same;
                    if same.len() > 1 {
                        same_content_groups
                            .push(same.drain(..).map(|item| item.to_owned()).collect());
                    }
                    if not_checked.len() < 2 {
                        break;
                    }
                }
                same_content_groups
            })
            .flatten()
            .collect();
        Self { groups: out }
    }

    pub fn sort(&mut self) {
        self.groups.sort_by(|a, b| a[0].cmp(&b[0]));
    }

    pub fn total_files(&self) -> usize {
        self.groups.iter().flatten().count()
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn to_file(&self, dest: &std::path::Path) -> std::io::Result<()> {
        let out_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dest)?;
        serde_json::to_writer(out_file, self)?;
        Ok(())
    }

    pub fn as_string_iters(&self) -> impl Iterator<Item = impl Iterator<Item = &'_ str>> {
        self.groups
            .iter()
            .map(|group| group.iter().filter_map(|path| path.to_str()))
    }
}

/// Compare all file's contents to the first one in iterator. If they match they are returned in
/// the first vector (among with the first file) if they dont they are returned in the second
/// vector.
fn group_same_content<'a>(
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
            true
        });
    }
    (matching, not_matching)
}

/// Create a walkdir where dirent are associated with the file sizes.
fn make_walkdir(dir: &std::path::Path) -> jwalk::WalkDirGeneric<(usize, u64)> {
    jwalk::WalkDirGeneric::<(usize, u64)>::new(dir).process_read_dir(
        |_depth, _path, _rd_state, children| {
            stop_walking_in_git_repo(children);
            do_not_enter_some_directories(children);
            retain_not_hidden_and_add_size_on_state(children);
        },
    )
}

/// Removed hidden files from yielded files and add file len to their client_state for further
/// processing.
fn retain_not_hidden_and_add_size_on_state(
    children: &mut Vec<Result<jwalk::DirEntry<(usize, u64)>, jwalk::Error>>,
) {
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
}

/// Tell walkdir not to go in the hardcoded list of directories.
fn do_not_enter_some_directories(
    children: &mut [Result<jwalk::DirEntry<(usize, u64)>, jwalk::Error>],
) {
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
}

/// This sets all read_path to None so that walkir doesn't go any deeper in this directory.
fn stop_walking_in_git_repo(children: &mut [Result<jwalk::DirEntry<(usize, u64)>, jwalk::Error>]) {
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
