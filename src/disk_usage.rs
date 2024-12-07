pub fn get_disk_usage<P>(path: P) -> u64
where
    P: AsRef<std::path::Path>,
{
    jwalk::WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.path().metadata().ok())
        .map(|meta| meta.len())
        .sum()
}
