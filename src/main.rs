mod product;
mod server;

use ean13::Ean13;
use genbinticket::Label;
use product::AbcProduct;
use server::start_server;
use std::collections::HashMap;

use clap::Parser;

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
        default_value = "C:\\ABC Software\\Database Export\\Company001\\Data\\item.data"
    )]
    detail_file: String,

    /// Path to the ABC Item Posted Data file obtained by running report 7-10 (SQL Export)
    #[arg(
        short,
        long,
        default_value = "C:\\ABC Software\\Database Export\\Company001\\Data\\item_posted.data"
    )]
    posted_file: String,

    /// Enable debug mode, which starts the server on 0.0.0.0
    #[arg(short = 'D', long)]
    debug: bool,
}

pub fn read_216(tabfile: &str) -> Result<Vec<(String, Option<u32>)>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(false)
        .from_path(tabfile)
        .or_else(|e| Err(format!("Can't open {} due to {}", tabfile, e)))?;

    let mut skus_qtys = Vec::new();
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
        let qty: Option<u32> = match record.get(6) {
            Some(s) => s.parse().ok(),
            None => None,
        };

        skus_qtys.push((sku.to_string(), qty));
    }
    Ok(skus_qtys)
}

async fn labels_from_skus(
    skus_qtys: Vec<(String, Option<u32>)>,
    all_products: &HashMap<String, AbcProduct>,
    img_cache: &mut HashMap<Ean13, String>,
) -> Vec<Label> {
    let mut labels = Vec::new();
    for (sku, qty) in skus_qtys {
        let Some(product) = all_products.get(&sku) else {
            continue;
        };
        let Some(label) = product.label(img_cache, qty).await else {
            continue;
        };
        labels.push(label);
    }
    labels
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let skus_qtys = read_216(&cli.tabfile)?;
    let mut img_cache = product::read_cached_imgs();
    let all_products = product::parse_abc_item_files(&cli.detail_file, &cli.posted_file)
        .or_else(|e| Err(format!("Cannot read ABC inventory export due to {}", e)))?;
    let (good_skus, failed_skus): (Vec<(String, Option<u32>)>, Vec<(String, Option<u32>)>) =
        skus_qtys
            .into_iter()
            .partition(|(sku, _)| all_products.contains_key(sku));
    if failed_skus.len() > 0 {
        eprintln!(
            "The following skus were not found in the ABC Inventory Export: {:?}",
            failed_skus
        );
    }
    eprintln!("Fetching images. This may take some time...");
    let labels = labels_from_skus(good_skus, &all_products, &mut img_cache).await;

    // Save the updated image cache for faster fetching next time
    product::write_cached_imgs(&img_cache)
        .or_else(|e| Err(format!("Can't save product image cache due to {}", e)))?;

    start_server(labels, cli.debug)
        .await
        .or_else(|e| Err(format!("Failed to start server due to {}", e)))?;
    Ok(())
}
