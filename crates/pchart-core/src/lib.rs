pub mod analysis;
pub mod calendar;
pub mod edit;
pub mod input;
pub mod model;
pub mod schedule;
pub mod text;

pub use analysis::{Analysis, Check, Handoff, Next, Severity, State, Verdict, analyse, why};
pub use calendar::{Calendar, Holidays};
pub use edit::{Edit, Fix, apply_all};
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
