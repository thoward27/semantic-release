use git2::Repository;

use crate::*;

pub fn changelog(repo: &Repository) -> Vec<String> {
    log::debug!("generating changelog");
    let versions: Vec<String> = utils::versions(repo).iter().map(|v| v.to_tag()).collect();
    let mut messages: Vec<String> = vec![];
    for (index, walker) in utils::walkers(repo).into_iter().enumerate() {
        let commits: Vec<String> = walker
            .map(|c| repo.find_commit(c.unwrap()).unwrap())
            .map(utils::commit_message)
            .collect();
        messages.extend(commits);
        messages.extend(vec![format!(
            "\n## {}",
            versions.get(index).unwrap_or(&"wip".to_string()).clone()
        )]);
    }
    messages.reverse();
    messages
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_changelog() {
        let dir = tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let relpath = Path::new("CHANGELOG.md");
        update(&repo, relpath, "What are the rules.", "Initial commit");
        assert_eq!(changelog(&repo).len(), 2);
    }
}
