mod recursive_cleaner;

fn main() {
    recursive_cleaner::RecursiveCleaner::new().run(std::env::args().skip(1));
}
