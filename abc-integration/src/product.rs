use generator::Label;
use rust_decimal::Decimal;
use serde::{Deserialize, ser::Error};
use std::{
    collections::HashMap,
    fs::{self, File},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

use inventory_utils::Ean13;

fn price_from_str(price_str: &str) -> Result<Decimal, rust_decimal::Error> {
    let price_str: String = price_str
        .chars()
        .filter(|c| c.is_digit(10) || c == &'.')
        .collect();
    Decimal::from_str(&price_str)
}

#[derive(Deserialize)]
struct FetchImgResp {
    items: Vec<FetchImgItem>,
}

#[derive(Deserialize)]
struct FetchImgItem {
    images: Vec<String>,
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

fn fetch_img(ean: Ean13, cache: &mut HashMap<Ean13, String>) -> Option<String> {
    if let Some(url) = cache.get(&ean) {
        return Some(url.to_string());
    }

    sleep(Duration::from_millis(100)); // Add a rate limit for better citizenship
    let fetch_url = format!(
        "https://api.upcitemdb.com/prod/trial/lookup?upc={}",
        ean.to_upca_string()
    );
    let raw_response = reqwest::blocking::get(fetch_url).ok()?;
    if !raw_response.status().is_success() {
        return None;
    }
    let text = raw_response.text();
    let resp: FetchImgResp = serde_json::from_str(&text.ok()?).ok()?;
    let item = resp.items.get(0)?;
    let img = item.images.get(0)?;
    cache.insert(ean, img.to_string());
    Some(img.to_string())
}

#[derive(Debug, Clone)]
pub struct AbcProduct {
    sku: String,
    alt_skus: Vec<String>,
    desc: String,
    upcs: Vec<Ean13>,
    list: Decimal,
    cost: Decimal,
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

    pub fn cost(&self) -> Decimal {
        self.cost
    }

    pub fn stock(&self) -> f64 {
        self.stock
    }

    pub fn last_sold(&self) -> Option<chrono::NaiveDate> {
        self.last_sold
    }

    pub fn set_list(&mut self, new_list: Decimal) {
        self.list = new_list;
    }

    pub fn label(&self, cache: &mut HashMap<Ean13, String>) -> Option<Label> {
        let upc = self.upcs().get(0)?.clone();
        let mut label = match fetch_img(upc.clone(), cache) {
            Some(i) => Label::new().with_img(&i),
            None => Label::new(),
        };
        for sku in &self.alt_skus {
            label.push_alt(sku);
        }
        label
            .with_sku(&self.sku())
            .with_desc(&self.desc())
            .with_price(self.list())
            .with_ean13(upc)
            .build()
    }
}

pub struct AbcProductBuilder {
    sku: Option<String>,
    alt_skus: Vec<String>,
    desc: Option<String>,
    upcs: Vec<Ean13>,
    list: Option<Decimal>,
    cost: Option<Decimal>,
    stock: Option<f64>,
    last_sold: Option<chrono::NaiveDate>,
}

impl AbcProductBuilder {
    pub fn new() -> Self {
        AbcProductBuilder {
            sku: None,
            alt_skus: Vec::new(),
            desc: None,
            upcs: Vec::new(),
            list: None,
            cost: None,
            stock: None,
            last_sold: None,
        }
    }

    pub fn with_sku(self, sku: &str) -> Self {
        AbcProductBuilder {
            sku: Some(sku.to_string()),
            ..self
        }
    }

    pub fn with_desc(self, desc: &str) -> Self {
        AbcProductBuilder {
            desc: Some(desc.to_string()),
            ..self
        }
    }

    pub fn with_upcs(self, upcs: Vec<Ean13>) -> Self {
        AbcProductBuilder { upcs, ..self }
    }

    pub fn add_upc(self, upc: Ean13) -> Self {
        let mut new_upcs = self.upcs.to_vec();
        new_upcs.push(upc);
        AbcProductBuilder {
            upcs: new_upcs,
            ..self
        }
    }

    pub fn with_list(self, list: Decimal) -> Self {
        AbcProductBuilder {
            list: Some(list),
            ..self
        }
    }

    pub fn with_cost(self, cost: Decimal) -> Self {
        AbcProductBuilder {
            cost: Some(cost),
            ..self
        }
    }

    pub fn with_stock(self, stock: f64) -> Self {
        AbcProductBuilder {
            stock: Some(stock),
            ..self
        }
    }

    pub fn push_alt(&mut self, sku: &str) {
        self.alt_skus.push(sku.to_string());
    }

    pub fn build(self) -> Option<AbcProduct> {
        Some(AbcProduct {
            sku: self.sku.clone()?,
            alt_skus: self.alt_skus,
            desc: self.desc.clone()?,
            upcs: self.upcs,
            list: self.list?,
            cost: self.cost?,
            stock: self.stock?,
            last_sold: self.last_sold,
        })
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
        let cost = row.get(8).ok_or(csv::Error::custom(format!(
            "Cannot fetch cost from row {}",
            i
        )))?;
        let cost = price_from_str(cost).or(Err(csv::Error::custom(format!(
            "Cannot parse a price in cents for cost in row {}",
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
                cost,
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
