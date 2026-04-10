mod app;
mod auth;
mod components;
mod db;
mod middleware;
mod pages;
mod server;
mod session;

use std::{
    env,
    error::Error,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use leptos::view;
use sqlx::SqlitePool;

use crate::app::App;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) database_url: Option<String>,
    pub(crate) db_pool: SqlitePool,
    pub(crate) session_store: session::SessionStore,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blog001=info,axum=info,sqlx=warn".into()),
        )
        .init();

    let db_pool = db::connect_and_migrate_from_env().await?;

    let state = Arc::new(AppState {
        database_url: env::var("DATABASE_URL").ok(),
        db_pool,
        session_store: session::SessionStore::new(Duration::from_secs(60 * 60 * 24)),
    });
    let bind_addr = read_bind_addr()?;
    let admin_router = Router::new().route("/", get(admin_index)).layer(
        axum::middleware::from_fn_with_state(state.clone(), middleware::auth::require_admin_auth),
    );

    let router = Router::new()
        .route("/", get(index))
        .route("/login", get(login_page))
        .route("/healthz", get(healthz))
        .route("/auth/login", post(auth::login))
        .route("/auth/session", get(auth::validate_session))
        .route("/auth/logout", post(auth::logout))
        .nest("/server/posts", server::posts::router())
        .nest("/admin", admin_router)
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

async fn healthz(State(state): State<Arc<AppState>>) -> Result<&'static str, StatusCode> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
        .map(|_| "ok")
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let database_configured = state.database_url.is_some();

    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <App database_configured=database_configured />
        }
    });

    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    ))
}

async fn login_page() -> Html<String> {
    let app_html = leptos::ssr::render_to_string(|| {
        view! {
            <pages::login::LoginPage />
        }
    });

    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Login | blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    ))
}

async fn admin_index(
    Extension(user): Extension<middleware::auth::AuthUser>,
) -> Html<String> {
    let username = user.username;
    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <components::admin_layout::AdminLayout username=username.clone()>
                <h1 class="text-2xl font-bold tracking-tight text-slate-900 sm:text-3xl">"Admin Dashboard"</h1>
                <p class="text-base text-slate-700">"You are authenticated and can access protected admin routes."</p>
            </components::admin_layout::AdminLayout>
        }
    });

    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Admin | blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    ))
}
