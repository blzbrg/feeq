use std::path::{Path, PathBuf};

fn new_name(path : &Path, head_name: &str) -> Result<PathBuf, crate::Error> {
    let file_name : Option<&str> = path.file_name().and_then(std::ffi::OsStr::to_str);

    match (file_name, path.parent()) {
        (Some(s), Some(parent)) => {let new_name = String::from(head_name) + "_" + s;
                                    Ok(parent.join(new_name))},
        _ => Err(crate::Error::UnusableFilename(path.to_owned())),
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct RenamePlan (std::vec::Vec<(PathBuf, PathBuf)>); // newtype

impl RenamePlan {
    pub fn create<'a, I : Iterator::<Item=&'a Path>>(head : &crate::seq::Head, files : I) -> Result<RenamePlan, crate::Error> {
        let mut acc = vec![];
        for path in files {
            match new_name(path, &head.base) {
                Ok(new_path) => acc.push((path.to_owned(), new_path)),
                Err(e) => return Err(e), // TODO: cleaner error handling
            };
        }
        Ok(RenamePlan(acc))
    }

    pub fn execute(&self) -> Result<(), std::io::Error> {
        let RenamePlan(plan_vec) = self;
        plan_vec
            .iter()
            .try_fold((),
                      |_, (old_path, new_path)| std::fs::rename(old_path, new_path))
    }
}

impl std::fmt::Display for RenamePlan {
    fn fmt(&self, formatter : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let RenamePlan(old_new_pairs,) = self;
        old_new_pairs.iter().try_fold((), |_, (old, new)| {
            write!(formatter, "Rename {} to {}\r\n", old.display(), new.display())
        })
    }
}

#[cfg(test)]
mod test {
    use super::RenamePlan;
    use crate::test_lib::path_helper;
    use crate::seq::Head;
    use std::path::PathBuf;

    fn pb_tuple(a : &str, b : &str) -> (PathBuf, PathBuf) {
        (PathBuf::from(a), PathBuf::from(b))
    }

    #[test]
    fn rename_plan_create_test() {
        // Only absolute paths
        assert_eq!(Ok(RenamePlan(vec![pb_tuple("/foo/a.txt", "/foo/a_a.txt"),
                                      pb_tuple("/foo/b.txt", "/foo/a_b.txt"),])),
                   RenamePlan::create(&Head{base : String::from("a"), head_file : PathBuf::from("/foo/a.txt")},
                                      path_helper(&["/foo/a.txt", "/foo/b.txt"]).into_iter()));
        // Only relative paths
        assert_eq!(Ok(RenamePlan(vec![pb_tuple("a.txt", "a_a.txt"),
                                      pb_tuple("b.txt", "a_b.txt"),])),
                   RenamePlan::create(&Head{base : String::from("a"), head_file : PathBuf::from("a.txt")},
                                      path_helper(&["a.txt", "b.txt"]).into_iter()));
        // Both relative and absolute paths
        assert_eq!(Ok(RenamePlan(vec![pb_tuple("a.txt", "a_a.txt"),
                                      pb_tuple("/foo/b.txt", "/foo/a_b.txt"),])),
                   RenamePlan::create(&Head{base : String::from("a"), head_file : PathBuf::from("a.txt")},
                                      path_helper(&["a.txt", "/foo/b.txt"]).into_iter()));
        // TODO: test error cases?
    }

    #[test]
    fn rename_plan_display_test() {
        assert_eq!("Rename /foo/a to /foo/a_a\r\n",
                   format!("{}", RenamePlan(vec![pb_tuple("/foo/a", "/foo/a_a")])));
        assert_eq!("Rename /foo/a to /foo/a_a\r\nRename /foo/b to /foo/a_b\r\n",
                   format!("{}", RenamePlan(vec![pb_tuple("/foo/a", "/foo/a_a"),
                                                 pb_tuple("/foo/b", "/foo/a_b")])));

    }
}
