use git2::Repository;

use crate::*;

pub fn version(repo: &Repository) -> Version {
    log::debug!("running version");
    let mut proposed = languages::get(&repo).expect("couldn't get version");
    match core::bump::bump(&repo) {
        Bump::Major => {
            if proposed.major == 0 {
                proposed.increment_minor()
            } else {
                proposed.increment_major()
            }
        }
        Bump::Minor => proposed.increment_minor(),
        Bump::Patch => proposed.increment_patch(),
        Bump::Prerelease => (), // TODO: should increment rc
        Bump::None => (),
    };
    proposed
}

#[cfg(test)]
mod test {
    use super::test_utils::*;
    use super::*;

    #[test]
    fn test_update_rust() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("Cargo.toml");

        update(
            &repo,
            relpath,
            "[package]\nversion=\"0.1.0\"",
            "Initial Commit",
        );
        let v1 = version(&repo);
        assert_eq!(v1, languages::get(&repo).unwrap());

        update(
            &repo,
            Path::new("README.md"),
            "really cool thing",
            "feat: just wow",
        );
        let v2 = version(&repo);
        assert_ne!(v1, v2);
    }
}
