use chrono::Utc;
use genbinticket::Label;
use rust_decimal::Decimal;
use serde::ser::Error;
use std::{
    collections::HashMap,
    fs::{self, File},
    str::FromStr,
};
use vendor_controller::VendorController;
use vendor_controllers::ControllerWrapper;

use ean13::Ean13;

/// Strips out any characters from a string that would not be part of a base10 decimal number. EG
/// "$" or the string "USD"
///
/// # Arguments
///
/// * `price_str` - The raw, unfiltered string to parse a number from
///
/// # Returns
///
/// A [`rust_decimal::Decimal`] number representing an exact price as extracted from `price_str`
///
/// # Errors
///
/// Forwards any [`rust_decimal::Error`] encountered while parsing data
fn price_from_str(price_str: &str) -> Result<Decimal, rust_decimal::Error> {
    let price_str: String = price_str
        .chars()
        .filter(|c| c.is_digit(10) || c == &'.')
        .collect();
    Decimal::from_str(&price_str)
}

pub fn read_cached_imgs() -> HashMap<Ean13, String> {
    let raw_text = fs::read_to_string("cached-imgs.json").unwrap_or(String::new());
    serde_json::from_str(&raw_text).unwrap_or(HashMap::new())
}

pub fn write_cached_imgs(cache: &HashMap<Ean13, String>) -> Result<(), std::io::Error> {
    let file = File::create("cached-imgs.json")?;
    serde_json::to_writer(file, cache)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct AbcProduct {
    sku: String,
    alt_skus: Vec<String>,
    desc: String,
    upcs: Vec<Ean13>,
    list: Decimal,
    stock: f64,
    last_sold: Option<chrono::NaiveDate>,
}

impl AbcProduct {
    pub fn sku(&self) -> String {
        self.sku.clone()
    }

    pub fn desc(&self) -> String {
        self.desc.clone()
    }

    pub fn upcs(&self) -> Vec<Ean13> {
        self.upcs.to_vec()
    }

    pub fn list(&self) -> Decimal {
        self.list
    }

    async fn fetch_img(
        &self,
        cache: &mut HashMap<Ean13, String>,
        vendors: &HashMap<String, ControllerWrapper>,
    ) -> Option<String> {
        let ean = self.upcs().last()?.clone();
        if let Some(url) = cache.get(&ean) {
            return Some(url.to_string());
        }

        let mut img = None;
        for (_name, vendor) in vendors {
            match vendor {
                ControllerWrapper::Ids(controller) => {
                    let Ok(Some(prod)) = controller.product_from_ean(ean.clone()).await else {
                        continue;
                    };
                    img = Some(prod.get_img_url());
                }
                ControllerWrapper::Dib(controller) => {
                    let Ok(Some(prod)) = controller.product_from_ean(ean.clone()).await else {
                        continue;
                    };
                    img = Some(prod.get_img_url());
                }
                ControllerWrapper::Bci(controller) => {
                    let Ok(Some(prod)) = controller.product_from_ean(ean.clone()).await else {
                        continue;
                    };
                    img = Some(prod.get_img_url());
                }
            }
        }

        if let Some(i) = img.clone() {
            cache.insert(ean.clone(), i.to_string());
        }

        img
    }

    pub async fn label(
        &self,
        cache: &mut HashMap<Ean13, String>,
        qty: Option<u32>,
        vendors: &HashMap<String, ControllerWrapper>,
    ) -> Option<Label> {
        let upc = self.upcs().last()?.clone();

        let mut label = match self.fetch_img(cache, vendors).await {
            Some(i) => Label::new().with_img(&i),
            None => Label::new(),
        };

        if let Some(q) = qty {
            label = label.with_qty(q);
        }
        for sku in &self.alt_skus {
            label.push_alt(sku);
        }
        Some(
            label
                .with_sku(&self.sku())
                .with_desc(&self.desc())
                .with_price(self.list())
                .with_date(Utc::now().naive_local().date())
                .with_ean13(upc),
        )
    }
}

pub fn parse_abc_item_files(
    item_path: &str,
    posted_path: &str,
) -> Result<HashMap<String, AbcProduct>, csv::Error> {
    let mut item_data = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(item_path)?;
    let mut posted_data = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_path(posted_path)?;

    let mut i = 0;
    let mut products = HashMap::new();
    while let Some(row) = item_data.records().next() {
        i += 1;
        let row = row?;
        let sku = row
            .get(0)
            .ok_or(csv::Error::custom(format!(
                "Cannot deserialize sku in row {}",
                i
            )))?
            .to_string();
        let desc = row
            .get(1)
            .ok_or(csv::Error::custom(format!(
                "Cannot deserialize desc in row {}",
                i
            )))?
            .to_string();
        let upc_str: String = row
            .get(43)
            .ok_or(csv::Error::custom(format!(
                "Cannot fetch upcs in row {}",
                i
            )))?
            .chars()
            .filter(|c| c.is_digit(10) || *c == ',')
            .collect();
        let upcs: Vec<Ean13> = upc_str
            .split(",")
            .filter_map(|s| {
                if s.len() == 11 {
                    // Some ABC UPCs leave out the check digit, so make one up and let [`Ean13::from_str_nonstrict`] fix it
                    Ean13::from_str_nonstrict(&format!("{}0", s)).ok()
                } else if s.len() < 11 {
                    // Anything less than 11 characters long is probably a dead upc
                    None
                } else {
                    // Anything 12 characters and up has a chance of being a good upc
                    Ean13::from_str_nonstrict(s).ok()
                }
            })
            .collect();
        let list = row.get(6).ok_or(csv::Error::custom(format!(
            "Cannot fetch list price from row {}",
            i
        )))?;
        let list = price_from_str(list).or(Err(csv::Error::custom(format!(
            "Cannot parse a price in cents for list in row {}",
            i
        ))))?;
        let mut alt_skus = Vec::new();
        for i in 40..43 {
            if let Some(sku) = row.get(i) {
                alt_skus.push(sku.to_string());
            }
        }

        products.insert(
            sku.clone(),
            AbcProduct {
                sku,
                alt_skus,
                desc,
                upcs,
                list,
                stock: 0.0,
                last_sold: None,
            },
        );
    }

    let mut i = 0;
    while let Some(row) = posted_data.records().next() {
        i += 1;
        let row = row?;
        let sku = row
            .get(0)
            .ok_or(csv::Error::custom(format!(
                "Cannot deserialize sku in row {} of posted items",
                i
            )))?
            .to_string();
        let stock_str = row
            .get(19)
            .ok_or(csv::Error::custom(format!(
                "Cannot deserialize stock in row {} of posted items",
                i
            )))?
            .to_string();
        let stock: f64 = stock_str.parse().or(Err(csv::Error::custom(format!(
            "Cannot parse f64 from stock_str in row {} of posted items",
            i
        ))))?;
        let last_sold_str: String = row
            .get(1)
            .ok_or(csv::Error::custom(format!(
                "Cannot deserialize last_sold in row {} of posted items",
                i
            )))?
            .to_string();
        let last_sold = chrono::NaiveDate::parse_from_str(&last_sold_str, "%Y-%m-%d").ok();
        let mut existing_record = products
            .get(&sku)
            .ok_or(csv::Error::custom(format!(
                "Cannot find existing product for item with sku {} in row {} of posted_data",
                &sku, i
            )))?
            .clone();
        existing_record.stock = stock;
        existing_record.sku = existing_record.sku.to_uppercase();
        existing_record.last_sold = last_sold;
        products.insert(sku, existing_record);
    }
    Ok(products)
}
