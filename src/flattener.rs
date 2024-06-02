mod cli;
mod flatten;
mod path;

pub use cli::Cli;

pub fn run(opts: Cli) {
    flatten::flatten(
        &opts.root,
        &opts.dest,
        opts.file_names.into_iter(),
        opts.keep_dir_names,
    );
}
