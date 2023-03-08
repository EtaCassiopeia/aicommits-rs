use config::Config;
use std::error::Error;
use std::process;

mod config;
mod git;
mod openai;

fn handle_error<T>(result: Result<T, Box<dyn Error>>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Application error: {err}");
            process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = Config::new()?;

    let base_url = String::from("https://api.openai.com/v1/");

    let client = openai::async_client(&config.openai_api_key, &base_url)?;

    let staged_files: Vec<String> = handle_error(git::get_staged_files());

    println!("{}", git::get_detected_message(staged_files));

    let staged_diff: String = git::get_staged_diff()?;

    let commit_message: String =
        handle_error(openai::generate_commit_message(&client, &staged_diff).await);

    println!("Commit message: {}", commit_message);

    Ok(())
}
