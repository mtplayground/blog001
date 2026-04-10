mod app;
mod auth;
mod components;
mod db;
mod middleware;
mod markdown;
mod pages;
mod server;
mod session;

use std::{
    collections::BTreeSet,
    collections::HashMap,
    env,
    error::Error,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use leptos::view;
use sqlx::{FromRow, SqlitePool};

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
    let admin_router = Router::new()
        .route("/", get(admin_index))
        .route("/posts", get(admin_posts_index))
        .route("/posts/new", get(admin_new_post))
        .route("/posts/{id}/edit", get(admin_edit_post))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::require_admin_auth,
        ));

    let router = Router::new()
        .route("/", get(index))
        .route("/posts/{slug}", get(post_detail))
        .route("/login", get(login_page))
        .route("/healthz", get(healthz))
        .route("/auth/login", post(auth::login))
        .route("/auth/session", get(auth::validate_session))
        .route("/auth/logout", post(auth::logout))
        .nest("/server/posts", server::posts::router())
        .nest("/server/tags", server::tags::router())
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

async fn index(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    #[derive(Debug, FromRow)]
    struct HomeRow {
        post_id: i64,
        title: String,
        slug: String,
        body: String,
        created_at: String,
        tag_name: Option<String>,
        tag_slug: Option<String>,
    }

    let rows = sqlx::query_as::<_, HomeRow>(
        r#"
        SELECT
            p.id as post_id,
            p.title,
            p.slug,
            p.body,
            p.created_at,
            t.name as tag_name,
            t.slug as tag_slug
        FROM posts p
        LEFT JOIN post_tags pt ON pt.post_id = p.id
        LEFT JOIN tags t ON t.id = pt.tag_id
        WHERE p.is_published = 1
        ORDER BY p.created_at DESC, p.id DESC, t.name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut posts = Vec::<pages::home::HomePostSummary>::new();
    let mut post_index = HashMap::<i64, usize>::new();
    let mut tag_counts = HashMap::<(String, String), usize>::new();

    for row in rows {
        let idx = if let Some(existing_idx) = post_index.get(&row.post_id) {
            *existing_idx
        } else {
            let next_idx = posts.len();
            posts.push(pages::home::HomePostSummary {
                title: row.title.clone(),
                slug: row.slug.clone(),
                excerpt: excerpt(&row.body, 180),
                published_at: row.created_at.clone(),
                tag_names: Vec::new(),
                tag_slugs: Vec::new(),
            });
            post_index.insert(row.post_id, next_idx);
            next_idx
        };

        if let (Some(tag_name), Some(tag_slug)) = (row.tag_name, row.tag_slug) {
            if !posts[idx].tag_slugs.contains(&tag_slug) {
                posts[idx].tag_names.push(tag_name.clone());
                posts[idx].tag_slugs.push(tag_slug.clone());
                *tag_counts.entry((tag_name, tag_slug)).or_insert(0) += 1;
            }
        }
    }

    let mut tags = tag_counts
        .into_iter()
        .map(|((name, slug), count)| components::tag_filter::TagFilterItem {
            name,
            slug,
            count,
        })
        .collect::<Vec<_>>();
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <pages::home::HomePage posts=posts.clone() tags=tags.clone() />
        }
    });

    Ok(Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    )))
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
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<middleware::auth::AuthUser>,
) -> Result<Html<String>, StatusCode> {
    #[derive(Debug, FromRow)]
    struct DashboardRow {
        title: String,
        slug: String,
        is_published: bool,
        updated_at: String,
    }

    let post_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts")
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let recent_posts = sqlx::query_as::<_, DashboardRow>(
        r#"
        SELECT title, slug, is_published, updated_at
        FROM posts
        ORDER BY updated_at DESC
        LIMIT 5
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summaries = recent_posts
        .into_iter()
        .map(|row| pages::admin::dashboard::RecentPostSummary {
            title: row.title,
            slug: row.slug,
            is_published: row.is_published,
            updated_at: row.updated_at,
        })
        .collect::<Vec<_>>();

    let username = user.username;
    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <components::admin_layout::AdminLayout username=username.clone()>
                <pages::admin::dashboard::AdminDashboard post_count=post_count recent_posts=summaries.clone() />
            </components::admin_layout::AdminLayout>
        }
    });

    Ok(Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Admin | blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    )))
}

