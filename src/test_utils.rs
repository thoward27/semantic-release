use std::fs;

use crate::utils;

pub use git2::*;
pub use semver::Version;
pub use std::path::Path;
pub use tempfile::tempdir;

pub fn update(repo: &git2::Repository, relpath: &Path, content: &str, message: &str) {
    fs::write(repo.path().parent().unwrap().join(relpath), content).unwrap();
    utils::add(&repo, relpath);
    utils::commit(&repo, &message);
}
