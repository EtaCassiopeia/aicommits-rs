use std::error::Error;
use std::process::Command;

fn assert_git_repo() -> Result<(), Box<dyn Error>> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&output.stdout);

    if output.trim() != "true" {
        return Err("Not a git repo".into());
    }
    Ok(())
}

pub fn get_staged_files() -> Result<Vec<String>, Box<dyn Error>> {
    assert_git_repo()?;

    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--name-only")
        .output()
        .expect("failed to execute process");

    if output.stdout.len() == 0 {
        return Err(
            "No staged changes found. Make sure to stage your changes with `git add`.".into(),
        );
    }

    let output = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let files: Vec<String> = output.split_whitespace().map(|s| s.to_string()).collect();

    Ok(files)
}

pub fn get_detected_message(files: Vec<String>) -> String {
    format!(
        "Detected {} staged file{}",
        files.len().to_string(),
        if files.len() > 1 { "s" } else { "" }
    )
}

pub fn get_staged_diff() -> Result<String, Box<dyn Error>> {
    assert_git_repo()?;

    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .expect("failed to execute process");

    if output.stdout.len() == 0 {
        return Err(
            "No staged changes found. Make sure to stage your changes with `git add`.".into(),
        );
    }

    Ok(String::from_utf8(output.stdout).map_err(|e| e.to_string())?)
}
