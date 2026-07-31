/// Error returned when an operation tries to divide by zero.
#[derive(Clone, Debug)]
pub struct ZeroDivisionError;

impl std::fmt::Display for ZeroDivisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "division by zero")
    }
}

impl std::error::Error for ZeroDivisionError {}
