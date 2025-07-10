use crate::Result;
use git2::{Repository, Signature};
use std::path::Path;

pub struct GitContext {
    repo: Repository,
}

impl GitContext {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }
    
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::init(path)?;
        Ok(Self { repo })
    }
    
    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses.is_empty())
    }
    
    pub fn add_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_path(path.as_ref())?;
        index.write()?;
        Ok(())
    }
    
    pub fn commit(&self, message: &str) -> Result<()> {
        let signature = Signature::now("Tessera", "tessera@example.com")?;
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let parent_commit = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };
        
        let parents = match &parent_commit {
            Some(commit) => vec![commit],
            None => vec![],
        };
        
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        
        Ok(())
    }
    
    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let branch_name = head.shorthand().unwrap_or("HEAD");
        Ok(branch_name.to_string())
    }
    
    pub fn get_last_commit_info(&self) -> Result<(String, String)> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        let message = commit.message().unwrap_or("No message").to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        Ok((message, author))
    }
}