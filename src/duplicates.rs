mod walker;
use rayon::prelude::*;
use spinoff::{spinners, Spinner};
use walker::*;

#[derive(clap::Args, Debug)]
#[command()]
pub struct Cli {
    #[arg(short, long, value_hint = clap::ValueHint::DirPath)]
    pub output: Option<std::path::PathBuf>,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub directories_to_scan: Vec<std::path::PathBuf>,
}

impl Cli {
    pub fn run(&self) {
        let mut dw = DuplicatesWalker::new(false);
        let map = dw.make_filesize_map_for_paths(
            self.directories_to_scan
                .iter()
                .map(|pathbuf| pathbuf.as_ref()),
        );
        let groups = dw.gen_matching_file_groups(&map);
        if let Some(filepath) = &self.output {
            output(&groups, filepath);
        }
        let mut to_delete = std::collections::HashSet::<std::path::PathBuf>::new();
        for group in groups.as_string_iters() {
            let path_strs: Vec<_> = group.collect();
            let len = path_strs.len();
            let ans = inquire::MultiSelect::new(
                "What files to you want to delete in this group ?",
                path_strs,
            )
            .prompt();
            match ans {
                Ok(selected) => {
                    if selected.len() != len
                        || inquire::Confirm::new(
                            "You are about to delete all file in set. Are you sure?",
                        )
                        .with_default(false)
                        .prompt()
                        .unwrap_or(false)
                    {
                        selected
                            .iter()
                            .map(|item| std::path::PathBuf::from(item))
                            .for_each(|path| {
                                to_delete.insert(path);
                            });
                    } else {
                        println!("Skipping this set.");
                    }
                }
                Err(inquire::InquireError::OperationCanceled) => {
                    break;
                }
                Err(_) => {
                    eprintln!("stopping");
                    return;
                }
            }
        }
        println!("the following files will be deleted:\n{:#?}", to_delete);
        let cont = inquire::Confirm::new("Continue?")
            .with_default(false)
            .prompt()
            .unwrap_or(false);
        if cont {
            let results: Vec<_> = to_delete
                .par_drain()
                .map(|path| {
                    let res = std::fs::remove_file(&path);
                    (path, res)
                })
                .collect();
            results.iter().for_each(|(path, res)| {
                if let Err(err) = res {
                    eprintln!("{}: {}", path.display(), err);
                } else {
                    println!("removed {}", path.display());
                }
            });
        }
    }
}

fn output(groups: &MatchingFilesGroups, filepath: &std::path::Path) {
    let mut spinner = Spinner::new(
        spinners::Dots,
        format!("Serialising to \"{}\"", filepath.display()),
        None,
    );
    if let Err(err) = groups.to_file(filepath) {
        spinner.fail(&format!(
            "Failed serialisation to \"{}\"{}",
            filepath.display(),
            err
        ));
    } else {
        spinner.success(&format!("Serialised to \"{}\"", filepath.display()));
    }
}
