use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use tower::ServiceExt;

mod support;

fn admin_auth() -> &'static str {
    "Bearer mock-access:vben"
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body()).await.expect("read body");
    serde_json::from_slice(&body).expect("parse json")
}

#[tokio::test]
async fn user_list_uses_paginated_shape() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/system/user/list?page=1&pageSize=10")
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
async fn create_update_delete_user_flow_is_supported() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/user")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "qa-user",
                        "password": "123456",
                        "realName": "QA User",
                        "email": "qa-user@example.com",
                        "phone": "13800000111",
                        "deptId": "2",
                        "roleIds": ["role-admin"],
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
    let user_id = created["data"]["id"].as_str().expect("user id").to_string();

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/system/user/{user_id}"))
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "realName": "QA Team User",
                        "email": "qa-team@example.com",
                        "phone": "13800000112",
                        "deptId": "3",
                        "roleIds": ["role-user"],
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
                .uri("/system/user/list?page=1&pageSize=20")
                .header("authorization", admin_auth())
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_json = response_json(list_response).await;
    let user = list_json["data"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["id"] == user_id)
        .cloned()
        .expect("updated user");

    assert_eq!(user["realName"], "QA Team User");
    assert_eq!(user["email"], "qa-team@example.com");
    assert_eq!(user["deptId"], "3");
    assert_eq!(user["status"], 0);
    assert_eq!(user["roleIds"], json!(["role-user"]));

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/system/user/{user_id}"))
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
async fn created_user_can_login_and_fetch_user_info_from_persistence() {
    let db = support::TestDatabase::new().await;
    let app = db.app().await;

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/system/user")
                .header("authorization", admin_auth())
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "persisted-login",
                        "password": "123456",
                        "realName": "Persisted Login",
                        "deptId": "2",
                        "roleIds": ["role-admin"],
                        "status": 1
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(create_response.status(), StatusCode::OK);

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "persisted-login",
                        "password": "123456"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(login_response.status(), StatusCode::OK);
    let login_json = response_json(login_response).await;
    let access_token = login_json["data"]["accessToken"]
        .as_str()
        .expect("access token")
        .to_string();

    let user_info_response = app
        .oneshot(
            Request::builder()
                .uri("/user/info")
                .header("authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(user_info_response.status(), StatusCode::OK);
    let user_info_json = response_json(user_info_response).await;

    assert_eq!(user_info_json["data"]["username"], "persisted-login");
    assert_eq!(user_info_json["data"]["realName"], "Persisted Login");
    assert_eq!(user_info_json["data"]["roles"], json!(["admin"]));
    db.cleanup().await;
}
