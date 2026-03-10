use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    api::{auth, system, timezone},
    db::database_url_from_env,
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(auth::health))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/codes", get(auth::access_codes))
        .route("/user/info", get(auth::user_info))
        .route("/menu/all", get(auth::menu_all))
        .route("/system/role/list", get(system::list_roles))
        .route("/system/role", post(system::create_role))
        .route(
            "/system/role/:id",
            put(system::update_role).delete(system::delete_role),
        )
        .route("/system/menu/list", get(system::list_menus))
        .route("/system/menu/name-exists", get(system::menu_name_exists))
        .route("/system/menu/path-exists", get(system::menu_path_exists))
        .route("/system/menu", post(system::create_menu))
        .route(
            "/system/menu/:id",
            put(system::update_menu).delete(system::delete_menu),
        )
        .route("/system/dept/list", get(system::list_depts))
        .route("/system/dept", post(system::create_dept))
        .route(
            "/system/dept/:id",
            put(system::update_dept).delete(system::delete_dept),
        )
        .route("/timezone/getTimezone", get(timezone::get_timezone))
        .route(
            "/timezone/getTimezoneOptions",
            get(timezone::get_timezone_options),
        )
        .route("/timezone/setTimezone", post(timezone::set_timezone))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin([
                    HeaderValue::from_static("http://127.0.0.1:5555"),
                    HeaderValue::from_static("http://localhost:5555"),
                    HeaderValue::from_static("http://127.0.0.1:5666"),
                    HeaderValue::from_static("http://localhost:5666"),
                    HeaderValue::from_static("http://127.0.0.1:5999"),
                    HeaderValue::from_static("http://localhost:5999"),
                ])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_credentials(true),
        )
}

pub async fn build_app() -> Result<Router, sqlx::Error> {
    build_app_with_database_url(&database_url_from_env()).await
}

pub async fn build_app_with_database_url(database_url: &str) -> Result<Router, sqlx::Error> {
    let state = AppState::from_database_url(database_url).await?;
    Ok(build_router(state))
}
