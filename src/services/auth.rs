use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use serde_json::{json, Value};

use crate::{
    models::auth::{AuthUser, AuthUserRecord, LoginRequest, LoginResponse, UserInfoResponse},
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
        home_path: user.home_path.clone(),
        real_name: user.real_name.clone(),
        roles: user.roles.clone(),
        user_id: user.user_id.clone(),
        username: user.username.clone(),
    }
}

pub async fn bootstrap_menu_tree(state: &AppState, user: &AuthUser) -> Value {
    let system_children = crate::services::system::navigation_menus_for_user(state, &user.user_id)
        .await
        .ok()
        .unwrap_or_default();

    let role_specific = if user.roles.iter().any(|role| role == "super") {
        json!({
            "component": "/demos/access/super-visible",
            "meta": {
                "icon": "mdi:button-cursor",
                "title": "demos.access.superVisible"
            },
            "name": "AccessSuperVisibleDemo",
            "path": "/demos/access/super-visible"
        })
    } else if user.roles.iter().any(|role| role == "admin") {
        json!({
            "component": "/demos/access/admin-visible",
            "meta": {
                "icon": "mdi:button-cursor",
                "title": "demos.access.adminVisible"
            },
            "name": "AccessAdminVisibleDemo",
            "path": "/demos/access/admin-visible"
        })
    } else {
        json!({
            "component": "/demos/access/user-visible",
            "meta": {
                "icon": "mdi:button-cursor",
                "title": "demos.access.userVisible"
            },
            "name": "AccessUserVisibleDemo",
            "path": "/demos/access/user-visible"
        })
    };

    let mut menus = vec![
        json!({
            "meta": {
                "order": -1,
                "title": "page.dashboard.title"
            },
            "name": "Dashboard",
            "path": "/dashboard",
            "redirect": "/analytics",
            "children": [
                {
                    "name": "Analytics",
                    "path": "/analytics",
                    "component": "/dashboard/analytics/index",
                    "meta": {
                        "affixTab": true,
                        "title": "page.dashboard.analytics"
                    }
                },
                {
                    "name": "Workspace",
                    "path": "/workspace",
                    "component": "/dashboard/workspace/index",
                    "meta": {
                        "title": "page.dashboard.workspace"
                    }
                }
            ]
        }),
        json!({
            "meta": {
                "icon": "ic:baseline-view-in-ar",
                "keepAlive": true,
                "order": 1000,
                "title": "demos.title"
            },
            "name": "Demos",
            "path": "/demos",
            "redirect": "/demos/access",
            "children": [
                {
                    "name": "AccessDemos",
                    "path": "/demosaccess",
                    "meta": {
                        "icon": "mdi:cloud-key-outline",
                        "title": "demos.access.backendPermissions"
                    },
                    "redirect": "/demos/access/page-control",
                    "children": [
                        {
                            "name": "AccessPageControlDemo",
                            "path": "/demos/access/page-control",
                            "component": "/demos/access/index",
                            "meta": {
                                "icon": "mdi:page-previous-outline",
                                "title": "demos.access.pageAccess"
                            }
                        },
                        {
                            "name": "AccessButtonControlDemo",
                            "path": "/demos/access/button-control",
                            "component": "/demos/access/button-control",
                            "meta": {
                                "icon": "mdi:button-cursor",
                                "title": "demos.access.buttonControl"
                            }
                        },
                        role_specific
                    ]
                }
            ]
        }),
    ];

    if !system_children.is_empty() {
        menus.push(json!({
            "meta": {
                "icon": "ion:settings-outline",
                "order": 9997,
                "title": "system.title"
            },
            "name": "System",
            "path": "/system",
            "children": system_children
        }));
    }

    Value::Array(menus)
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

async fn build_auth_user(state: &AppState, user: AuthUserRecord) -> Result<AuthUser, sqlx::Error> {
    let permission_ids = user_repo::list_user_permission_ids(&state.pool, &user.user_id).await?;
    let mut access_codes = user_repo::list_permission_auth_codes(&state.pool, &permission_ids).await?;
    access_codes.extend(demo_access_codes(&user.role_ids));
    access_codes.sort();
    access_codes.dedup();

    Ok(AuthUser {
        access_codes,
        avatar: user.avatar,
        home_path: user.home_path,
        id: user.id.clone(),
        real_name: user.real_name,
        roles: user.role_ids.iter().map(|role_id| role_code_from_id(role_id)).collect(),
        status: user.status,
        user_id: user.user_id,
        username: user.username,
    })
}

fn demo_access_codes(role_ids: &[String]) -> Vec<String> {
    let role_codes = role_ids
        .iter()
        .map(|role_id| role_code_from_id(role_id))
        .collect::<Vec<_>>();
    if role_codes.iter().any(|role| role == "super" || role == "admin") {
        return vec![
            "AC_100010".into(),
            "AC_100020".into(),
            "AC_100030".into(),
        ];
    }
    if role_codes.iter().any(|role| role == "user") {
        return vec!["AC_1000001".into(), "AC_1000002".into()];
    }
    Vec::new()
}

fn role_code_from_id(role_id: &str) -> String {
    role_id
        .strip_prefix("role-")
        .unwrap_or(role_id)
        .to_string()
}
