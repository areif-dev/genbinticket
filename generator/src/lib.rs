pub mod template_env;

use chrono::NaiveDate;
use inventory_utils::Ean13;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CustomNaiveDate(chrono::NaiveDate);

impl CustomNaiveDate {
    pub fn parse_from_str(s: &str, fmt: &str) -> Result<CustomNaiveDate, chrono::ParseError> {
        Ok(CustomNaiveDate(NaiveDate::parse_from_str(s, fmt)?))
    }

    pub fn new(d: NaiveDate) -> Self {
        CustomNaiveDate(d)
    }
}

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

    pub fn price(&self) -> Option<Decimal> {
        self.price.clone()
    }

    pub fn sku(&self) -> Option<String> {
        self.sku.clone()
    }

    pub fn desc(&self) -> Option<String> {
        self.desc.clone()
    }

    pub fn ean13(&self) -> Option<Ean13> {
        self.ean13.clone()
    }

    pub fn alt_skus(&self) -> Vec<String> {
        self.alt_skus.clone()
    }

    pub fn img(&self) -> Option<String> {
        self.img.clone()
    }

    pub fn date(&self) -> Option<CustomNaiveDate> {
        self.date.clone()
    }

    pub fn qty(&self) -> Option<u32> {
        self.qty
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

    pub fn with_date(self, date: NaiveDate) -> Self {
        Label {
            date: Some(CustomNaiveDate(date)),
            ..self
        }
    }

    pub fn push_alt(&mut self, sku: &str) {
        self.alt_skus.push(sku.to_string());
    }
}
