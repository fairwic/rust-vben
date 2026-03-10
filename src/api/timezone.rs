use axum::{extract::Json, response::IntoResponse};
use serde_json::{json, Value};

use crate::response::success_response;

pub async fn get_timezone() -> impl IntoResponse {
    Json(success_response("Asia/Shanghai"))
}

pub async fn get_timezone_options() -> impl IntoResponse {
    Json(success_response(vec![
        json!({ "label": "(UTC+08:00) Asia/Shanghai", "value": "Asia/Shanghai" }),
        json!({ "label": "(UTC+09:00) Asia/Tokyo", "value": "Asia/Tokyo" }),
        json!({ "label": "(UTC+00:00) UTC", "value": "UTC" }),
    ]))
}

pub async fn set_timezone(Json(_payload): Json<Value>) -> impl IntoResponse {
    Json(success_response(""))
}
