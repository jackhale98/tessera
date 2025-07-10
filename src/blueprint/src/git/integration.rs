use anyhow::Result;
use git2::{Repository, Signature};
use std::path::Path;

pub struct GitIntegration;

impl GitIntegration {
    pub fn init_repository<P: AsRef<Path>>(path: P) -> Result<Repository> {
        let repo = Repository::init(path)?;
        Ok(repo)
    }

    pub fn add_and_commit<P: AsRef<Path>>(
        repo_path: P,
        file_path: &Path,
        message: &str,
    ) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        let mut index = repo.index()?;

        // Add file to index
        index.add_path(file_path)?;
        index.write()?;

        // Create commit
        let signature = Signature::now("Blueprint", "blueprint@example.com")?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let parent_commit = match repo.head() {
            Ok(head) => {
                let oid = head.target().unwrap();
                Some(repo.find_commit(oid)?)
            }
            Err(_) => None,
        };

        let parent_commits = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_commits,
        )?;

        Ok(())
    }

    pub fn create_branch(repo_path: &Path, branch_name: &str) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        let head = repo.head()?;
        let oid = head.target().unwrap();
        let commit = repo.find_commit(oid)?;

        repo.branch(branch_name, &commit, false)?;

        // Checkout the new branch
        let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
        repo.checkout_tree(&obj, None)?;
        repo.set_head(&format!("refs/heads/{}", branch_name))?;

        Ok(())
    }

    pub fn get_current_branch(repo_path: &Path) -> Result<String> {
        let repo = Repository::open(repo_path)?;
        let head = repo.head()?;

        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    }
}
