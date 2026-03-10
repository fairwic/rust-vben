use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use tower::ServiceExt;

mod support;

fn admin_auth() -> &'static str {
    "Bearer mock-access:admin"
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body()).await.expect("read body");
    serde_json::from_slice(&body).expect("parse json")
}

#[tokio::test]
async fn created_role_should_survive_new_app_instance() {
    let db = support::TestDatabase::new().await;
    let app_a = db.app().await;

    let create_response = app_a
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/role")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Persisted Role",
                        "permissions": ["System:Menu:List"],
                        "remark": "should persist",
                        "status": 1
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    let role_id = created["data"]["id"].as_str().expect("role id").to_string();

    let app_b = db.app().await;
    let list_response = app_b
        .oneshot(
            Request::builder()
                .uri("/system/role/list?page=1&pageSize=50")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = response_json(list_response).await;
    let exists = list_json["data"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .any(|item| item["id"] == role_id);

    assert!(exists, "expected role to persist across app instances");
    db.cleanup().await;
}

#[tokio::test]
async fn created_menu_should_keep_uniqueness_checks_after_new_app_instance() {
    let db = support::TestDatabase::new().await;
    let app_a = db.app().await;

    let create_response = app_a
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/menu")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "authCode": "System:Persisted:List",
                        "component": "/persisted/list",
                        "meta": {
                            "icon": "carbon:catalog",
                            "title": "system.persisted.title"
                        },
                        "name": "PersistedMenu",
                        "path": "/system/persisted",
                        "pid": "2",
                        "status": 1,
                        "type": "menu"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(create_response.status(), StatusCode::OK);

    let app_b = db.app().await;
    let name_exists_response = app_b
        .clone()
        .oneshot(
            Request::builder()
                .uri("/system/menu/name-exists?name=PersistedMenu")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(name_exists_response.status(), StatusCode::OK);
    let name_exists = response_json(name_exists_response).await;
    assert_eq!(name_exists["data"], true, "expected name uniqueness to persist");

    let path_exists_response = app_b
        .oneshot(
            Request::builder()
                .uri("/system/menu/path-exists?path=%2Fsystem%2Fpersisted")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(path_exists_response.status(), StatusCode::OK);
    let path_exists = response_json(path_exists_response).await;
    assert_eq!(path_exists["data"], true, "expected path uniqueness to persist");
    db.cleanup().await;
}

#[tokio::test]
async fn created_dept_should_survive_new_app_instance() {
    let db = support::TestDatabase::new().await;
    let app_a = db.app().await;

    let create_response = app_a
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/dept")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Persisted Dept",
                        "pid": "1",
                        "remark": "should persist",
                        "status": 1
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(create_response.status(), StatusCode::OK);
    let created = response_json(create_response).await;
    let dept_id = created["data"]["id"].as_str().expect("dept id").to_string();

    let app_b = db.app().await;
    let list_response = app_b
        .oneshot(
            Request::builder()
                .uri("/system/dept/list")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = response_json(list_response).await;
    let root = list_json["data"]
        .as_array()
        .expect("tree")
        .iter()
        .find(|item| item["id"] == "1")
        .expect("root");
    let exists = root["children"]
        .as_array()
        .expect("children")
        .iter()
        .any(|item| item["id"] == dept_id);

    assert!(exists, "expected dept to persist across app instances");
    db.cleanup().await;
}
