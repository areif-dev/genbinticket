use clap::ValueEnum;
use std::{cmp::Ordering, io, sync::Arc};
use tower_http::services::ServeDir;

use axum::{
    Router,
    extract::{Query, State},
    response::Html,
    routing::get,
};
use generator::{Label, template_env::TemplateEnvironment};
use reqwest::StatusCode;
use serde::Deserialize;

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

#[derive(Deserialize)]
struct RootQuery {
    sort: Option<SortOption>,
}

async fn root(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RootQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut labels = state.labels.clone();
    sort_labels(&mut labels, query.sort.unwrap_or(SortOption::Upc));
    Ok(Html::from(
        render_template(&state.template_env, &labels)
            .or_else(|e| Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))))?,
    ))
}

pub async fn start_server(labels: Vec<Label>, debug: bool) -> Result<(), io::Error> {
    let env =
        template_env::setup_env().or_else(|e| Err(io::Error::new(io::ErrorKind::Other, e)))?;
    let shared_state = Arc::new(AppState {
        template_env: env,
        labels,
    });

    let app = Router::new()
        .route("/", get(root))
        .nest_service("/static", ServeDir::new("./static"))
        .with_state(shared_state);
    let (address, port) = if debug {
        ("0.0.0.0", 5000)
    } else {
        ("localhost", 0)
    };
    let listener = tokio::net::TcpListener::bind((address, port)).await?;
    let addr = listener.local_addr()?;
    eprintln!("Starting server on address {}", addr);
    webbrowser::open(&format!("http://localhost:{}", addr.port()))?;
    axum::serve(listener, app).await?;
    Ok(())
}
