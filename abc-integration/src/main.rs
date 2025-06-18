mod product;

use std::{
    io::Write,
    process::{self, Stdio},
};

use clap::Parser;
use generator::Label;

#[derive(Parser)]
struct Cli {
    /// Path the the TSV file containing the data for ABC report 2-16 (Bill Details)
    #[arg(
        short,
        long,
        default_value = "C:\\Users\\User\\Documents\\My ABC Files\\TabOutput.tsv"
    )]
    tabfile: String,

    /// Path to the ABC Item Detail file obtained by running report 7-10 (SQL Export)
    #[arg(
        short,
        long,
        default_value = "C:\\ABC Software\\Database Export\\Company001\\Data\\item_detail.data"
    )]
    detail_file: String,

    /// Path to the ABC Item Posted Data file obtained by running report 7-10 (SQL Export)
    #[arg(
        short,
        long,
        default_value = "C:\\ABC Software\\Database Export\\Company001\\Data\\item_posted.data"
    )]
    posted_file: String,
}

pub fn read_216(tabfile: &str) -> Result<Vec<String>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(false)
        .from_path(tabfile)
        .or_else(|e| Err(format!("Can't open {} due to {}", tabfile, e)))?;

    let mut skus = Vec::new();
    for (i, result) in rdr.records().skip(2).enumerate() {
        // ABC 2-16 report splits bills at every 100 items and inserts a new header. Always skip
        // that header
        if i > 0 && i % 100 == 0 {
            continue;
        }

        let record =
            result.or_else(|e| Err(format!("Bad row in bill TabOutput caused by {}", e)))?;
        let sku = match record.get(1) {
            Some(s) => s,
            None => {
                continue;
            }
        };
        if sku == "" {
            continue;
        }
        skus.push(sku.to_string());
    }
    Ok(skus)
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let skus = read_216(&cli.tabfile)?;
    let mut img_cache = product::read_cached_imgs();
    let all_products = product::parse_abc_item_files(&cli.detail_file, &cli.posted_file)
        .or_else(|e| Err(format!("Cannot read ABC inventory export due to {}", e)))?;
    let (good_skus, failed_skus): (Vec<String>, Vec<String>) = skus
        .into_iter()
        .partition(|sku| all_products.contains_key(sku));
    if failed_skus.len() > 0 {
        eprintln!(
            "The following skus were not found in the ABC Inventory Export: {:?}",
            failed_skus
        );
    }
    let labels: Vec<Label> = good_skus
        .into_iter()
        .filter_map(|sku| {
            let product = all_products.get(&sku).unwrap();
            product.label(&mut img_cache)
        })
        .collect();

    // Save the updated image cache for faster fetching next time
    product::write_cached_imgs(&img_cache)
        .or_else(|e| Err(format!("Can't save product image cache due to {}", e)))?;

    let mut child = process::Command::new("./generator")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .or_else(|e| Err(format!("Cannot spawn the generator process due to {}", e)))?;
    if let Some(mut stdin) = child.stdin.take() {
        let input_string = serde_json::to_string(&labels).or_else(|e| {
            Err(format!(
                "Can't pass labels to the label generator due to {}",
                e
            ))
        })?;
        writeln!(&mut stdin, "{}", input_string)
            .or_else(|e| Err(format!("Cannot send label info to generator due to {}", e)))?;
    }
    let status = child.wait().or_else(|e| {
        Err(format!(
            "Label generator failed to execute because of {}",
            e
        ))
    })?;
    if !status.success() {
        return Err(format!(
            "Label generator failed. Here is a copy of its output {:?}",
            child.stderr
        ))?;
    }
    webbrowser::open("file://./out.html").or(Err(format!(
        "Failed to open web browser. Please see the file out.html for your labels"
    )))?;
    Ok(())
}
