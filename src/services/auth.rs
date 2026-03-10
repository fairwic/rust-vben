use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use serde_json::{json, Value};

use crate::{
    models::auth::{LoginRequest, LoginResponse, MockUser, UserInfoResponse, MOCK_USERS},
    state::AppState,
};

pub async fn login(
    _state: &AppState,
    payload: LoginRequest,
) -> Result<(HeaderValue, LoginResponse), (StatusCode, &'static str)> {
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username and password are required",
        ));
    }

    let user = find_user_by_credentials(&payload.username, &payload.password)
        .ok_or((StatusCode::FORBIDDEN, "Username or password is incorrect."))?;

    Ok((refresh_cookie_for(user.username), to_login_response(user)))
}

pub fn refresh(headers: &HeaderMap) -> Result<(HeaderValue, String), (StatusCode, &'static str)> {
    let username = username_from_refresh_headers(headers)
        .ok_or((StatusCode::FORBIDDEN, "Forbidden Exception"))?;
    Ok((refresh_cookie_for(&username), access_token_for(&username)))
}

pub fn authorize(headers: &HeaderMap) -> Option<&'static MockUser> {
    let username = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(username_from_authorization)?;
    find_user_by_username(&username)
}

pub fn access_codes_for(username: &str) -> Vec<String> {
    match username {
        "vben" | "admin" => vec![
            "AC_100010".into(),
            "AC_100020".into(),
            "AC_100030".into(),
            "System:Role:List".into(),
            "System:Role:Create".into(),
            "System:Role:Edit".into(),
            "System:Role:Delete".into(),
            "System:Menu:List".into(),
            "System:Menu:Create".into(),
            "System:Menu:Edit".into(),
            "System:Menu:Delete".into(),
            "System:Dept:List".into(),
            "System:Dept:Create".into(),
            "System:Dept:Edit".into(),
            "System:Dept:Delete".into(),
        ],
        "jack" => vec!["AC_1000001".into(), "AC_1000002".into()],
        _ => Vec::new(),
    }
}

pub fn to_user_info(user: &MockUser) -> UserInfoResponse {
    UserInfoResponse {
        avatar: user.avatar.to_string(),
        home_path: user.home_path.to_string(),
        real_name: user.real_name.to_string(),
        roles: user.roles.iter().map(|role| (*role).to_string()).collect(),
        user_id: user.user_id.to_string(),
        username: user.username.to_string(),
    }
}

pub async fn bootstrap_menu_tree(state: &AppState, username: &str) -> Value {
    let system_children = crate::services::system::navigation_menus(state)
        .await
        .ok()
        .and_then(|menus| {
            menus.into_iter()
                .find(|menu| menu.path == "/system")
                .and_then(|menu| menu.children)
        })
        .unwrap_or_default();

    let role_specific = match username {
        "vben" => json!({
            "component": "/demos/access/super-visible",
            "meta": {
                "icon": "mdi:button-cursor",
                "title": "demos.access.superVisible"
            },
            "name": "AccessSuperVisibleDemo",
            "path": "/demos/access/super-visible"
        }),
        "admin" => json!({
            "component": "/demos/access/admin-visible",
            "meta": {
                "icon": "mdi:button-cursor",
                "title": "demos.access.adminVisible"
            },
            "name": "AccessAdminVisibleDemo",
            "path": "/demos/access/admin-visible"
        }),
        _ => json!({
            "component": "/demos/access/user-visible",
            "meta": {
                "icon": "mdi:button-cursor",
                "title": "demos.access.userVisible"
            },
            "name": "AccessUserVisibleDemo",
            "path": "/demos/access/user-visible"
        }),
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

    if username != "jack" {
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

fn to_login_response(user: &MockUser) -> LoginResponse {
    LoginResponse {
        access_token: access_token_for(user.username),
        avatar: user.avatar.to_string(),
        home_path: user.home_path.to_string(),
        id: user.user_id.to_string(),
        real_name: user.real_name.to_string(),
        roles: user.roles.iter().map(|role| (*role).to_string()).collect(),
        user_id: user.user_id.to_string(),
        username: user.username.to_string(),
    }
}

fn find_user_by_credentials(username: &str, password: &str) -> Option<&'static MockUser> {
    MOCK_USERS
        .iter()
        .find(|user| user.username == username && user.password == password)
}

fn find_user_by_username(username: &str) -> Option<&'static MockUser> {
    MOCK_USERS.iter().find(|user| user.username == username)
}
