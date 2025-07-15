use std::cell::RefCell;
use std::{error::Error, fmt};
use extend::ext;



thread_local! {
    static panic_on_query_error: /*LocalKey*/RefCell<bool> = RefCell::new(false);
}


// -------- Assert No Query Error -------- //

pub fn set_panic_on_query_error(on: bool) {
    panic_on_query_error.with_borrow_mut(|r| *r = on);
}

// -------- Query Error -------- //
#[derive(Debug, Clone, PartialEq)] // Display
pub struct QueryError { pub msg: String, pub line: u64 }

impl QueryError {
    pub fn new(msg: String, line: u64) -> QueryError {
        let err = QueryError { msg: dedent(&msg), line };
        panic_on_query_error.with_borrow(|&panic_| 
            if panic_ { panic!("\n{}\n", err); }
        );
        return err;
    }
}

impl Error for QueryError {}
impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "QueryError at line {}: {}", self.line, self.msg)
    }
}

fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let min_indent = lines.iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    lines.into_iter().map(|line| {
            if line.len() >= min_indent {&line[min_indent..]} else {line}
        }).collect::<Vec<&str>>().join("\n")
}

macro_rules! query_error {
    ($line:expr, $($arg:tt)*) => {{
        use crate::tools::err::QueryError;
        let msg = format!($($arg)*);
        let err = QueryError::new(msg, $line);
        Err(Box::new(err))
    }}
}

pub(crate) use query_error;

// -------- Model Error -------- //

#[derive(Debug)] pub struct ModelError { pub msg: String }

impl Error for ModelError {}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ModelError: {}", self.msg)
    }
}



// -------- ShortHands -------- //

pub type AnyFallible<T> = Result<T, Box<dyn Error>>;
pub type Fallible<T> = Result<T, Box<QueryError>>;


#[ext] pub impl<T, E: Error> Result<T, E> {
    #[inline] fn ignore_err(self) {}
    #[inline] fn assert_ok(self) { #[cfg(debug_assertions)] self.unwrap(); }
}
#[ext] pub impl<T> Option<T> {
    #[inline] fn assert_some(self) { #[cfg(debug_assertions)] self.unwrap(); }
}

