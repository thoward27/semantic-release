use git2::Repository;

use crate::utils;
use crate::*;

pub fn bump(repo: &Repository) -> Bump {
    if utils::is_head_tagged(repo) {
        Bump::None
    } else {
        utils::walkers(repo)
            .pop()
            .unwrap()
            .map(|c| utils::commit_bump(&repo.find_commit(c.unwrap()).unwrap()))
            .max()
            .unwrap_or(Bump::None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_bump() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("README.md");

        update(&repo, relpath, "Hello world.", "Initial Commit");
        assert_eq!(bump(&repo), Bump::None);

        update(&repo, relpath, "Hello nightman.", "feat: better intro");
        assert_eq!(bump(&repo), Bump::Minor);

        update(
            &repo,
            relpath,
            "Hello dayman",
            "feat: best intro\nBREAKING CHANGE",
        );
        assert_eq!(bump(&repo), Bump::Major);
    }
}
