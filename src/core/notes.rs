use git2::Repository;

use crate::utils;
use crate::*;

pub fn notes(repo: &Repository) -> Vec<String> {
    log::debug!("generating notes");
    let version: String = if utils::is_head_tagged(&repo) {
        "HEAD".to_string()
    } else {
        match utils::versions(&repo).last() {
            Some(version) => version.to_tag(),
            None => "HEAD".to_string(),
        }
    };
    let mut commits: Vec<String> = utils::walkers(&repo)
        .pop()
        .unwrap()
        .map(|c| repo.find_commit(c.unwrap()).unwrap())
        .map(utils::commit_message)
        .chain(vec![version])
        .collect();

    commits.reverse();
    commits
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_notes() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("README.md");
        update(&repo, relpath, "Title", "Initial Commit");
        assert_eq!(notes(&repo).len(), 2);
        assert!(notes(&repo).contains(&"HEAD".to_string()));
    }
}
