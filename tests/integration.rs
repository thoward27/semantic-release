use semantic_release::*;
use std::{fs, path::Path};
use tempfile::{tempdir, TempDir};

// TODO: Consolidate the two definitions of update
pub fn update(repo: &git2::Repository, relpath: &Path, content: &str, message: &str) {
    fs::write(repo.path().parent().unwrap().join(relpath), content).unwrap();
    utils::add(&repo, relpath);
    utils::commit(&repo, &message);
}

pub fn new_repo(dir: &TempDir) -> Repository {
    git2::Repository::init(dir.path()).unwrap()
}

pub fn assert_changelog_contains(dir: &TempDir, content: &str) -> String {
    assert!(changelog(new_repo(&dir)).is_ok());
    let result = fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
    assert!(result.contains(content));
    result
}

#[test]
fn test_rust() {
    let dir = tempdir().unwrap();
    let repo = new_repo(&dir);

    // For this integration test we will explore these three files:
    let toml_path = Path::new("Cargo.toml");
    let readme_path = Path::new("README.md");

    // First create the initial commit, with a version number.
    update(
        &repo,
        toml_path,
        "[package]\nversion=\"0.1.0\"",
        "Initial commit",
    );

    // Check the resulting changelog.
    let result = assert_changelog_contains(&dir, "Initial commit");
    assert!(result.starts_with("\n## wip"));

    // At this point, version should do nothing since we have no conventional commits.
    assert!(version(new_repo(&dir)).is_ok());
    assert_eq!(
        languages::get(&repo).unwrap(),
        semver::Version::new(0, 1, 0)
    );

    // So, let's add a conventional commit.
    update(&repo, readme_path, "# hello world", "fix: readme");
    assert!(version(new_repo(&dir)).is_ok());
    assert_eq!(
        languages::get(&repo).unwrap(),
        semver::Version::new(0, 1, 1)
    );

    // What about the Changelog?
    assert_changelog_contains(&dir, "fix: readme");

    // Tag the current version.
    utils::tag(&repo, languages::get(&repo).unwrap());

    // Now that there is a tagged commit, version should start with v0.1.1
    let result = assert_changelog_contains(&dir, "fix: readme");
    assert!(result.starts_with("\n## v0.1.1"));

    update(&repo, readme_path, "# Goodbye world", "feat: new readme");
    assert!(release(new_repo(&dir)).is_ok());
}
