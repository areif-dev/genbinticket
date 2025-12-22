use std::str::FromStr;

use barcoders::{
    generators::image::{Color, Image, Rotation},
    sym::ean13::{EAN13, UPCA},
};
use base64::{Engine, engine::general_purpose};
use gtin::{Gtin, GtinKind};
use minijinja::context;
use rust_decimal::Decimal;

pub use minijinja::Environment as TemplateEnvironment;

use crate::Label;

pub fn pretty_price(val: &str) -> String {
    let decimal = Decimal::from_str(val).unwrap_or(Decimal::new(999999, 2));
    format!("${:.2}", decimal)
}

pub fn encode_barcode(code: &str) -> String {
    let code = match Gtin::new(code) {
        Ok(c) => c,
        Err(_) => {
            return "Error while encoding".to_string();
        }
    };
    let try_barcode = match code.kind() {
        GtinKind::Gtin12 => UPCA::new(&code.to_string()[1..]),
        GtinKind::Gtin13 => EAN13::new(&code.to_string_no_padding()),
        _ => {
            return "Error while encoding".to_string();
        }
    };
    let Ok(barcode) = try_barcode else {
        return "Error while encoding".to_string();
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

pub fn format_gtin(code: &str) -> String {
    let code = match Gtin::new(code) {
        Ok(c) => c,
        Err(_) => {
            return "Encountered invalid GTIN".to_string();
        }
    };

    let full_string = code.to_string_no_padding();
    let last_3 = full_string
        .chars()
        .rev()
        .take(3)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    let the_rest = full_string
        .chars()
        .rev()
        .skip(3)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("{}-{}", &the_rest, &last_3)
}

pub fn setup_env() -> Result<TemplateEnvironment<'static>, minijinja::Error> {
    let mut env = minijinja::Environment::new();
    env.add_filter("pretty_price", pretty_price);
    env.add_filter("encode_barcode", encode_barcode);
    env.add_filter("format_gtin", format_gtin);
    env.set_loader(minijinja::path_loader("templates"));
    Ok(env)
}

pub fn render_template(
    env: &minijinja::Environment,
    labels: &[Label],
) -> Result<String, minijinja::Error> {
    let template = env.get_template("base.html")?;
    let pages: Vec<Vec<Label>> = labels.chunks(30).map(|chunk| chunk.to_vec()).collect();
    template.render(context! {
        pages => pages,
    })
}
