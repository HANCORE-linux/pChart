use std::path::Path;

use pchart_core::{Error, Plan};

pub fn load(path: &Path) -> Result<Plan, Error> {
    let text = std::fs::read_to_string(path).map_err(|err| Error::File(format!("{}: {err}", path.display())))?;
    Plan::parse(&text).map_err(|err| match err {
        Error::File(message) => Error::File(format!("{}: {message}", path.display())),
        other => other,
    })
}
