use multimap::MultiMap;
use rayon::prelude::*;
use spinoff::{spinners, Spinner};
use std::io::Read;

#[derive(clap::Args, Debug)]
#[command()]
pub struct Cli {
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    pub output: Option<std::path::PathBuf>,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub directories_to_scan: Vec<std::path::PathBuf>,
}

pub fn cli(cli: Cli) {
    let mut dw = DuplicatesWalker::new(false);
    let map = dw.make_filesize_map_for_paths(
        cli.directories_to_scan
            .iter()
            .map(|pathbuf| pathbuf.as_ref()),
    );
    let groups = dw.gen_matching_file_groups(&map);

    if let Some(output) = &cli.output {
        let mut spinner = Spinner::new(
            spinners::Dots,
            format!("Serialising to \"{}\"", output.display()),
            None,
        );
        if let Err(err) = groups.to_file(output) {
            spinner.fail(&format!(
                "Failed serialisation to \"{}\"{}",
                output.display(),
                err
            ));
        } else {
            spinner.success(&format!("Serialised to \"{}\"", output.display()));
        }
    }
}

pub struct DuplicatesWalker {
    quiet: bool,
    spinner: Option<Spinner>,
}

impl DuplicatesWalker {
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet,
            spinner: None,
            // file_by_sizes: multimap::multimap!(),
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
            .map(|path| {
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
            .flatten()
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
        let out = MatchingFilesGroups::from_size_groups(file_by_sizes);
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
            .open(dest)?;
        serde_json::to_writer(out_file, self)?;
        Ok(())
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
            return true;
        });
    }
    (matching, not_matching)
}

/// Create a walkdir where dirent are associated with the file sizes.
fn make_walkdir(dir: &std::path::Path) -> jwalk::WalkDirGeneric<(usize, u64)> {
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
