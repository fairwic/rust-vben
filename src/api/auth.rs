use axum::{
    extract::{Json, State},
    http::{header, HeaderMap},
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
    models::auth::LoginRequest,
    response::{error_response, success_response},
    services::auth as auth_service,
    state::AppState,
};

pub async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> Response {
    match auth_service::login(&state, payload).await {
        Ok((cookie, data)) => {
            let mut response = Json(success_response(data)).into_response();
            response.headers_mut().insert(header::SET_COOKIE, cookie);
            response
        }
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn logout(headers: HeaderMap) -> Response {
    let mut response = Json(success_response("")).into_response();
    if auth_service::read_refresh_token(&headers).is_some() {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, auth_service::expired_refresh_cookie());
    }
    response
}

pub async fn refresh(headers: HeaderMap) -> Response {
    match auth_service::refresh(&headers) {
        Ok((cookie, token)) => {
            let mut response = Json(token).into_response();
            response.headers_mut().insert(header::SET_COOKIE, cookie);
            response
        }
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn access_codes(headers: HeaderMap) -> Response {
    match auth_service::authorize(&headers) {
        Some(user) => Json(success_response(auth_service::access_codes_for(
            user.username,
        )))
        .into_response(),
        None => error_response(
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized Exception",
        ),
    }
}

pub async fn user_info(headers: HeaderMap) -> Response {
    match auth_service::authorize(&headers) {
        Some(user) => Json(success_response(auth_service::to_user_info(user))).into_response(),
        None => error_response(
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized Exception",
        ),
    }
}

pub async fn menu_all(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match auth_service::authorize(&headers) {
        Some(user) => Json(success_response(
            auth_service::bootstrap_menu_tree(&state, user.username).await,
        ))
        .into_response(),
        None => error_response(
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized Exception",
        ),
    }
}
