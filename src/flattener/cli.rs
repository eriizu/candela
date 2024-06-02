const LONG_ABOUT: &str = "\
Copy given files from a root directory structure to a destination directory, filing everything under one subdirectory level.

Running with flags '--root ./root --dest ./dest **/*.txt' and this dir structure:
- ./root/
  - subdir_a/
    - foo.txt
    - subdir_ab/
      - tata.txt
      - toto.txt
  - subdir_b/
    - tata.txt
    - subdir_bb/
      - toto.txt

will produce:
- ./dest/
  - subdir_a/
    - foo.txt
    - toto.txt
    - tata.txt
  - subdir_b/
    - tata.txt
    - toto.txt

In case of conflicting file names, run with -k. With the same root as above, produces:
- ./dest/
  - subdir_a/
    - foo.txt
    - subdir_ab.toto.txt
    - subdir_ab.tata.txt
  - subdir_b/
    - tata.txt
    - toto.txt";
const ABOUT: &str = "\
Copy given files from a root directory structure to a destination directory, filing everything under one subdirectory level.";
const KEEP_DIR_NAMES_HELP: &str = "\
Keep the sub directory names of the original path, separed by '.'";
const ROOT_HELP: &str = "\
Directory to use as root, only one depth of directories under root will be kept in copy";
const DEST_HELP: &str = "Destination directory for the files";

#[derive(clap::Args, Debug)]
#[command(author, version, about = ABOUT, long_about = LONG_ABOUT)]
pub struct Cli {
    //#[arg(short, long, default_value_t = true, help = KEEP_DIR_NAMES_HELP)]
    #[arg(short, long, help = KEEP_DIR_NAMES_HELP)]
    pub keep_dir_names: bool,

    #[arg(short, long, help = ROOT_HELP)]
    pub root: std::ffi::OsString,

    #[arg(short, long, help = DEST_HELP)]
    pub dest: std::ffi::OsString,

    pub file_names: Vec<std::ffi::OsString>,
}
