use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: T,
    pub error: Option<String>,
    pub message: String,
}

pub fn success_response<T>(data: T) -> ApiResponse<T> {
    ApiResponse {
        code: 0,
        data,
        error: None,
        message: "ok".to_string(),
    }
}

pub fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(ApiResponse {
            code: -1,
            data: Value::Null,
            error: Some(message.to_string()),
            message: message.to_string(),
        }),
    )
        .into_response()
}
