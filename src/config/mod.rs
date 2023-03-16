use std::env;
use std::error::Error;

pub struct Config {
    pub openai_api_key: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let openai_api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not found")?;

        Ok(Self { openai_api_key })
    }
}
