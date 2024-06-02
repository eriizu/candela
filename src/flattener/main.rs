use clap::Parser;
use delflat::*;

fn main() {
    let opts = delflat::cli::Cli::parse();

    flatten(
        &opts.root,
        &opts.dest,
        opts.file_names.into_iter(),
        opts.keep_dir_names,
    );
    // if opts.keep_dir_names {
    //     flatten(&opts.root, &opts.dest, opts.file_names.into_iter());
    // } else {
    //     flatten_discard(&opts.root, &opts.dest, opts.file_names.into_iter());
    // }
}

// fn main1() {
//     let mut args = std::env::args().skip(1);
//     let mode = args
//         .nth(0)
//         .expect("need a mode \"keep or discard\" dir name");
//     let source_dir = args.nth(0).expect("need a source dir");
//     let destination_dir = args.nth(0).expect("need a destination dir");
//     assert_eq!(
//         args.nth(0).expect("need a list of file preceeded by --"),
//         "--"
//     );
//     match mode.as_str() {
//         "keep" => flatten(&source_dir, &destination_dir, args),
//         "discard" => flatten_discard(&source_dir, &destination_dir, args),
//         _ => eprintln!("mode needs to be discard or keep"),
//     }
// }
