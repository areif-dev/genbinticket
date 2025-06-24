use std::{io, sync::Arc};

use axum::{Json, Router, extract::State, response::Html, routing::get};

use crate::template_env::{self, render_template};

async fn root(State(Arc(env)): State<Arc<minijinja::Environment<'static>>>) -> Html<String> {}

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
