aicommits-rs
------------

[![Build Status](https://travis-ci.org/andrewthauer/aicommits-rs.svg?branch=master)](https://travis-ci.org/andrewthauer/aicommits-rs)

A CLI that writes your git commit messages for you with AI. This project was inspired by [AI Commits](https://github.com/Nutlope/aicommits) and has been ported to Rust.

We were inspired by the AI Commits project and have ported it to Rust. We hope to build on its success and take this project even further.

[![asciicast](https://asciinema.org/a/OgAALBPROYiY01EtJ9Ovuj5ty.svg)](https://asciinema.org/a/OgAALBPROYiY01EtJ9Ovuj5ty)

## Installation

1. Install Rust and Cargo. Then run:

```bash
cargo install aicommits-rs
```

2. Retrieve your API key from [OpenAI](https://platform.openai.com/account/api-keys)

   > Note: If you haven't already, you'll have to create an account and set up billing.
3. Set your API key as an environment variable:

```bash
export OPENAI_API_KEY=<your_api_key>
```

## Usage

```bash
aicommits-rs
```

## How it works

This CLI tool uses `git diff` to obtain all of the most recent code changes, and then sends them to OpenAI's GPT-3 to generate a commit message that is returned. If the returned commit message is not satisfactory, you can run the command again to generate a new commit message, otherwise you can use the returned commit message to commit your changes.

> Note: An error may occur when the number of changes being made to a file is too large for the OpenAI API to process. To avoid this error, try reducing the number of staged changes or writing your own commit message. 

> Note: This tool is still in development, and is not guaranteed to work as expected.


