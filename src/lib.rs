mod commands;
mod core;
pub mod languages;
pub mod utils;

#[cfg(test)]
mod test_utils;

use std::fmt;

pub use git2::Repository;
use semver::Version;

pub use commands::*;

#[derive(Debug)]
pub enum SemanticError {
    BumpError,
    VersionError,
    NotesError,
    ChangelogError,
    ReleaseError,
    IOError,
}

#[derive(PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Bump {
    None,
    Prerelease,
    // Build
    Patch,
    Minor,
    Major,
}

impl fmt::Display for Bump {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub type SemanticResult = Result<(), SemanticError>;

trait ToTag {
    fn to_tag(&self) -> String;
}

impl ToTag for Version {
    fn to_tag(&self) -> String {
        format!("v{}", self)
    }
}