async fn admin_new_post(Extension(user): Extension<middleware::auth::AuthUser>) -> Html<String> {
    render_editor_page(user.username, None)
}

async fn admin_edit_post(
    Path(id): Path<i64>,
    Extension(user): Extension<middleware::auth::AuthUser>,
) -> Html<String> {
    render_editor_page(user.username, Some(id))
}

fn render_editor_page(username: String, post_id: Option<i64>) -> Html<String> {
    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <components::admin_layout::AdminLayout username=username.clone()>
                <pages::admin::editor::PostEditorPage post_id=post_id />
            </components::admin_layout::AdminLayout>
        }
    });

    Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Post Editor | blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    ))
}

async fn admin_posts_index(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<middleware::auth::AuthUser>,
) -> Result<Html<String>, StatusCode> {
    #[derive(Debug, FromRow)]
    struct PostListRow {
        id: i64,
        title: String,
        slug: String,
        is_published: bool,
        updated_at: String,
    }

    let rows = sqlx::query_as::<_, PostListRow>(
        r#"
        SELECT id, title, slug, is_published, updated_at
        FROM posts
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let posts = rows
        .into_iter()
        .map(|row| pages::admin::posts::AdminPostListItem {
            id: row.id,
            title: row.title,
            slug: row.slug,
            is_published: row.is_published,
            updated_at: row.updated_at,
        })
        .collect::<Vec<_>>();

    let username = user.username;
    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <components::admin_layout::AdminLayout username=username.clone()>
                <pages::admin::posts::AdminPostsPage posts=posts.clone() />
            </components::admin_layout::AdminLayout>
        }
    });

    Ok(Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Manage Posts | blog001</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>"
    )))
}

async fn post_detail(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    #[derive(Debug, FromRow)]
    struct PostDetailRow {
        title: String,
        body: String,
        created_at: String,
        tag_name: Option<String>,
        tag_slug: Option<String>,
    }

    let rows = sqlx::query_as::<_, PostDetailRow>(
        r#"
        SELECT
            p.title,
            p.body,
            p.created_at,
            t.name as tag_name,
            t.slug as tag_slug
        FROM posts p
        LEFT JOIN post_tags pt ON pt.post_id = p.id
        LEFT JOIN tags t ON t.id = pt.tag_id
        WHERE p.slug = ?1 AND p.is_published = 1
        ORDER BY t.name ASC
        "#,
    )
    .bind(slug)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let first = rows.first().ok_or(StatusCode::NOT_FOUND)?;

    let mut tags = Vec::<pages::post::PostTag>::new();
    let mut seen = BTreeSet::<String>::new();
    for row in &rows {
        if let (Some(name), Some(tag_slug)) = (&row.tag_name, &row.tag_slug) {
            if seen.insert(tag_slug.clone()) {
                tags.push(pages::post::PostTag {
                    name: name.clone(),
                    slug: tag_slug.clone(),
                });
            }
        }
    }

    let content_html = markdown::render_markdown(&first.body);
    let title = first.title.clone();
    let published_at = first.created_at.clone();

    let app_html = leptos::ssr::render_to_string(move || {
        view! {
            <pages::post::PostPage
                title=title.clone()
                published_at=published_at.clone()
                tags=tags.clone()
                content_html=content_html.clone()
            />
        }
    });

    Ok(Html(format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title><link rel=\"stylesheet\" href=\"/style/main.css\"></head><body>{app_html}</body></html>",
        first.title
    )))
}

fn excerpt(content: &str, max_chars: usize) -> String {
    let trimmed = content.trim();
    let mut chars = trimmed.chars();
    let collected = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{collected}...")
    } else {
        collected
    }
}
