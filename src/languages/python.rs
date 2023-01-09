use std::fs;
use std::io::Result;
use std::io::Write;
use std::path::PathBuf;

use git2::Repository;
use toml_edit::value;
use toml_edit::Document;

use crate::*;

fn path(repo: &Repository) -> PathBuf {
    repo.path().parent().unwrap().join("pyproject.toml")
}

fn load(repo: &Repository) -> Result<Document> {
    let config: String = fs::read_to_string(path(repo))?;
    Ok(config
        .parse::<Document>()
        .expect("Could not parse pyproject.toml"))
}

fn save(repo: &Repository, config: Document) {
    let mut file = fs::File::create(path(repo)).unwrap();
    file.write_all(config.to_string_in_original_order().as_bytes())
        .unwrap();
}

pub fn get(repo: &Repository) -> Option<Version> {
    let config = load(repo).ok()?;
    Version::parse(config["tool"]["poetry"]["version"].as_str()?).ok()
}

pub fn put(repo: &Repository, version: Version) {
    let mut config = load(repo).unwrap();
    config["tool"]["poetry"]["version"] = value(version.to_string());
    save(repo, config);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_get() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let abspath = dir.path().join("pyproject.toml");
        fs::write(&abspath, "[tool.poetry]\nversion=\"0.1.0\"").unwrap();
        assert_eq!(get(&repo), Some(Version::new(0, 1, 0)));
    }

    #[test]
    fn get_put() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("pyproject.toml");
        update(
            &repo,
            relpath,
            "[tool.poetry]\nversion=\"0.1.0\"",
            "Initial Commit",
        );
        put(&repo, Version::new(1, 0, 0));
        assert_eq!(get(&repo), Some(Version::new(1, 0, 0)));
    }
}
