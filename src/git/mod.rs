use std::error::Error;
use std::process::{Command, Output};

struct GitRepo;

trait GitRepoOps {
    fn is_git_repo(&self) -> bool;
}

impl GitRepoOps for GitRepo {
    fn is_git_repo(&self) -> bool {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--is-inside-work-tree")
            .output()
            .expect("failed to execute process");

        let output = String::from_utf8_lossy(&output.stdout);

        output.trim() == "true"
    }
}

pub struct StagedFiles {
    pub files: Vec<String>,
    pub diff: String,
}

impl StagedFiles {
    fn from_output(output: Output) -> Result<Self, Box<dyn Error>> {
        let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;

        if stdout.is_empty() {
            return Err(
                "No staged changes found. Make sure to stage your changes with `git add`.".into(),
            );
        }

        let files: Vec<String> = stdout.split_whitespace().map(|s| s.to_string()).collect();
        let diff: String = Self::get_diff()?;

        Ok(Self { files, diff })
    }

    pub fn get_detected_message(&self) -> String {
        format!(
            "Detected {} staged file{}:",
            self.files.len(),
            if self.files.len() > 1 { "s" } else { "" }
        )
    }

    fn get_diff() -> Result<String, Box<dyn Error>> {
        let output = Command::new("git")
            .arg("diff")
            .arg("--cached")
            .output()
            .expect("failed to execute process");

        Ok(String::from_utf8(output.stdout).map_err(|e| e.to_string())?)
    }
}

pub fn get_staged_files() -> Result<StagedFiles, Box<dyn Error>> {
    let git_repo = GitRepo;

    if !git_repo.is_git_repo() {
        return Err("Not a git repo".into());
    }

    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--name-only")
        .output()
        .expect("failed to execute process");

    StagedFiles::from_output(output)
}
