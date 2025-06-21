mod product;

use clap::ValueEnum;
use std::{cmp::Ordering, env::current_dir, fs, path::PathBuf};

use clap::Parser;
use generator::{Label, template_env};

#[derive(Clone, ValueEnum)]
enum SortOption {
    Upc,
    Sku,
    Date,
    Preserve,
}

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

    #[arg(short, long, default_value = "preserve")]
    sort: SortOption,
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

fn sort_labels(labels: &mut [Label], sort_option: SortOption) {
    match sort_option {
        SortOption::Upc => labels.sort_by(|a, b| {
            let a = match a.ean13() {
                Some(e) => e.to_string(),
                None => return Ordering::Less,
            };
            let b = match b.ean13() {
                Some(e) => e.to_string(),
                None => return Ordering::Greater,
            };
            let trans_a = format!("{}{}", &a[10..], &a[..11]);
            let trans_b = format!("{}{}", &b[10..], &b[..11]);
            trans_a.cmp(&trans_b)
        }),
        SortOption::Sku => labels.sort_by(|a, b| {
            let a = match a.sku() {
                Some(s) => s,
                None => return Ordering::Less,
            };
            let b = match b.sku() {
                Some(s) => s,
                None => return Ordering::Greater,
            };
            a.cmp(&b)
        }),
        SortOption::Date => labels.sort_by(|a, b| {
            let a = match a.date() {
                Some(d) => d,
                None => return Ordering::Less,
            };
            let b = match b.date() {
                Some(d) => d,
                None => return Ordering::Greater,
            };
            a.cmp(&b)
        }),
        SortOption::Preserve => (),
    }
}

fn main() -> Result<(), String> {
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
    let mut labels: Vec<Label> = good_skus
        .into_iter()
        .filter_map(|(sku, qty)| {
            let product = all_products.get(&sku).unwrap();
            product.label(&mut img_cache, qty)
        })
        .collect();
    sort_labels(&mut labels, cli.sort);

    // Save the updated image cache for faster fetching next time
    product::write_cached_imgs(&img_cache)
        .or_else(|e| Err(format!("Can't save product image cache due to {}", e)))?;

    let env = template_env::setup_env()
        .or_else(|e| Err(format!("Failed to configure label template due to {}", e)))?;
    let render = template_env::render_template(&env, &labels)
        .or_else(|e| Err(format!("Failed to render label template due to {}", e)))?;
    fs::write("./out.html", render)
        .or_else(|e| Err(format!("Cannot write template to save file due to {}", e)))?;
    let mut file = current_dir().or_else(|e| {
        Err(format!(
            "Failed to get path to working directory due to {}",
            e
        ))
    })?;
    file.push("out.html");
    webbrowser::open(&format!("file://{}", file.display())).or(Err(format!(
        "Failed to open web browser. Please see the file out.html for your labels"
    )))?;
    Ok(())
}
