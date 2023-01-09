#![allow(clippy::trivial_regex)]

use crate::*;
use std::path::Path;

use git2::{Commit, Oid, Repository, Revwalk, Signature, Sort};
use regex::RegexSet;
use semver::Version;

/// Returns all of the versions of the application in order with versions[0] being the first tagged version.
pub fn versions(repo: &Repository) -> Vec<Version> {
    let mut tags: Vec<Version> = repo
        .tag_names(Some("v*.*.*"))
        .unwrap()
        .iter()
        .map(|t| Version::parse(&t.unwrap().to_string()[1..]).unwrap())
        .collect();
    tags.sort();
    tags
}

/// Creates and returns a repo walker.
pub fn walker(repo: &Repository, start: Option<Version>, stop: Option<Version>) -> Revwalk {
    log::debug!("walker({:?}..{:?})", start, stop);
    let mut walker = repo.revwalk().expect("could not create walker");
    if start.is_none() && stop.is_none() {
        walker.push_head().expect("could not push HEAD to walker");
    } else if start.is_some() {
        walker
            .push_range(&format!(
                "{}..{}",
                start.unwrap().to_tag(),
                match stop {
                    Some(version) => version.to_tag(),
                    None => "".to_string(),
                }
            ))
            .unwrap();
    } else {
        walker
            .push(
                repo.revparse_single(&stop.unwrap().to_tag())
                    .unwrap()
                    .peel_to_commit()
                    .unwrap()
                    .id(),
            )
            .unwrap();
    }
    walker.set_sorting(Sort::REVERSE).unwrap();
    walker
}

/// Returns a vector of walkers, the entire history of the project.
pub fn walkers(repo: &Repository) -> Vec<Revwalk> {
    // collect all of the versions.
    let mut tags: Vec<Version> = versions(repo);
    tags.reverse();

    let mut start: Option<Version> = None;
    let mut stop: Option<Version> = tags.pop();
    let mut walkers: Vec<Revwalk> = vec![];
    loop {
        walkers.push(walker(repo, start, stop.clone()));
        // If there's nothing left to get, push up to HEAD.
        if tags.is_empty() {
            if stop.is_some() && !is_head_tagged(repo) {
                walkers.push(walker(repo, stop, None));
            }
            break;
        } else {
            start = stop;
            stop = tags.pop();
        }
    }
    walkers
}

/// Determines if the HEAD is tagged.
pub fn is_head_tagged(repo: &Repository) -> bool {
    if let Some(version) = versions(repo).pop() {
        let latest_tag = version.to_tag();
        let tagged = repo
            .revparse_single(&latest_tag)
            .unwrap()
            .peel_to_commit()
            .unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        tagged.id() == head.id()
    } else {
        false
    }
}

pub fn commit_message(commit: Commit) -> String {
    format!(
        " - {} - {} ({})",
        commit.summary().unwrap(),
        commit.author().email().unwrap(),
        commit.id()
    )
}

pub fn commit_bump(commit: &Commit) -> Bump {
    log::debug!("commit message: {}", commit.summary().unwrap());
    // check if breaking change is in the message
    // check what it starts with: regex ^(type)(\(scope\)(! breaking change)?: )
    let set = RegexSet::new([
        r"(test|build|chore|ci|docs|perf|refactor|revert|style)(\([\w ]+\))?: \w+.*",
        r"fix(\([\w ]+\))?: \w+.*",
        r"feat(\([\w ]+\))?: \w+.*",
        r"BREAKING CHANGE",
    ])
    .unwrap();
    match set.matches(commit.message().unwrap()).into_iter().max() {
        Some(m) => match m {
            0 => Bump::Prerelease,
            1 => Bump::Patch,
            2 => Bump::Minor,
            3 => Bump::Major,
            _ => Bump::None,
        },
        _ => Bump::None,
    }
}

/// Add files to the staging area.
pub fn add(repo: &Repository, path: &Path) {
    log::debug!("adding: {:?}", path);
    let path = path
        .strip_prefix(repo.path().parent().unwrap())
        .unwrap_or(path);
    let mut index = repo.index().expect("Could not get index");
    index.add_path(path).expect("Could not add path");
    index.write().unwrap();
}

