pub fn comps_after_root<'a>(
    path: &'a std::path::Path,
    root: &'a std::path::Path,
) -> Option<std::path::Components<'a>> {
    let mut comps_path = path.components();
    let comps_root = root.components();

    for comp in comps_root {
        if let Some(tmp) = comps_path.next() {
            if tmp != comp {
                eprintln!("not the same root");
                return None;
            }
        } else {
            eprintln!("path shorter than root");
            return None;
        }
    }
    Some(comps_path)
}

/// Get the first component that isn't part of root from a path.
/// Example:
/// - path: abc/def/ghi/jkl
/// - root: abc/def
/// - returns: ghi
pub fn first_comp_not_root(
    path: &std::path::Path,
    root: &std::path::Path,
) -> Option<std::ffi::OsString> {
    let mut comps_path = comps_after_root(path, root)?;
    comps_path.next().map(|tmp| tmp.as_os_str().to_os_string())
}

#[cfg(test)]
mod tests_first_comp_not_root {
    #[test]
    fn one_level() {
        let root = std::path::PathBuf::from("delivery");
        let path = std::path::PathBuf::from("delivery/toto/main.cpp");
        let result = super::first_comp_not_root(&path, &root);
        let expected = std::ffi::OsString::from("toto");
        assert_ne!(result, None);
        let result = result.unwrap();
        assert_eq!(result, expected);
    }
    #[test]
    fn two_levels() {
        let root = std::path::PathBuf::from("delivery/foo");
        let path = std::path::PathBuf::from("delivery/foo/toto/main.cpp");
        let result = super::first_comp_not_root(&path, &root);
        let expected = std::ffi::OsString::from("toto");
        assert_ne!(result, None);
        let result = result.unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn missmatched_names() {
        let root = std::path::PathBuf::from("delivery/foo");
        let path = std::path::PathBuf::from("not_delivery/foo/toto/main.cpp");
        let result = super::first_comp_not_root(&path, &root);
        assert_eq!(result, None);
    }

    #[test]
    fn path_shorter_than_root() {
        let root = std::path::PathBuf::from("delivery/foo");
        let path = std::path::PathBuf::from("delivery");
        let result = super::first_comp_not_root(&path, &root);
        assert_eq!(result, None);
    }
}

/// Generate destination string for given entry, without keeping components of
/// the source path.
///
/// Example:
/// "root/tata/src/stack/create.c" as an entry returns
/// "output_dir/tata/create.c"
pub fn gen_destination(
    entry: &std::path::Path,
    root: &std::path::Path,
    output_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    let first_part = first_comp_not_root(entry, root)?;
    let file_name = entry.file_name()?;
    if file_name == first_part {
        eprintln!("first part and file name are equal");
        return None;
    }
    let mut new_path = std::path::PathBuf::new();
    new_path.push(output_dir);
    new_path.push(first_part);
    new_path.push(file_name);
    Some(new_path)
}

#[cfg(test)]
mod test_gen_destination {
    use super::gen_destination;

    #[test]
    fn normal_1() {
        let root = std::path::PathBuf::from("delivery/");
        let path = std::path::PathBuf::from("delivery/martin/src/main.cpp");
        let output = std::path::PathBuf::from("flat");
        let result = gen_destination(&path, &root, &output).unwrap();
        let expected = std::ffi::OsString::from("flat/martin/main.cpp");
        assert_eq!(result, expected);
    }
}

/// Generate destination string for given entry, **keeping** components of
/// the source path and seperating them by dots.
///
/// Example:
/// "root/tata/src/stack/create.c" as an entry returns
/// "output_dir/tata/src.stack.create.c"
pub fn gen_destination2(
    entry: &std::path::Path,
    root_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    let mut comps = comps_after_root(entry, root_dir)?;
    let mut acc_ostr = std::ffi::OsString::new();
    let subdir_name = comps.next()?;
    let mut first = true;
    for component in comps {
        if !first {
            acc_ostr.push(".");
        }
        first = false;
        acc_ostr.push(component);
    }
    let mut result = std::path::PathBuf::new();
    result.push(output_dir);
    result.push(subdir_name);
    result.push(acc_ostr);
    Some(result)
}

#[cfg(test)]
mod test_gen_destination2 {
    use super::gen_destination2;

    #[test]
    fn normal_1() {
        let root = std::path::PathBuf::from("delivery/");
        let path = std::path::PathBuf::from("delivery/martin/src/main.cpp");
        let output = std::path::PathBuf::from("flat");
        let result = gen_destination2(&path, &root, &output).unwrap();
        let expected = std::ffi::OsString::from("flat/martin/src.main.cpp");
        assert_eq!(result, expected);
    }
    #[test]
    fn deep_normal() {
        let root = std::path::PathBuf::from("delivery/tata");
        let path = std::path::PathBuf::from("delivery/tata/martin/src/main.cpp");
        let output = std::path::PathBuf::from("flat");
        let result = gen_destination2(&path, &root, &output).unwrap();
        let expected = std::ffi::OsString::from("flat/martin/src.main.cpp");
        assert_eq!(result, expected);
    }

    #[test]
    fn missmatched_1st_dir() {
        let root = std::path::PathBuf::from("delivery/tata");
        let path = std::path::PathBuf::from("not_delivery/tata/martin/src/main.cpp");
        let output = std::path::PathBuf::from("flat");
        let result = gen_destination2(&path, &root, &output);
        assert_eq!(result, None);
    }

    #[test]
    fn missmatched_2nd_dir() {
        let root = std::path::PathBuf::from("delivery/tata");
        let path = std::path::PathBuf::from("delivery/not_tata/martin/src/main.cpp");
        let output = std::path::PathBuf::from("flat");
        let result = gen_destination2(&path, &root, &output);
        assert_eq!(result, None);
    }
}
