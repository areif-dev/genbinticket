pub mod template_env;

use inventory_utils::Ean13;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Label {
    price: Option<Decimal>,
    sku: Option<String>,
    desc: Option<String>,
    ean13: Option<Ean13>,
    alt_skus: Vec<String>,
    img: Option<String>,
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

    pub fn push_alt(&mut self, sku: &str) {
        self.alt_skus.push(sku.to_string());
    }
}
