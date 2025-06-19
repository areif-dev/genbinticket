pub mod template_env;

use inventory_utils::Ean13;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Label {
    price: Decimal,
    sku: String,
    desc: String,
    ean13: Ean13,
    alt_skus: Vec<String>,
    img: Option<String>,
}

impl Label {
    pub fn new() -> LabelBuilder {
        LabelBuilder::new()
    }

    pub fn push_alt(&mut self, sku: &str) {
        self.alt_skus.push(sku.to_string());
    }
}

pub struct LabelBuilder {
    price: Option<Decimal>,
    sku: Option<String>,
    desc: Option<String>,
    ean13: Option<Ean13>,
    alt_skus: Vec<String>,
    img: Option<String>,
}

impl LabelBuilder {
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

    pub fn build(self) -> Option<Label> {
        Some(Label {
            price: self.price?,
            sku: self.sku?,
            desc: self.desc?,
            ean13: self.ean13?,
            alt_skus: self.alt_skus,
            img: self.img,
        })
    }

    pub fn with_price(self, price: Decimal) -> Self {
        LabelBuilder {
            price: Some(price),
            ..self
        }
    }

    pub fn with_sku(self, sku: &str) -> Self {
        LabelBuilder {
            sku: Some(sku.to_string()),
            ..self
        }
    }

    pub fn with_desc(self, desc: &str) -> Self {
        LabelBuilder {
            desc: Some(desc.to_string()),
            ..self
        }
    }

    pub fn with_img(self, img: &str) -> Self {
        LabelBuilder {
            img: Some(img.to_string()),
            ..self
        }
    }

    pub fn with_ean13(self, ean13: Ean13) -> Self {
        LabelBuilder {
            ean13: Some(ean13),
            ..self
        }
    }

    pub fn push_alt(&mut self, sku: &str) {
        self.alt_skus.push(sku.to_string());
    }
}
