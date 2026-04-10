use axum::{
    extract::{Json, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use crate::AppState;

const LOGIN_USERNAME: &str = "admin";
const LOGIN_PASSWORD: &str = "changeme";
const SESSION_COOKIE_NAME: &str = "blog001_session";
const SESSION_MAX_AGE_SECONDS: u64 = 60 * 60 * 24;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub username: Option<String>,
}

pub async fn login(
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    if payload.username != LOGIN_USERNAME || payload.password != LOGIN_PASSWORD {
        return (
            StatusCode::UNAUTHORIZED,
            Json(AuthStatusResponse {
                authenticated: false,
                username: None,
            }),
        )
            .into_response();
    }

    let Some(session_id) = state
        .session_store
        .create_session(payload.username.clone())
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthStatusResponse {
                authenticated: false,
                username: None,
            }),
        )
            .into_response();
    };

    let cookie_value = build_set_cookie_value(&session_id);
    let mut response = (
        StatusCode::OK,
        Json(AuthStatusResponse {
            authenticated: true,
            username: Some(payload.username),
        }),
    )
        .into_response();

    if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, header_value);
    } else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthStatusResponse {
                authenticated: false,
                username: None,
            }),
        )
            .into_response();
    }

    response
}

pub async fn validate_session(
    State(state): State<std::sync::Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let username = headers
        .get(header::COOKIE)
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(extract_session_cookie)
        .and_then(|session_id| state.session_store.validate_session(&session_id));

    (
        StatusCode::OK,
        Json(AuthStatusResponse {
            authenticated: username.is_some(),
            username,
        }),
    )
}

pub async fn logout(
    State(state): State<std::sync::Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(session_id) = headers
        .get(header::COOKIE)
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(extract_session_cookie)
    {
        let _ = state.session_store.invalidate_session(&session_id);
    }

    let clear_cookie = format!(
        "{SESSION_COOKIE_NAME}=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax"
    );

    let mut response = (
        StatusCode::OK,
        Json(AuthStatusResponse {
            authenticated: false,
            username: None,
        }),
    )
        .into_response();

    if let Ok(header_value) = HeaderValue::from_str(&clear_cookie) {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, header_value);
    }

    response
}

fn build_set_cookie_value(session_id: &str) -> String {
    format!(
        "{SESSION_COOKIE_NAME}={session_id}; HttpOnly; Path=/; Max-Age={SESSION_MAX_AGE_SECONDS}; SameSite=Lax"
    )
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
