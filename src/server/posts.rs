use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::AppState;

const SESSION_COOKIE_NAME: &str = "blog001_session";

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub slug: String,
    pub title: String,
    pub markdown: String,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub slug: Option<String>,
    pub title: Option<String>,
    pub markdown: Option<String>,
    pub is_published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TogglePublishRequest {
    pub is_published: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub include_drafts: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub markdown: String,
    pub is_published: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, FromRow)]
struct PostRow {
    id: i64,
    slug: String,
    title: String,
    body: String,
    is_published: bool,
    created_at: String,
    updated_at: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_post).get(list_posts))
        .route("/{id}", get(get_post).put(update_post).delete(delete_post))
        .route("/{id}/publish", put(set_publish_status))
}

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreatePostRequest>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    if payload.slug.trim().is_empty() || payload.title.trim().is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "slug and title are required");
    }

    if payload.markdown.trim().is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "markdown is required");
    }

    let query = sqlx::query_as::<_, PostRow>(
        r#"
        INSERT INTO posts (slug, title, body, is_published)
        VALUES (?1, ?2, ?3, ?4)
        RETURNING id, slug, title, body, is_published, created_at, updated_at
        "#,
    )
    .bind(payload.slug.trim())
    .bind(payload.title.trim())
    .bind(payload.markdown)
    .bind(payload.is_published.unwrap_or(false));

    match query.fetch_one(&state.db_pool).await {
        Ok(row) => (StatusCode::CREATED, Json(map_post(row))).into_response(),
        Err(err) => {
            if is_unique_violation(&err) {
                error_response(StatusCode::CONFLICT, "slug already exists")
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to create post")
            }
        }
    }
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ListPostsQuery>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    let include_drafts = query.include_drafts.unwrap_or(true);

    let rows = if include_drafts {
        sqlx::query_as::<_, PostRow>(
            r#"
            SELECT id, slug, title, body, is_published, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await
    } else {
        sqlx::query_as::<_, PostRow>(
            r#"
            SELECT id, slug, title, body, is_published, created_at, updated_at
            FROM posts
            WHERE is_published = 1
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await
    };

    match rows {
        Ok(rows) => {
            let posts: Vec<PostResponse> = rows.into_iter().map(map_post).collect();
            (StatusCode::OK, Json(posts)).into_response()
        }
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to list posts"),
    }
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    match sqlx::query_as::<_, PostRow>(
        r#"
        SELECT id, slug, title, body, is_published, created_at, updated_at
        FROM posts
        WHERE id = ?1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(row)) => (StatusCode::OK, Json(map_post(row))).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "post not found"),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to read post"),
    }
}

pub async fn update_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    let slug = payload.slug.as_deref().map(str::trim).map(str::to_owned);
    let title = payload.title.as_deref().map(str::trim).map(str::to_owned);

    if slug.as_deref() == Some("") || title.as_deref() == Some("") {
        return error_response(StatusCode::BAD_REQUEST, "slug and title cannot be empty");
    }

    let query = sqlx::query_as::<_, PostRow>(
        r#"
        UPDATE posts
        SET
            slug = COALESCE(?1, slug),
            title = COALESCE(?2, title),
            body = COALESCE(?3, body),
            is_published = COALESCE(?4, is_published),
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ?5
        RETURNING id, slug, title, body, is_published, created_at, updated_at
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(payload.markdown)
    .bind(payload.is_published)
    .bind(id);

    match query.fetch_optional(&state.db_pool).await {
        Ok(Some(row)) => (StatusCode::OK, Json(map_post(row))).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "post not found"),
        Err(err) => {
            if is_unique_violation(&err) {
                error_response(StatusCode::CONFLICT, "slug already exists")
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to update post")
            }
        }
    }
}

pub async fn set_publish_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<TogglePublishRequest>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    match sqlx::query_as::<_, PostRow>(
        r#"
        UPDATE posts
        SET
            is_published = ?1,
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
        WHERE id = ?2
        RETURNING id, slug, title, body, is_published, created_at, updated_at
        "#,
    )
    .bind(payload.is_published)
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    {
        Ok(Some(row)) => (StatusCode::OK, Json(map_post(row))).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "post not found"),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to toggle publish status",
        ),
    }
}

pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    match sqlx::query("DELETE FROM posts WHERE id = ?1")
        .bind(id)
        .execute(&state.db_pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => error_response(StatusCode::NOT_FOUND, "post not found"),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to delete post"),
    }
}

fn require_auth(state: &Arc<AppState>, headers: &HeaderMap) -> Result<String, axum::response::Response> {
    let username = headers
        .get(header::COOKIE)
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(extract_session_cookie)
        .and_then(|session_id| state.session_store.validate_session(&session_id));

    username.ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, "authentication required"))
}

fn extract_session_cookie(cookie_header: &str) -> Option<String> {
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|part| {
            let (name, value) = part.split_once('=')?;
            if name == SESSION_COOKIE_NAME {
                Some(value.to_string())
            } else {
                None
            }
        })
}

fn map_post(row: PostRow) -> PostResponse {
    PostResponse {
        id: row.id,
        slug: row.slug,
        title: row.title,
        markdown: row.body,
        is_published: row.is_published,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn error_response(status: StatusCode, message: &'static str) -> axum::response::Response {
    (status, Json(ErrorResponse { error: message.to_string() })).into_response()
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => {
            let message = db_err.message().to_ascii_lowercase();
            message.contains("unique") || message.contains("constraint")
        }
        _ => false,
    }
}
