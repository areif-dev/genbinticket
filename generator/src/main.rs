mod template_env;

use std::{
    error::Error,
    fs,
    io::{BufRead, stdin},
    path::PathBuf,
};

use clap::Parser;
use generator::Label;
use minijinja::context;

#[derive(Parser)]
struct Cli {
    /// Apply no sorting and print the labels in the order they were supplied
    #[arg(long, short)]
    preserve_order: bool,

    /// Read input from a file
    #[arg(long, short)]
    file: Option<PathBuf>,
}

impl Cli {
    fn read_input(&self) -> Result<String, String> {
        Ok(match &self.file {
            Some(f) => fs::read_to_string(f)
                .or_else(|e| Err(format!("Could not read data file due to {}", e)))?,
            None => {
                let mut lines = stdin().lock().lines();
                let mut full = String::new();
                while let Some(line) = lines.next() {
                    full = format!(
                        "{}{}",
                        full,
                        line.or_else(|e| Err(format!("Cannot read line from stdin due to {}", e)))?
                    );
                }
                full
            }
        })
    }
}

fn main() -> Result<(), String> {
    let env = template_env::setup_env().or_else(|e| {
        Err(format!(
            "Failed to configure the label templates due to {}",
            e
        ))
    })?;

    let cli = Cli::parse();
    let raw_text = cli.read_input()?;
    let labels: Vec<Label> = serde_json::from_str(&raw_text)
        .or_else(|e| Err(format!("Cannot parse label data from input due to {}", e)))?;
    let render = template_env::render_template(&env, &labels)
        .or_else(|e| Err(format!("Failed to render label template due to {}", e)))?;
    fs::write("out.html", render)
        .or_else(|e| Err(format!("Cannot write output html due to {}", e)))?;
    Ok(())
}
