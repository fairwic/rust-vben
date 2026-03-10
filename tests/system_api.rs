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
async fn role_list_uses_latest_paginated_shape() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/system/role/list?page=1&pageSize=10")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert!(json["data"]["items"].is_array());
    assert!(json["data"]["total"].is_number());
    db.cleanup().await;
}

#[tokio::test]
async fn create_update_delete_role_flow_is_supported() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/role")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "QA",
                        "permissions": ["System:Menu:List"],
                        "remark": "quality",
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
    let role_id = created["data"]["id"]
        .as_str()
        .expect("created role id")
        .to_string();

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/system/role/{role_id}"))
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "QA Team",
                        "permissions": ["System:Menu:List", "System:Dept:List"],
                        "remark": "updated",
                        "status": 0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(update_response.status(), StatusCode::OK);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/system/role/list?page=1&pageSize=20")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let list_json = response_json(list_response).await;
    let role = list_json["data"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["id"] == role_id)
        .cloned()
        .expect("updated role in list");
    assert_eq!(role["name"], "QA Team");
    assert_eq!(role["status"], 0);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/system/role/{role_id}"))
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(delete_response.status(), StatusCode::OK);
    db.cleanup().await;
}

#[tokio::test]
async fn menu_crud_and_uniqueness_checks_follow_latest_contract() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/menu")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "authCode": "System:Reports:List",
                        "component": "/reports/list",
                        "meta": {
                            "icon": "carbon:report",
                            "title": "system.reports.title"
                        },
                        "name": "SystemReports",
                        "path": "/system/reports",
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
    let created = response_json(create_response).await;
    let menu_id = created["data"]["id"].as_str().expect("menu id").to_string();

    let name_exists_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/system/menu/name-exists?name=SystemReports")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(name_exists_response.status(), StatusCode::OK);
    let name_exists = response_json(name_exists_response).await;
    assert_eq!(name_exists["data"], true);

    let path_exists_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/system/menu/path-exists?path=%2Fsystem%2Freports")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(path_exists_response.status(), StatusCode::OK);
    let path_exists = response_json(path_exists_response).await;
    assert_eq!(path_exists["data"], true);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/system/menu/{menu_id}"))
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(delete_response.status(), StatusCode::OK);
    db.cleanup().await;
}

#[tokio::test]
async fn dept_crud_updates_tree_results() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/dept")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Platform",
                        "pid": "1",
                        "remark": "platform team",
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

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/system/dept/{dept_id}"))
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Platform Team",
                        "pid": "1",
                        "remark": "updated team",
                        "status": 0
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(update_response.status(), StatusCode::OK);

    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/system/dept/list")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let list_json = response_json(list_response).await;
    let tree = list_json["data"].as_array().expect("dept tree");
    let engineering = tree
        .iter()
        .find(|item| item["id"] == "1")
        .expect("root dept");
    let created_node = engineering["children"]
        .as_array()
        .expect("children")
        .iter()
        .find(|item| item["id"] == dept_id)
        .cloned()
        .expect("created dept");
    assert_eq!(created_node["name"], "Platform Team");
    assert_eq!(created_node["status"], 0);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/system/dept/{dept_id}"))
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(delete_response.status(), StatusCode::OK);
    db.cleanup().await;
}
