use std::error::Error;
use std::process;
use std::process::Command;
use std::string::String;

use ansi_term::Color::White;
use ansi_term::Colour::Black;
use ansi_term::Colour::Cyan;
use ansi_term::Colour::Green;
use ansi_term::Colour::Purple;
use ansi_term::Colour::Red;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;

use config::Config;

mod config;
mod git;
mod openai;

fn handle_error<T>(result: Result<T, Box<dyn Error>>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "\n\n{} Application error: {}",
                Red.bold().paint("✘"),
                Purple.bold().paint(err.to_string())
            );
            process::exit(1);
        }
    }
}

fn join_vec_indented<S: ToString>(v: Vec<S>) -> String {
    let mut result = String::new();
    for item in v.iter() {
        result.push_str(&format!("    {}\n", item.to_string()));
    }
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", Black.on(Cyan).paint("aicommits-rs"));

    let config: Config = handle_error(Config::new());

    let base_url = String::from("https://api.openai.com/v1/");

    let client = openai::async_client(&config.openai_api_key, &base_url)?;

    let staged_files: Vec<String> = handle_error(git::get_staged_files());

    println!(
        "{}",
        White.bold().paint(git::get_detected_message(&staged_files))
    );
    println!("{}", Green.bold().paint(join_vec_indented(staged_files)));

    let staged_diff: String = git::get_staged_diff()?;

    let mut spinner = spinners_rs::Spinner::new(
        spinners_rs::Spinners::Aesthetic,
        "The AI is analyzing your changes...",
    );

    spinner.start();

    let commit_message: String =
        handle_error(openai::generate_commit_message(&client, &staged_diff).await);

    spinner.stop_with_message("\n✓ Changes analyzed\n\n");

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "{}\n{}",
            "Use this commit message?",
            Green.bold().paint(commit_message.clone())
        ))
        .default(true)
        .show_default(false)
        .wait_for_newline(false)
        .interact()
        .unwrap()
    {
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(commit_message)
            .status()
            .expect("failed to execute process");
    }

    Ok(())
}
