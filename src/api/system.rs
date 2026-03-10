use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    models::system::{
        CreateDeptRequest, CreateMenuRequest, CreateRoleRequest, DeptListResponse, MenuExistsQuery,
        RoleListQuery, UpdateDeptRequest, UpdateMenuRequest, UpdateRoleRequest,
    },
    response::{error_response, success_response},
    services::{auth as auth_service, system as system_service},
    state::AppState,
};

fn ensure_authorized(headers: &HeaderMap) -> Result<(), Response> {
    auth_service::authorize(headers).map(|_| ()).ok_or_else(|| {
        error_response(
            axum::http::StatusCode::UNAUTHORIZED,
            "Unauthorized Exception",
        )
    })
}

pub async fn list_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<RoleListQuery>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::list_roles(&state, query).await {
        Ok(result) => Json(success_response(result)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn create_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRoleRequest>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::create_role(&state, payload).await {
        Ok(role) => Json(success_response(role)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn update_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::update_role(&state, &id, payload).await {
        Ok(role) => Json(success_response(role)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn delete_role(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::delete_role(&state, &id).await {
        Ok(role) => Json(success_response(role)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn list_menus(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::list_menus(&state).await {
        Ok(result) => Json(success_response(result)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn menu_name_exists(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MenuExistsQuery>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::menu_name_exists(&state, query.id.as_deref(), &query.name).await {
        Ok(result) => Json(success_response(result)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn menu_path_exists(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MenuExistsQuery>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::menu_path_exists(&state, query.id.as_deref(), &query.path).await {
        Ok(result) => Json(success_response(result)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn create_menu(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateMenuRequest>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::create_menu(&state, payload).await {
        Ok(menu) => Json(success_response(menu)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn update_menu(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateMenuRequest>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::update_menu(&state, &id, payload).await {
        Ok(menu) => Json(success_response(menu)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn delete_menu(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::delete_menu(&state, &id).await {
        Ok(menu) => Json(success_response(menu)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn list_depts(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::list_depts(&state).await {
        Ok(depts) => {
            let depts: DeptListResponse = depts;
            Json(success_response(depts)).into_response()
        }
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn create_dept(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDeptRequest>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::create_dept(&state, payload).await {
        Ok(dept) => Json(success_response(dept)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn update_dept(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateDeptRequest>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::update_dept(&state, &id, payload).await {
        Ok(dept) => Json(success_response(dept)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}

pub async fn delete_dept(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Err(response) = ensure_authorized(&headers) {
        return response;
    }
    match system_service::delete_dept(&state, &id).await {
        Ok(dept) => Json(success_response(dept)).into_response(),
        Err((status, message)) => error_response(status, message),
    }
}
