mod app;

use std::{
    env,
    error::Error,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{extract::State, response::Html, routing::get, Router};
use leptos::prelude::*;

use crate::app::App;

#[derive(Clone, Debug)]
struct AppState {
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blog001=info,axum=info".into()),
        )
        .init();

    let state = Arc::new(AppState {
        database_url: env::var("DATABASE_URL").ok(),
    });
    let bind_addr = read_bind_addr()?;

    let router = Router::new()
        .route("/", get(index))
        .route("/healthz", get(healthz))
        .with_state(state);

    tracing::info!(%bind_addr, "starting blog001 server");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

fn read_bind_addr() -> Result<SocketAddr, Box<dyn Error + Send + Sync>> {
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port_raw = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let ip = IpAddr::from_str(&host).map_err(|err| format!("invalid HOST `{host}`: {err}"))?;
    let port = port_raw
        .parse::<u16>()
        .map_err(|err| format!("invalid PORT `{port_raw}`: {err}"))?;

    Ok(SocketAddr::from((ip, port)))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let database_configured = state.database_url.is_some();

    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <App database_configured=database_configured />
        }
    });

    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>blog001</title></head><body>{app_html}</body></html>"
    ))
}