/// Commit files with the message given.
pub fn commit(repo: &Repository, message: &str) -> Oid {
    let oid = repo
        .index()
        .expect("Could not get index")
        .write_tree()
        .unwrap();
    let signature = Signature::now("Semantic Release", "info@tomhoward.codes")
        .expect("could not make signature");
    let parent = match repo.head().ok() {
        Some(head) => {
            let commit = head.peel_to_commit().unwrap();
            vec![commit]
        }
        None => vec![],
    };
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &repo.find_tree(oid).expect("could not find tree"),
        parent.iter().collect::<Vec<&Commit>>().as_slice(),
    )
    .expect("Failed to commit.")
}

pub fn amend(repo: &Repository, oid: Oid) {
    let commit = repo.find_commit(oid).unwrap();
    commit
        .amend(Some("HEAD"), None, None, None, None, None)
        .unwrap();
}

/// Tag the repo with the version.
pub fn tag(repo: &Repository, version: Version) {
    repo.tag(
        &version.to_tag(),
        &repo.revparse_single("HEAD").unwrap(),
        &Signature::now("Semantic Release", "info@tomhoward.codes")
            .expect("could not make signature"),
        "",
        false,
    )
    .unwrap();
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_versions() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("README.md");
        assert_eq!(versions(&repo), vec![]);

        update(&repo, relpath, "Hello world.", "Build: things");
        tag(&repo, Version::new(0, 1, 0));
        assert_eq!(versions(&repo).len(), 1);

        update(
            &repo,
            relpath,
            "Hello world, again.",
            "build: better things.",
        );
        tag(&repo, Version::new(0, 1, 1));
        assert_eq!(versions(&repo).len(), 2);
        assert_eq!(
            versions(&repo),
            vec![Version::new(0, 1, 0), Version::new(0, 1, 1)]
        );

        update(&repo, relpath, "Hello moon.", "build: best things.");
        tag(&repo, Version::new(0, 2, 0));
        assert_eq!(versions(&repo).len(), 3);
        assert_eq!(
            versions(&repo),
            vec![
                Version::new(0, 1, 0),
                Version::new(0, 1, 1),
                Version::new(0, 2, 0)
            ]
        );
    }

    #[test]
    fn test_walker() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");

        update(&repo, relpath, "[package]", "Initial Commit");
        assert_eq!(walker(&repo, None, None).count(), 1);

        update(&repo, relpath, "[package]\n", "build: test");
        assert_eq!(walker(&repo, None, None).count(), 2);

        let version = Version::new(0, 1, 0);
        tag(&repo, version.clone());
        assert_eq!(walker(&repo, None, Some(version)).count(), 2);
    }

    #[test]
    fn test_walkers() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"",
            "initial commit",
        );
        assert_eq!(walkers(&repo).len(), 1);
        assert_eq!(walkers(&repo).pop().unwrap().count(), 1);

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"\n",
            "build: bob",
        );
        assert_eq!(walkers(&repo).len(), 1);
        assert_eq!(walkers(&repo).pop().unwrap().count(), 2);

        tag(&repo, Version::new(0, 1, 0));
        assert_eq!(walkers(&repo).len(), 1);
        assert_eq!(walkers(&repo).pop().unwrap().count(), 2);

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.1\"",
            "build: version bump",
        );
        assert_eq!(walkers(&repo).len(), 2);
        assert_eq!(walkers(&repo).pop().unwrap().count(), 1);

        // Should be ..v0.1.0, v0.1.0..HEAD, each with one commit
        assert_eq!(walkers(&repo).len(), 2);
        assert_eq!(walkers(&repo).pop().unwrap().count(), 1);
        tag(&repo, Version::new(0, 1, 1));

        // Should be ..v0.1.0, v0.1.0..v0.1.1, each with one commit
        assert_eq!(walkers(&repo).len(), 2);

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.2\"",
            "build: version bump",
        );

        // ..v0.1.0, v0.1.0..v0.1.1, v0.1.1..HEAD
        assert_eq!(walkers(&repo).len(), 3);
        tag(&repo, Version::new(0, 1, 2));

        // ..v0.1.0, v0.1.0..v0.1.1, v0.1.1..v0.1.2
        assert_eq!(walkers(&repo).len(), 3);
    }
}
