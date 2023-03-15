use std::{error::Error, process::Command, string::String};

use ansi_term::{
    Color::White,
    Colour::{Black, Cyan, Green, Purple, Red, Yellow},
};
use dialoguer::console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use spinners_rs::{Spinner, Spinners};

use crate::{
    config::Config,
    git::{get_staged_files, StagedFiles},
    openai::{GitCommitMessageGenerator, OpenAiClient},
};

mod config;
mod git;
mod openai;

//let result = handle_error!(some_function_that_returns_result());
macro_rules! handle_error {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                eprintln!(
                    "\n\n{} {}",
                    Red.bold().paint("✘"),
                    Purple.bold().paint(err.to_string())
                );
                std::process::exit(1);
            }
        }
    };
}

pub trait PostfixHandleError<T> {
    //TODO Is there a way to make this unary postfix operator? (e.g. result! instead of result.handle_error())
    fn handle_error(self) -> T;
}

impl<T> PostfixHandleError<T> for Result<T, Box<dyn Error>> {
    fn handle_error(self) -> T {
        handle_error!(self)
    }
}

struct AICommits {
    git_commit_message_generator: GitCommitMessageGenerator,
    staged_files: StagedFiles,
}

impl AICommits {
    fn new(client: OpenAiClient) -> Result<Self, Box<dyn Error>> {
        let git_commit_message_generator = GitCommitMessageGenerator::new(client);
        let staged_files = get_staged_files()?;
        println!(
            "{}",
            White.bold().paint(staged_files.get_detected_message())
        );
        println!(
            "{}",
            Green.bold().paint(join_vec_indented(&staged_files.files))
        );
        Ok(Self {
            git_commit_message_generator,
            staged_files,
        })
    }

    fn use_commit_message(&self, commit_message: String) -> Result<(), Box<dyn Error>> {
        let message_prompt = format!(
            "{}\n{}",
            "Use this commit message?",
            Green.bold().paint(commit_message.clone())
        );
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(message_prompt)
            .default(true)
            .show_default(false)
            .wait_for_newline(false)
            .interact()?
        {
            let items = ["feat", "fix", "nano", "BREAKING-CHANGE"];
            println!(
                "{}",
                Yellow
                    .bold()
                    .paint("Select the type of commit you want to make:")
            );
            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&items)
                .default(0)
                .interact_on_opt(&Term::stderr())?;
            let commit_message = format!("{}: {}", items[selection.unwrap()], commit_message);
            Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(commit_message)
                .status()?;
        }
        Ok(())
    }
}

fn join_vec_indented<S: ToString>(v: &[S]) -> String {
    v.iter()
        .map(|item| format!("    {}\n", item.to_string()))
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", Black.on(Cyan).paint("aicommits-rs"));
    let config = Config::new()?;
    let base_url = "https://api.openai.com/v1/".to_string();
    let client = OpenAiClient::new(&config.openai_api_key, &base_url)?;
    let ai_commits = AICommits::new(client)?;
    let mut spinner = Spinner::new(Spinners::Aesthetic, "The AI is analyzing your changes...");
    spinner.start();
    let commit_message = (ai_commits
        .git_commit_message_generator
        .generate_commit_message(&ai_commits.staged_files.diff)
        .await)
        .handle_error();
    spinner.stop_with_message("\n✓ Changes analyzed\n\n");
    ai_commits.use_commit_message(commit_message)?;
    Ok(())
}
