pub mod seq;
pub mod rename;

#[cfg(test)]
pub mod test_lib;

use std::io::BufRead;

// Debug needed for assert macros
#[derive(PartialEq, Eq, Debug)]
pub enum Error {
    UnusableFilename(std::path::PathBuf),
    NoInputFiles,
}

// This is needed for displaying the error in main
impl std::fmt::Display for Error {
    fn fmt(&self, formatter : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::UnusableFilename(path) => write!(formatter, "Filename from {} could not be used",
                                                    path.display()),
            Error::NoInputFiles => write!(formatter, "No input files were provided"),
        }
    }
}

pub struct Config {
    pub show_plan : bool,
    pub execute_plan : bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            show_plan : true,
            execute_plan : true,
        }
    }
}

pub fn config_args() -> clap::Command<'static> {
    clap::Command::new("feeq")
        .arg(clap::Arg::new("show-plan")
             .long("show-plan")
             .value_parser(clap::builder::BoolValueParser::new())
             .default_value("true")
             .value_name("BOOLEAN")
             .help("Output the rename plan before performing the renames."))
        .arg(clap::Arg::new("execute-plan")
             .long("execute-plan")
             .value_parser(clap::builder::BoolValueParser::new())
             .default_value("true")
             .value_name("BOOLEAN")
             .help("Execute the rename plan. When false, plan is constructed and optionally printed\
                   according to other args, but never run."))

}

impl From<clap::parser::ArgMatches> for Config {
    fn from(matches : clap::parser::ArgMatches) -> Self {
        Config {
            show_plan : *matches.get_one::<bool>("show-plan")
                .expect("show-plan should have clap default"),
            execute_plan : *matches.get_one::<bool>("execute-plan")
                .expect("execute-plan should have clap default"),

        }
    }
}

pub fn read_filenames<E : std::io::Read>(
    base_dir : &std::path::Path, readable : E)
    -> Result<std::vec::Vec<std::path::PathBuf>, std::io::Error> { // TODO: avoid vec?
    let buf_read = std::io::BufReader::new(readable);
    let mut ret = std::vec::Vec::<std::path::PathBuf>::new();
    for line_or_err in buf_read.lines() {
        match line_or_err {
            Ok(line) => {let p = std::path::Path::new(line.as_str());
                         // Confusion caution: can mix relative and absolute paths. This may be
                         // tightened later to simplify other requirements/checks.
                         let pb = if p.is_relative() {
                             base_dir.join(p)
                         } else {
                             p.into()
                         };
                         ret.push(pb);},
            Err(e) => {return Err(e)}, // TODO: cleaner error handling
        };
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_filenames_test() {
        let cursor = std::io::Cursor::new(b"foo\n/bar/foo\n");
        let res = super::read_filenames(std::path::Path::new("/bar"), cursor);
        assert!(res.is_ok());

        let barfoo = || {std::path::PathBuf::from("/bar/foo")}; // thunk
        assert_eq!(vec![barfoo(), barfoo()], res.expect("Err filenames even after assert?"));

        // TODO: test error case? Not sure how to construct that artificially.
    }

    #[test]
    fn read_filenames_empty_test() {
        let cursor = std::io::Cursor::new([]);
        let res = super::read_filenames(std::path::Path::new("/bar"), cursor);
        assert_eq!(Some(vec![]), res.ok());
    }
}
