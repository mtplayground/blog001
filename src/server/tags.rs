use std::{collections::BTreeSet, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::AppState;

const SESSION_COOKIE_NAME: &str = "blog001_session";

#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssociateTagsRequest {
    pub tag_ids: Option<Vec<i64>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, FromRow)]
struct TagRow {
    id: i64,
    name: String,
    slug: String,
    created_at: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_tags).post(create_tag))
        .route("/{id}", delete(delete_tag))
        .route("/post/{post_id}", get(get_tags_for_post).put(associate_tags_with_post))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    match sqlx::query_as::<_, TagRow>(
        r#"
        SELECT id, name, slug, created_at
        FROM tags
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(rows) => {
            let tags = rows.into_iter().map(map_tag).collect::<Vec<_>>();
            (StatusCode::OK, Json(tags)).into_response()
        }
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to list tags"),
    }
}

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateTagRequest>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    let name = payload.name.trim();
    if name.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "tag name is required");
    }

    let slug = payload
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| slugify(name));

    match sqlx::query_as::<_, TagRow>(
        r#"
        INSERT INTO tags (name, slug)
        VALUES (?1, ?2)
        RETURNING id, name, slug, created_at
        "#,
    )
    .bind(name)
    .bind(slug)
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(row) => (StatusCode::CREATED, Json(map_tag(row))).into_response(),
        Err(err) => {
            if is_unique_violation(&err) {
                error_response(StatusCode::CONFLICT, "tag already exists")
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to create tag")
            }
        }
    }
}

pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    match sqlx::query("DELETE FROM tags WHERE id = ?1")
        .bind(id)
        .execute(&state.db_pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT.into_response(),
        Ok(_) => error_response(StatusCode::NOT_FOUND, "tag not found"),
        Err(_) => error_response(StatusCode::INTERNAL_SERVER_ERROR, "failed to delete tag"),
    }
}

pub async fn get_tags_for_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(post_id): Path<i64>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    if !post_exists(&state, post_id).await {
        return error_response(StatusCode::NOT_FOUND, "post not found");
    }

    match fetch_tags_for_post(&state, post_id).await {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to fetch tags for post",
        ),
    }
}

pub async fn associate_tags_with_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(post_id): Path<i64>,
    Json(payload): Json<AssociateTagsRequest>,
) -> impl IntoResponse {
    if let Err(response) = require_auth(&state, &headers) {
        return response;
    }

    if !post_exists(&state, post_id).await {
        return error_response(StatusCode::NOT_FOUND, "post not found");
    }

    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to start tag association transaction",
            )
        }
    };

    if sqlx::query("DELETE FROM post_tags WHERE post_id = ?1")
        .bind(post_id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to clear existing post tags",
        );
    }

    let mut final_tag_ids = BTreeSet::new();

    if let Some(tag_ids) = payload.tag_ids {
        for id in tag_ids {
            final_tag_ids.insert(id);
        }
    }

    if let Some(tags) = payload.tags {
        for raw_name in tags {
            let name = raw_name.trim();
            if name.is_empty() {
                continue;
            }

            let slug = slugify(name);
            let id_result = sqlx::query_scalar::<_, i64>(
                r#"
                INSERT INTO tags (name, slug)
                VALUES (?1, ?2)
                ON CONFLICT(slug) DO UPDATE SET name = excluded.name
                RETURNING id
                "#,
            )
            .bind(name)
            .bind(slug)
            .fetch_one(&mut *tx)
            .await;

            let id = match id_result {
                Ok(id) => id,
                Err(_) => {
                    return error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to upsert tag while associating",
                    )
                }
            };

            final_tag_ids.insert(id);
        }
    }

    for tag_id in final_tag_ids {
        if sqlx::query(
            r#"
            INSERT OR IGNORE INTO post_tags (post_id, tag_id)
            VALUES (?1, ?2)
            "#,
        )
        .bind(post_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await
        .is_err()
        {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to associate tags with post",
            );
        }
    }

    if tx.commit().await.is_err() {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to commit tag association",
        );
    }

    match fetch_tags_for_post(&state, post_id).await {
        Ok(tags) => (StatusCode::OK, Json(tags)).into_response(),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to fetch updated tags for post",
        ),
    }
}

async fn fetch_tags_for_post(state: &Arc<AppState>, post_id: i64) -> Result<Vec<TagResponse>, sqlx::Error> {
    let rows = sqlx::query_as::<_, TagRow>(
        r#"
        SELECT t.id, t.name, t.slug, t.created_at
        FROM tags t
        INNER JOIN post_tags pt ON pt.tag_id = t.id
        WHERE pt.post_id = ?1
        ORDER BY t.name ASC
        "#,
    )
    .bind(post_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows.into_iter().map(map_tag).collect())
}

async fn post_exists(state: &Arc<AppState>, post_id: i64) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts WHERE id = ?1")
        .bind(post_id)
        .fetch_one(&state.db_pool)
        .await
        .map(|count| count > 0)
        .unwrap_or(false)
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

fn map_tag(row: TagRow) -> TagResponse {
    TagResponse {
        id: row.id,
        name: row.name,
        slug: row.slug,
        created_at: row.created_at,
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if (ch.is_ascii_whitespace() || ch == '-' || ch == '_') && !out.ends_with('-') {
            out.push('-');
        }
    }

    out.trim_matches('-').to_string()
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
