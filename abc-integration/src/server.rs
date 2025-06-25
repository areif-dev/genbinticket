use std::{io, sync::Arc};

use axum::{Json, Router, extract::State, response::Html, routing::get};

use crate::template_env::{self, render_template};

pub struct AppState {
    template_env: TemplateEnvironment<'static>,
    labels: Vec<Label>,
}

#[derive(Clone, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOption {
    Upc,
    Sku,
    Date,
    Preserve,
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

pub async fn start_server() -> Result<(), io::Error> {
    let env =
        template_env::setup_env().or_else(|e| Err(io::Error::new(io::ErrorKind::Other, e)))?;
    let shared_state = Arc::new(env);
    let app = Router::new().route("/", get(root)).with_state(shared_state);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    axum::serve(listener, app).await?;
    Ok(())
}
