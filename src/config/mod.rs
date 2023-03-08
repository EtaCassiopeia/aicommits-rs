use std::env;
use std::error::Error;

pub struct Config {
    pub openai_api_key: String,
}

impl Config {
    pub fn new() -> Result<Config, Box<dyn Error>> {
        let openai_api_key: String = env::var("OPENAI_API_KEY")?;
        Ok(Config { openai_api_key })
    }
}
