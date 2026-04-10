use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

use crate::AppState;

const SESSION_COOKIE_NAME: &str = "blog001_session";

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub username: String,
}

pub async fn require_admin_auth(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let username = request
        .headers()
        .get(header::COOKIE)
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(extract_session_cookie)
        .and_then(|session_id| state.session_store.validate_session(&session_id));

    if let Some(username) = username {
        request.extensions_mut().insert(AuthUser { username });
        return next.run(request).await;
    }

    Redirect::to("/login").into_response()
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
