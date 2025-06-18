use std::str::FromStr;

use barcoders::{
    generators::image::{Color, Image, Rotation},
    sym::ean13::{EAN13 as barcodersEan13, UPCA},
};
use base64::{Engine, engine::general_purpose};
use inventory_utils::Ean13;
use rust_decimal::Decimal;

pub fn pretty_price(val: &str) -> String {
    let decimal = Decimal::from_str(val).unwrap_or(Decimal::new(999999, 2));
    format!("${:.2}", decimal)
}

pub fn encode_barcode(code: &str) -> String {
    let code = match Ean13::from_str(code) {
        Ok(c) => c,
        Err(_) => {
            return "Error while encoding".to_string();
        }
    };
    let barcode = if code.is_upca() {
        match UPCA::new(&code.to_string()[1..]) {
            Ok(b) => b,
            Err(_) => {
                return "Error while encoding".to_string();
            }
        }
    } else {
        match barcodersEan13::new(code.to_string()) {
            Ok(b) => b,
            Err(_) => {
                return "Error while encoding".to_string();
            }
        }
    };
    let png = Image::PNG {
        height: 80,
        xdim: 2,
        rotation: Rotation::Zero,
        foreground: Color::black(),
        background: Color::white(),
    };
    let encoded = barcode.encode();
    let bytes = match png.generate(&encoded) {
        Ok(b) => b,
        Err(_) => {
            return "Error while encoding".to_string();
        }
    };

    let b64 = general_purpose::STANDARD.encode(bytes);
    format!("data:image/png;base64,{}", b64)
}

pub fn format_ean13(code: &str) -> String {
    let code = match Ean13::from_str(code) {
        Ok(c) => c,
        Err(_) => {
            return "Encountered invalid EAN13".to_string();
        }
    };

    let s = code.to_string();
    format!("{}-{}", &s[0..10], &s[10..13])
}
