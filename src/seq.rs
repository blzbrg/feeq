use std::path::{PathBuf, Path};

#[derive(PartialEq, Eq, Debug)]
pub struct Head {
    pub base : String,
    pub head_file : PathBuf,
}

/// Return the portion of the filename before the first dot, or the entire filename if there is no dot.
fn basename(path : &Path) -> Result<String, super::Error> {
    let filename : Option<&str> = path.file_name().and_then(std::ffi::OsStr::to_str);
    match filename {
        None => Err(super::Error::UnusableFilename(path.to_owned())),
        Some(s) => {
            match s.split_once('.') {
                Some((base, _)) => Ok(base.to_owned()),
                None => Ok(s.to_owned()),
            }
        },
    }
}

/// Return the head to be used for a collection of paths.
///
/// Sort the paths by the "basename" (filename up to the first dot) and then choose the first one to
/// be the head.
pub fn find_head<'a, I : Iterator::<Item=&'a Path>>(paths : I) -> Result<Head, super::Error> {
    // Because we are going to sort everything by basename, we are going to call `basename` on
    // everything no matter what. Thus, might as well store it while we are at it.
    let mut base_to_file = std::collections::BTreeMap::<String, &Path>::new();
    for path in paths {
        match basename(path) {
            Ok(s) => base_to_file.insert(s, path),
            Err(e) => return Err(e), // TODO: can we use try_fold to avoid this early return?
        };
    }

    match base_to_file.into_iter().next() {
        Some((basename, path)) => Ok(Head{base: basename, head_file:path.to_owned()}),
        None => Err(super::Error::NoInputFiles),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};
    use super::{Head, find_head};

    fn path_helper<'a>(paths : &'a [&'a str]) -> std::vec::Vec<&'a Path> {
        paths.iter().map(Path::new).collect()
    }

    #[test]
    fn find_head_test() {
        assert_eq!(Err(crate::Error::NoInputFiles),
                   find_head(path_helper(&[]).into_iter()),
                   "No inputs");
        assert_eq!(Ok(Head{base: "a".to_owned(), head_file: PathBuf::from("/foo/a")}),
                   find_head(path_helper(&["/foo/a"]).into_iter()),
                   "Single path");
        assert_eq!(Ok(Head{base: "a".to_owned(), head_file: PathBuf::from("/foo/a")}),
                   find_head(path_helper(&["/foo/a", "/foo/b"]).into_iter()),
                   "Two paths in order");
        assert_eq!(Ok(Head{base: "a".to_owned(), head_file: PathBuf::from("/foo/a")}),
                   find_head(path_helper(&["/foo/b", "/foo/a"]).into_iter()),
                   "Two paths out of order");
        assert_eq!(Ok(Head{base: "a".to_owned(), head_file: PathBuf::from("a")}),
                   find_head(path_helper(&["b", "a"]).into_iter()),
                   "Bare names");
        assert_eq!(Ok(Head{base: "a".to_owned(), head_file: PathBuf::from("/foo/a")}),
                   find_head(path_helper(&["b", "/foo/a"]).into_iter()),
                   "Mixed path and name");
    }
}
