use std::path::Path;

pub fn path_helper<'a>(paths : &'a [&'a str]) -> std::vec::Vec<&'a Path> {
    paths.iter().map(Path::new).collect()
}
