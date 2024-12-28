use git2::{Repository, Error};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use tempfile::TempDir;


fn main(){
    let repo_path = get_repo_path();

    match prepare_repo(&repo_path){
        Ok(temp_repo) => {
            match analyze_repo(temp_repo.local_path()){
                Ok(analysis) => {
                    println!("\n=== Git Repository Analysis ===\n");
                    println!("Repository {}", analysis.repo_path);
                    println!("Branches {}",analysis.branches.len());
                    println!("Commits {}", analysis.commit_count);
                    println!("Contributors {}", analysis.contributors.len());
                    println!("Files {}", analysis.file_count);
                    println!("\nBranches:");
                    for branch in analysis.branches {
                        println!("  {}", branch);
                    }
                    println!("\nContributors:");
                    for (contributor, count) in analysis.contributors.iter() {
                        println!("  {}: {}", contributor, count);
                    }
                }
                Err(e) => {
                    println!("Error analyzing repository: {}", e);
                }
            }

            if temp_repo.is_temporary {
                println!("Cleaning up temporary repository...");
                drop(temp_repo.temp_dir);
            }
        }
        Err(e) => {
            println!("Error preparing repository: {}", e);
        }
    }
}

fn get_repo_path() -> String {
    loop {
        print!("Enter the path or URL to the Git repository: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let path = input.trim().to_string();

        if path.is_empty() {
            println!("Please enter a valid path or URL.");
        } else {
            return path;
        }
    }
}

struct TempRepo {
    is_temporary: bool,
    temp_dir: Option<TempDir>,
    local_path: String,
}

impl TempRepo {
    fn local_path(&self) -> &str {
        &self.local_path
    }
}

fn prepare_repo(path: &str) -> Result<TempRepo, Box<dyn std::error::Error>> {
    if Path::new(path).exists() {
        // Local repository path
        Ok(TempRepo {
            is_temporary: false,
            temp_dir: None,
            local_path: path.to_string(),
        })
    } else {
        // Remote repository URL
        println!("Cloning remote repository...");
        let temp_dir = TempDir::new()?;
        let local_path = temp_dir.path().to_string_lossy().to_string();
        Repository::clone(path, &local_path)?;
        Ok(TempRepo {
            is_temporary: true,
            temp_dir: Some(temp_dir),
            local_path,
        })
    }
}

struct RepoAnalysis {
    repo_path: String,
    branches: Vec<String>,
    commit_count: usize,
    contributors: HashMap<String, usize>,
    file_count: usize,
}

fn analyze_repo(path: &str) -> Result<RepoAnalysis, Error> {
    let repo = Repository::open(path)?;

    // Analyze branches
    let mut branches = Vec::new();
    for branch in repo.branches(None)? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            branches.push(name.to_string());
        }
    }

    
    let mut contributors = HashMap::new();
    let mut commit_count = 0;
    if let Ok(head) = repo.head() {
        let oid = head.target().ok_or_else(|| Error::from_str("No HEAD target"))?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push(oid)?;

        for commit_id in revwalk {
            if let Ok(commit) = repo.find_commit(commit_id?) {
                let author = commit.author().name().unwrap_or("Unknown").to_string();
                *contributors.entry(author).or_insert(0) += 1;
                commit_count += 1;
            }
        }
    }

    
    let mut file_count = 0;
    if let Ok(tree) = repo.find_tree(repo.head()?.peel_to_commit()?.tree_id()) {
        tree.walk(git2::TreeWalkMode::PreOrder, |_, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                file_count += 1;
            }
            0 
        })?;
    }

    Ok(RepoAnalysis {
        repo_path: path.to_string(),
        branches,
        commit_count,
        contributors,
        file_count,
    })
}