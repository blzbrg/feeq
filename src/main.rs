/// Union of std::io::Error and our custom error type.
enum CombinedError {
    Io(std::io::Error),
    Ours(feeq::Error),
}

impl std::convert::From<std::io::Error> for CombinedError {
    fn from(e : std::io::Error) -> Self {
        CombinedError::Io(e)
    }
}

impl std::convert::From<feeq::Error> for CombinedError {
    fn from(e : feeq::Error) -> Self {
        CombinedError::Ours(e)
    }
}

/// Error and a message. Used solely to communicate errors for common printing code.
pub struct TopLevelError<'a> {
    message : &'a str,
    err : CombinedError,
}

impl<'a> TopLevelError<'a> {
    fn new<E : Into<CombinedError>>(msg : &'a str, e : E) -> Self {
        TopLevelError{message : msg, err : e.into()}
    }
}

impl<'a> std::fmt::Display for TopLevelError<'a> {
    fn fmt(&self, formatter : &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.message)?;
        match &self.err {
            CombinedError::Io(e) => write!(formatter, " due to: {}", e),
            CombinedError::Ours(e) => write!(formatter, " due to: {}", e),
        }
    }
}

/// The real meat of main. This is a separate method so that it can use the question-mark early return syntax.
fn main_body() -> Result<(), TopLevelError<'static>> {
    let parsed = feeq::config_args().get_matches();
    let conf = feeq::Config::from(parsed);
    let cwd = std::env::current_dir()
        .map_err(|e| TopLevelError::new("No current working directory", e))?;
    let inputs = feeq::read_filenames(&cwd, std::io::stdin())
        .map_err(|e| TopLevelError::new("Could not read paths from stdin", e))?;
    let head = feeq::seq::find_head(&conf, inputs.iter().map(|x| x.as_ref()))
        .map_err(|e| TopLevelError::new("Could not select head", e))?;
    let plan = feeq::rename::RenamePlan::create(&conf, &head, inputs.iter().map(|x| x.as_ref()))
        .map_err(|e| TopLevelError::new("Could not create rename plan", e))?;

    if conf.show_plan {
        print!("{}", plan);
    }

    if conf.execute_plan {
        plan.execute()
            .map_err(|e| TopLevelError::new("Failure while executing rename plan", e))
    } else {
        Ok(())
    }
}

pub fn main() {
    // panic! produces distinctly less user friendly output, and returning a Result from main uses Debug instead of Display.
    match main_body() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}
