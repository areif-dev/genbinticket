mod product;
mod server;

use std::collections::HashMap;

use clap::Parser;
use ean13::Ean13;
use genbinticket::Label;
use product::AbcProduct;
use server::start_server;
use vendor_controller::VendorController;
use vendor_controllers::{ControllerWrapper, dib::DibController, ids::IdsController};

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

async fn vendors_from_216(tabfile: &str) -> Result<HashMap<String, ControllerWrapper>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(false)
        .from_path(tabfile)
        .or_else(|e| Err(format!("Can't open {} due to {}", tabfile, e)))?;

    let mut vendors = HashMap::new();
    for result in rdr.records().skip(1) {
        let record =
            result.or_else(|e| Err(format!("Bad row in bill TabOutput caused by {}", e)))?;

        if let Some("ITEM # & DESCRIPTION") = record.get(17) {
            if let Some(vend) = record.get(2) {
                // Skip this vendor because we've already handled it if it's in the vendor map
                if let Some(_) = vendors.get(vend) {
                    continue;
                }
                match vend {
                    "DO ITB0" => {
                        let Ok(user) = std::env::var("DIB_USER") else {
                            eprintln!("No env var for DIB_USER. Skipping");
                            continue;
                        };
                        let Ok(passwd) = std::env::var("DIB_PASSWD") else {
                            eprintln!("No env var for DIB_PASSWD. Skipping");
                            continue;
                        };
                        let dib_controller = match DibController::new(5000).await {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("Failed to create DIB Controller due to {}. Skipping", e);
                                continue;
                            }
                        };
                        // let Ok(dib_controller) = DibController::new().await else {
                        //     eprintln!("Failed to create DIB Controller. Will skip it.");
                        //     continue;
                        // };
                        let dib_controller = dib_controller.user(user).passwd(passwd);
                        if let Err(e) = dib_controller.login().await {
                            eprintln!("Failed to login DibController due to {}", e);
                            continue;
                        };

                        vendors.insert(
                            String::from("DO ITB0"),
                            ControllerWrapper::Dib(dib_controller),
                        );
                    }
                    "FLOHAL0" => {
                        let Ok(user) = std::env::var("IDS_USER") else {
                            eprintln!("No env var for IDS_USER. Skipping");
                            continue;
                        };
                        let Ok(passwd) = std::env::var("IDS_PASSWD") else {
                            eprintln!("No env var for IDS_PASSWD. Skipping");
                            continue;
                        };
                        let Ok(ids_controller) = IdsController::new(5000).await else {
                            eprintln!("Failed to create IDS Controller. Will skip it.");
                            continue;
                        };
                        let ids_controller = ids_controller.user(user).passwd(passwd);
                        if let Err(e) = ids_controller.login().await {
                            eprintln!("Failed to login IdsController due to {}", e);
                            continue;
                        }

                        vendors.insert(
                            String::from("FLOHAL0"),
                            ControllerWrapper::Ids(ids_controller),
                        );
                    }
                    others => {
                        eprintln!("No controller implemented for {}", others);
                        continue;
                    }
                }
            }
            continue;
        }
    }
    Ok(vendors)
}

fn skus_qtys_from_216(tabfile: &str) -> Result<Vec<(String, Option<u32>)>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(false)
        .from_path(tabfile)
        .or_else(|e| Err(format!("Can't open {} due to {}", tabfile, e)))?;

    let mut skus_qtys = Vec::new();
    for result in rdr.records().skip(1) {
        let record =
            result.or_else(|e| Err(format!("Bad row in bill TabOutput caused by {}", e)))?;

        if let Some("ITEM # & DESCRIPTION") = record.get(17) {
            continue;
        }

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
    vendors: &HashMap<String, ControllerWrapper>,
) -> Vec<Label> {
    let mut labels = Vec::new();
    for (sku, qty) in skus_qtys {
        let Some(product) = all_products.get(&sku) else {
            continue;
        };
        let Some(label) = product.label(img_cache, qty, vendors).await else {
            continue;
        };
        labels.push(label);
    }
    labels
}

#[tokio::main]
async fn main() -> Result<(), String> {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load env vars due to {}", e);
    }

    let cli = Cli::parse();
    let skus_qtys = skus_qtys_from_216(&cli.tabfile)?;
    let vendors = vendors_from_216(&cli.tabfile).await?;
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
    let labels = labels_from_skus(good_skus, &all_products, &mut img_cache, &vendors).await;

    // Save the updated image cache for faster fetching next time
    product::write_cached_imgs(&img_cache)
        .or_else(|e| Err(format!("Can't save product image cache due to {}", e)))?;

    start_server(labels, cli.debug)
        .await
        .or_else(|e| Err(format!("Failed to start server due to {}", e)))?;
    Ok(())
}
