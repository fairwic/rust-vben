use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use tower::ServiceExt;

mod support;

fn auth(username: &str) -> String {
    format!("Bearer mock-access:{username}")
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body()).await.expect("read body");
    serde_json::from_slice(&body).expect("parse json")
}

#[tokio::test]
async fn profile_update_should_persist_to_user_info() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/user/profile")
                .header("authorization", auth("vben"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "realName": "Vben Updated",
                        "desc": "Updated introduction"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(update_response.status(), StatusCode::OK);

    let user_info_response = app
        .oneshot(
            Request::builder()
                .uri("/user/info")
                .header("authorization", auth("vben"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(user_info_response.status(), StatusCode::OK);
    let user_info_json = response_json(user_info_response).await;
    assert_eq!(user_info_json["data"]["realName"], "Vben Updated");
    assert_eq!(user_info_json["data"]["desc"], "Updated introduction");
    db.cleanup().await;
}

#[tokio::test]
async fn password_change_should_require_old_password_and_affect_login() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let change_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/user/password")
                .header("authorization", auth("vben"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "oldPassword": "123456",
                        "newPassword": "654321"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(change_response.status(), StatusCode::OK);

    let old_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "vben",
                        "password": "123456"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(old_login.status(), StatusCode::FORBIDDEN);

    let new_login = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "vben",
                        "password": "654321"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(new_login.status(), StatusCode::OK);
    db.cleanup().await;
}

#[tokio::test]
async fn timezone_should_be_persisted_per_user() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let initial_vben = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/timezone/getTimezone")
                .header("authorization", auth("vben"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(initial_vben.status(), StatusCode::OK);
    let initial_vben_json = response_json(initial_vben).await;
    assert_eq!(initial_vben_json["data"], "Asia/Shanghai");

    let set_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/timezone/setTimezone")
                .header("authorization", auth("vben"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "timezone": "UTC"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(set_response.status(), StatusCode::OK);

    let updated_vben = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/timezone/getTimezone")
                .header("authorization", auth("vben"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let updated_vben_json = response_json(updated_vben).await;
    assert_eq!(updated_vben_json["data"], "UTC");

    let admin_timezone = app
        .oneshot(
            Request::builder()
                .uri("/timezone/getTimezone")
                .header("authorization", auth("admin"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let admin_timezone_json = response_json(admin_timezone).await;
    assert_eq!(admin_timezone_json["data"], "Asia/Shanghai");
    db.cleanup().await;
}

#[tokio::test]
async fn auth_codes_and_menu_all_should_follow_database_permissions_only() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let role_update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/system/role/role-user")
                .header("authorization", auth("vben"))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "普通用户",
                        "permissions": [],
                        "remark": "no permissions",
                        "status": 1
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(role_update.status(), StatusCode::OK);

    let codes_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/codes")
                .header("authorization", auth("jack"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(codes_response.status(), StatusCode::OK);
    let codes_json = response_json(codes_response).await;
    assert_eq!(codes_json["data"], json!([]));

    let menu_response = app
        .oneshot(
            Request::builder()
                .uri("/menu/all")
                .header("authorization", auth("jack"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(menu_response.status(), StatusCode::OK);
    let menu_json = response_json(menu_response).await;
    assert_eq!(menu_json["data"], json!([]));
    db.cleanup().await;
}
