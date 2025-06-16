mod filters;

use std::{
    error::Error,
    fs,
    io::{BufRead, stdin},
    path::PathBuf,
};

use clap::Parser;
use inventory_utils::Ean13;
use minijinja::context;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Label {
    price: Decimal,
    sku: String,
    desc: String,
    ean13: Ean13,
    alt_skus: Vec<String>,
    img: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut env = minijinja::Environment::new();
    env.add_filter("pretty_price", filters::pretty_price);
    env.add_filter("encode_barcode", filters::encode_barcode);
    env.add_filter("format_ean13", filters::format_ean13);
    env.add_template("base.html", include_str!("templates/base.html"))
        .or_else(|e| Err(format!("Cannot load label template due to {}", e)))?;
    let template = env
        .get_template("base.html")
        .or_else(|e| Err(format!("Cannot open label template due to {}", e)))?;

    let cli = Cli::parse();
    let raw_text = cli.read_input()?;
    let labels: Vec<Label> = serde_json::from_str(&raw_text)
        .or_else(|e| Err(format!("Cannot parse label data from input due to {}", e)))?;
    let pages: Vec<Vec<Label>> = labels.chunks(30).map(|chunk| chunk.to_vec()).collect();
    let render = template
        .render(context! {
            pages => pages,
        })
        .or_else(|e| Err(format!("Cannot render template due to {}", e)))?;
    fs::write("out.html", render)
        .or_else(|e| Err(format!("Cannot write output html due to {}", e)))?;
    Ok(())
}
