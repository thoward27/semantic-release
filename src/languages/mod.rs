pub mod python;
pub mod rust;

use git2::Repository;
use std::path::Path;

use crate::*;

pub fn get(repo: &Repository) -> Option<Version> {
    if let Some(version) = python::get(&repo) {
        return Some(version);
    }
    if let Some(version) = rust::get(&repo) {
        return Some(version);
    }
    None
}

pub fn put(repo: &Repository, version: Version) -> SemanticResult {
    if let Some(_current) = python::get(&repo) {
        python::put(&repo, version);
        Ok(())
    } else if let Some(_current) = rust::get(&repo) {
        rust::put(&repo, version);
        Ok(())
    } else {
        Err(SemanticError::IOError)
    }
}

/// Add the version-file.
pub fn add(repo: &Repository) -> SemanticResult {
    if let Some(_version) = python::get(&repo) {
        utils::add(&repo, Path::new("pyproject.toml"));
        Ok(())
    } else if let Some(_version) = rust::get(&repo) {
        utils::add(&repo, Path::new("Cargo.toml"));
        Ok(())
    } else {
        Err(SemanticError::IOError)
    }
}
