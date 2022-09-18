use std::path::{PathBuf, Path};

#[derive(PartialEq, Eq, Debug)]
pub enum HeadSource {
    InputFile(PathBuf),
    InferredFromOtherMember(PathBuf),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Head {
    pub base : String,
    pub source : HeadSource,
}

impl Head {
    pub fn from_file(base : String, source_file : PathBuf) -> Self {
        Self {base: base, source: HeadSource::InputFile(source_file)}
    }

    pub fn from_infer(base : String, infer_file : PathBuf) -> Self {
        Self {base: base, source: HeadSource::InferredFromOtherMember(infer_file)}
    }
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

/// Infer whether a basename is already a member of a sequence.
pub fn infer_membership<'a, 'b>(conf : &'a crate::Config, basename : &'b str) -> Option<&'b str> {
    match basename.split_once(conf.separator.as_str()) {
        Some((head, rest)) => {
            if !(head.is_empty() || rest.is_empty()) {
                Some(head)
            } else {
                None
            }
        },
        None => None
    }
}

/// Return the head to be used for a collection of paths.
///
/// Sort the paths by the "basename" (filename up to the first dot) and then choose the first one to
/// be the head.
pub fn find_head<'a, I : Iterator::<Item=&'a Path>>(conf : &crate::Config, paths : I)
                                                    -> Result<Head, super::Error> {
    // Because we are going to sort everything by basename, we are going to call `basename` on
    // everything no matter what. Thus, might as well store it while we are at it.
    let mut base_to_file = std::collections::BTreeMap::<String, &Path>::new();
    // Map from the path to the inferred head-name (ie. a sequence head that was added
    // previously).
    let mut inferred_others = std::collections::BTreeMap::<String, &Path>::new();
    for path in paths {
        match basename(path) {
            Ok(s) => { //base_to_file.insert(s, path),
                match infer_membership(conf, &s) {
                    Some(existing_headname) => {
                        inferred_others.insert(existing_headname.to_owned(), path);
                    },
                    None => {base_to_file.insert(s, path);},
                }
            },
            Err(e) => return Err(e), // TODO: can we use try_fold to avoid this early return?
        };
    }

    if inferred_others.len() == 1 {
        // If there is exactly one inferred other, we should use it as our head.
        let (inferred_headname, inference_source) = inferred_others.into_iter().next()
            .expect("inferred_others len one but can't get it???");
        Ok(Head::from_infer(inferred_headname, inference_source.to_owned()))
    } else if inferred_others.len() > 1 {
        Err(crate::Error::MultipleOtherHeads)
    } else {
        match base_to_file.into_iter().next() {
            Some((basename, path)) => Ok(Head::from_file(basename, path.to_owned())),
            None => Err(super::Error::NoInputFiles),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::test_lib::path_helper;
    use super::{Head, find_head, infer_membership};

    #[test]
    fn infer_membership_test() {
        let conf = crate::Config::default();
        assert_eq!(None, infer_membership(&conf, "abc"));
        assert_eq!(Some("a"), infer_membership(&conf, "a_bc"));
        assert_eq!(None, infer_membership(&conf, "_bc"));
        assert_eq!(None, infer_membership(&conf, "a_"));
    }

    #[test]
    fn find_head_from_path_test() {
        let conf = crate::Config::default();
        assert_eq!(Err(crate::Error::NoInputFiles),
                   find_head(&conf, path_helper(&[]).into_iter()),
                   "No inputs");
        assert_eq!(Ok(Head::from_file("a".to_owned(), PathBuf::from("/foo/a"))),
                   find_head(&conf, path_helper(&["/foo/a"]).into_iter()),
                   "Single path");
        assert_eq!(Ok(Head::from_file("a".to_owned(), PathBuf::from("/foo/a"))),
                   find_head(&conf, path_helper(&["/foo/a", "/foo/b"]).into_iter()),
                   "Two paths in order");
        assert_eq!(Ok(Head::from_file("a".to_owned(), PathBuf::from("/foo/a"))),
                   find_head(&conf, path_helper(&["/foo/b", "/foo/a"]).into_iter()),
                   "Two paths out of order");
        assert_eq!(Ok(Head::from_file("a".to_owned(), PathBuf::from("a"))),
                   find_head(&conf, path_helper(&["b", "a"]).into_iter()),
                   "Bare names");
        assert_eq!(Ok(Head::from_file("a".to_owned(), PathBuf::from("/foo/a"))),
                   find_head(&conf, path_helper(&["b", "/foo/a"]).into_iter()),
                   "Mixed path and name");
    }

    #[test]
    fn find_head_from_infer_test() {
        let conf = crate::Config::default();
        assert_eq!(Ok(Head::from_infer("a".to_owned(), PathBuf::from("/foo/a_b"))),
                   find_head(&conf, path_helper(&["/foo/a_b", "/foo/c"]).into_iter()));

        // Decoy sequence heads
        assert_eq!(Ok(Head::from_file("a".to_owned(), PathBuf::from("/foo/a"))),
                   find_head(&conf, path_helper(&["/foo/a", "/foo/b_"]).into_iter()));
        assert_eq!(Ok(Head::from_file("_a".to_owned(), PathBuf::from("/foo/_a"))),
                   find_head(&conf, path_helper(&["/foo/_a", "/foo/a"]).into_iter()));

        // Two sequences mixed
        assert_eq!(Err(crate::Error::MultipleOtherHeads),
                   find_head(&conf, path_helper(&["/foo/a_actual", "/foo/c_actual"]).into_iter()));

    }
}
