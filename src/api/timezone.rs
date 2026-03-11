use axum::{
    extract::{Json, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::{
    models::auth::TimezoneRequest,
    response::{error_response, success_response},
    services::auth as auth_service,
    state::AppState,
};

pub async fn get_timezone(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match auth_service::authorize(&state, &headers).await {
        Some(user) => match auth_service::get_timezone(&state, &user).await {
            Ok(timezone) => Json(success_response(timezone)).into_response(),
            Err((status, message)) => error_response(status, message),
        },
        None => error_response(
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized Exception",
        ),
    }
}

pub async fn get_timezone_options() -> impl IntoResponse {
    Json(success_response(vec![
        json!({ "label": "(UTC+08:00) Asia/Shanghai", "value": "Asia/Shanghai" }),
        json!({ "label": "(UTC+09:00) Asia/Tokyo", "value": "Asia/Tokyo" }),
        json!({ "label": "(UTC+00:00) UTC", "value": "UTC" }),
    ]))
}

pub async fn set_timezone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TimezoneRequest>,
) -> Response {
    match auth_service::authorize(&state, &headers).await {
        Some(user) => match auth_service::set_timezone(&state, &user, payload).await {
            Ok(()) => Json(success_response("")).into_response(),
            Err((status, message)) => error_response(status, message),
        },
        None => error_response(
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized Exception",
        ),
    }
}
