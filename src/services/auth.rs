use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use serde_json::Value;

use crate::{
    models::auth::{
        AuthUser, AuthUserRecord, LoginRequest, LoginResponse, TimezoneRequest,
        UpdatePasswordRequest, UpdateProfileRequest, UserInfoResponse,
    },
    repositories::user as user_repo,
    state::AppState,
};

pub async fn login(
    state: &AppState,
    payload: LoginRequest,
) -> Result<(HeaderValue, LoginResponse), (StatusCode, &'static str)> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username and password are required",
        ));
    }

    let user = user_repo::get_auth_user_by_username(&state.pool, &payload.username)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?
        .ok_or((StatusCode::FORBIDDEN, "Username or password is incorrect."))?;
    if user.status != 1 || user.password != payload.password {
        return Err((StatusCode::FORBIDDEN, "Username or password is incorrect."));
    }

    let auth_user = build_auth_user(state, user)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
    Ok((
        refresh_cookie_for(&auth_user.username),
        to_login_response(&auth_user),
    ))
}

pub async fn refresh(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(HeaderValue, String), (StatusCode, &'static str)> {
    let username = username_from_refresh_headers(headers)
        .ok_or((StatusCode::FORBIDDEN, "Forbidden Exception"))?;
    let user = user_repo::get_auth_user_by_username(&state.pool, &username)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?
        .ok_or((StatusCode::FORBIDDEN, "Forbidden Exception"))?;
    if user.status != 1 {
        return Err((StatusCode::FORBIDDEN, "Forbidden Exception"));
    }
    Ok((refresh_cookie_for(&username), access_token_for(&username)))
}

pub async fn authorize(state: &AppState, headers: &HeaderMap) -> Option<AuthUser> {
    let username = authorize_sync(headers)?;
    let user = user_repo::get_auth_user_by_username(&state.pool, &username)
        .await
        .ok()??;
    if user.status != 1 {
        return None;
    }
    build_auth_user(state, user).await.ok()
}

pub fn authorize_sync(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(username_from_authorization)
}

pub fn to_user_info(user: &AuthUser) -> UserInfoResponse {
    UserInfoResponse {
        avatar: user.avatar.clone(),
        desc: user.desc.clone(),
        home_path: user.home_path.clone(),
        real_name: user.real_name.clone(),
        roles: user.roles.clone(),
        user_id: user.user_id.clone(),
        username: user.username.clone(),
    }
}

pub async fn bootstrap_menu_tree(state: &AppState, user: &AuthUser) -> Value {
    let menus = crate::services::system::navigation_menus_for_user(state, &user.user_id)
        .await
        .ok()
        .unwrap_or_default();
    serde_json::to_value(menus).unwrap_or_else(|_| Value::Array(Vec::new()))
}

pub fn access_token_for(username: &str) -> String {
    format!("mock-access:{username}")
}

pub fn username_from_authorization(authorization: &str) -> Option<String> {
    authorization
        .strip_prefix("Bearer ")
        .and_then(|token| token.strip_prefix("mock-access:"))
        .map(ToOwned::to_owned)
}

pub fn read_refresh_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';').find_map(|part| {
                let trimmed = part.trim();
                trimmed.strip_prefix("jwt=").map(str::to_string)
            })
        })
}

pub fn refresh_cookie_for(username: &str) -> HeaderValue {
    let value =
        format!("jwt=mock-refresh:{username}; HttpOnly; Max-Age=86400; Path=/; SameSite=Lax");
    HeaderValue::from_str(&value).expect("valid refresh cookie")
}

pub fn expired_refresh_cookie() -> HeaderValue {
    HeaderValue::from_static("jwt=; HttpOnly; Max-Age=0; Path=/; SameSite=Lax")
}

fn username_from_refresh_headers(headers: &HeaderMap) -> Option<String> {
    read_refresh_token(headers)
        .and_then(|token| token.strip_prefix("mock-refresh:").map(ToOwned::to_owned))
}

fn to_login_response(user: &AuthUser) -> LoginResponse {
    LoginResponse {
        access_token: access_token_for(&user.username),
        avatar: user.avatar.clone(),
        home_path: user.home_path.clone(),
        id: user.user_id.clone(),
        real_name: user.real_name.clone(),
        roles: user.roles.clone(),
        user_id: user.user_id.clone(),
        username: user.username.clone(),
    }
}

pub async fn update_profile(
    state: &AppState,
    user: &AuthUser,
    payload: UpdateProfileRequest,
) -> Result<UserInfoResponse, (StatusCode, &'static str)> {
    if payload.real_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Real name is required"));
    }

    user_repo::update_user_profile(&state.pool, &user.user_id, &payload.real_name, &payload.desc)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;

    let updated = user_repo::get_auth_user_by_username(&state.pool, &user.username)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))?;
    let auth_user = build_auth_user(state, updated)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;

    Ok(to_user_info(&auth_user))
}

pub async fn update_password(
    state: &AppState,
    user: &AuthUser,
    payload: UpdatePasswordRequest,
) -> Result<(), (StatusCode, &'static str)> {
    if payload.old_password.is_empty() || payload.new_password.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password is required"));
    }

    let current_user = user_repo::get_auth_user_by_username(&state.pool, &user.username)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))?;
    if current_user.password != payload.old_password {
        return Err((StatusCode::FORBIDDEN, "Username or password is incorrect."));
    }

    user_repo::change_user_password(&state.pool, &user.user_id, &payload.new_password)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;

    Ok(())
}

pub async fn get_timezone(
    state: &AppState,
    user: &AuthUser,
) -> Result<String, (StatusCode, &'static str)> {
    user_repo::get_user_timezone(&state.pool, &user.user_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))
}

pub async fn set_timezone(
    state: &AppState,
    user: &AuthUser,
    payload: TimezoneRequest,
) -> Result<(), (StatusCode, &'static str)> {
    if payload.timezone.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Timezone is required"));
    }

    user_repo::set_user_timezone(&state.pool, &user.user_id, &payload.timezone)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"))?;
    Ok(())
}

async fn build_auth_user(state: &AppState, user: AuthUserRecord) -> Result<AuthUser, sqlx::Error> {
    let permission_ids = user_repo::list_user_permission_ids(&state.pool, &user.user_id).await?;
    let access_codes = user_repo::list_permission_auth_codes(&state.pool, &permission_ids).await?;

    Ok(AuthUser {
        access_codes,
        avatar: user.avatar,
        desc: user.desc,
        home_path: user.home_path,
        id: user.id.clone(),
        real_name: user.real_name,
        roles: user.role_ids.iter().map(|role_id| role_code_from_id(role_id)).collect(),
        status: user.status,
        user_id: user.user_id,
        username: user.username,
    })
}

fn role_code_from_id(role_id: &str) -> String {
    role_id
        .strip_prefix("role-")
        .unwrap_or(role_id)
        .to_string()
}
