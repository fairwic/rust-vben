use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct MockUser {
    pub avatar: &'static str,
    pub home_path: &'static str,
    pub password: &'static str,
    pub real_name: &'static str,
    pub roles: &'static [&'static str],
    pub user_id: &'static str,
    pub username: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub avatar: String,
    #[serde(rename = "homePath")]
    pub home_path: String,
    pub id: String,
    #[serde(rename = "realName")]
    pub real_name: String,
    pub roles: Vec<String>,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub avatar: String,
    #[serde(rename = "homePath")]
    pub home_path: String,
    #[serde(rename = "realName")]
    pub real_name: String,
    pub roles: Vec<String>,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
}

pub const MOCK_USERS: [MockUser; 3] = [
    MockUser {
        avatar: "",
        home_path: "/analytics",
        password: "123456",
        real_name: "Vben",
        roles: &["super"],
        user_id: "0",
        username: "vben",
    },
    MockUser {
        avatar: "",
        home_path: "/workspace",
        password: "123456",
        real_name: "Admin",
        roles: &["admin"],
        user_id: "1",
        username: "admin",
    },
    MockUser {
        avatar: "",
        home_path: "/analytics",
        password: "123456",
        real_name: "Jack",
        roles: &["user"],
        user_id: "2",
        username: "jack",
    },
];
