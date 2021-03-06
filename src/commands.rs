use std::fs;
use std::io::Write;
use std::path::Path;

use git2::Repository;

use crate::*;

/// Compute and print the suggested version bump.
pub fn bump(repo: Repository) -> SemanticResult {
    println!("{}", core::bump(&repo));
    Ok(())
}

/// Generate a changelog.
pub fn changelog(repo: Repository) -> SemanticResult {
    let mut fp = fs::File::create(repo.path().parent().unwrap().join("CHANGELOG.md")).unwrap();
    let changelog = core::changelog(&repo);
    changelog
        .iter()
        .for_each(|m| writeln!(fp, "{}", m).unwrap());
    Ok(())
}

/// Print release notes.
pub fn notes(repo: Repository) -> SemanticResult {
    let notes = core::notes(&repo);
    println!("{}", notes.join("\n"));
    Ok(())
}

/// Create an entire release.
pub fn release(repo: Repository) -> SemanticResult {
    let current = languages::get(&repo).unwrap();
    let proposed = core::version(&repo);
    if current == proposed {
        println!("No release needed. Staying at {}", current.to_tag());
    } else {
        languages::put(&repo, proposed.clone()).unwrap();
        languages::add(&repo).unwrap();
        let oid = utils::commit(
            &repo,
            &format!("build: version bump to {} [skip ci]", &proposed.to_tag()),
        );
        utils::tag(&repo, proposed);

        // TODO: this fails in Github Actions yet appears to work
        let path = repo.path().parent().unwrap().join("CHANGELOG.md");
        let mut fp = fs::File::create(path).unwrap();
        core::changelog(&repo)
            .iter()
            .for_each(|m| writeln!(fp, "{}", m).unwrap());

        utils::add(&repo, Path::new("CHANGELOG.md"));
        utils::amend(&repo, oid);
    }
    Ok(())
}

/// Update the project version.
pub fn version(repo: Repository) -> SemanticResult {
    let current = languages::get(&repo).unwrap();
    let proposed = core::version(&repo);
    if current != proposed {
        languages::put(&repo, proposed.clone()).unwrap();
    }
    println!("{}", proposed);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_bump() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"",
            "Not conventional",
        );
        assert!(bump(repo).is_ok());
    }

    #[test]
    fn test_changelog() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");
        update(&repo, relpath, "ocular patdown", "not conventional");
        assert!(changelog(repo).is_ok());

        let result = fs::read_to_string(dir.path().join("CHANGELOG.md")).unwrap();
        assert!(result.starts_with("\n## wip\n - not conventional "))
    }

    #[test]
    fn test_notes() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"",
            "build: conventional",
        );
        assert!(notes(repo).is_ok());
    }

    #[test]
    fn test_version() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"",
            "feat: special",
        );
        assert!(version(repo).is_ok());
    }

    #[test]
    fn test_release_rust() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");
        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"",
            "Initial Commit",
        );
        let current = languages::get(&repo).unwrap();
        release(repo).unwrap();
        let repo = git2::Repository::open(dir.path()).unwrap();
        let new = languages::get(&repo).unwrap();
        assert_eq!(current, new);

        update(
            &repo,
            Path::new("README.md"),
            "new thing",
            "feat: cool thing",
        );
        let current = languages::get(&repo).unwrap();
        release(repo).unwrap();
        let repo = git2::Repository::open(dir.path()).unwrap();
        let new = languages::get(&repo).unwrap();
        assert_ne!(current, new);
        let changelog = fs::read_to_string(
            repo.path()
                .parent()
                .unwrap()
                .join(Path::new("CHANGELOG.md")),
        )
        .unwrap();
        println!("{}", changelog);
        assert!(changelog.starts_with("\n## v0.2.0"));
    }
}
