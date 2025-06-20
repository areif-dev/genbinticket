pub mod template_env;

use chrono::NaiveDate;
use inventory_utils::Ean13;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct CustomNaiveDate(chrono::NaiveDate);

impl<'de> Deserialize<'de> for CustomNaiveDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(CustomNaiveDate(
            NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?,
        ))
    }
}

impl Serialize for CustomNaiveDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.0.format("%Y-%m-%d").to_string();
        s.serialize(serializer)
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Label {
    price: Option<Decimal>,
    sku: Option<String>,
    desc: Option<String>,
    ean13: Option<Ean13>,
    alt_skus: Vec<String>,
    img: Option<String>,
    date: Option<CustomNaiveDate>,
    qty: Option<u32>,
}

impl Label {
    pub fn new() -> Self {
        Self {
            price: None,
            sku: None,
            desc: None,
            ean13: None,
            alt_skus: Vec::new(),
            img: None,
            date: None,
            qty: None,
        }
    }

    pub fn with_qty(self, qty: u32) -> Self {
        Label {
            qty: Some(qty),
            ..self
        }
    }

    pub fn with_price(self, price: Decimal) -> Self {
        Label {
            price: Some(price),
            ..self
        }
    }

    pub fn with_sku(self, sku: &str) -> Self {
        Label {
            sku: Some(sku.to_string()),
            ..self
        }
    }

    pub fn with_desc(self, desc: &str) -> Self {
        Label {
            desc: Some(desc.to_string()),
            ..self
        }
    }

    pub fn with_img(self, img: &str) -> Self {
        Label {
            img: Some(img.to_string()),
            ..self
        }
    }

    pub fn with_ean13(self, ean13: Ean13) -> Self {
        Label {
            ean13: Some(ean13),
            ..self
        }
    }

    pub fn with_date(self, date: CustomNaiveDate) -> Self {
        Label {
            date: Some(date),
            ..self
        }
    }

    pub fn push_alt(&mut self, sku: &str) {
        self.alt_skus.push(sku.to_string());
    }
}
