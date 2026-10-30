pub mod calendar;
pub mod model;
pub mod schedule;

pub use calendar::{Calendar, Holidays};
pub use model::Plan;
pub use schedule::{Schedule, Slot, Span, schedule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    File(String),
    Plan(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::File(message) | Error::Plan(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}
