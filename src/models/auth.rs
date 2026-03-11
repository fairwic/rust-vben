use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub access_codes: Vec<String>,
    pub avatar: String,
    pub home_path: String,
    pub id: String,
    pub real_name: String,
    pub roles: Vec<String>,
    pub status: i32,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Clone)]
pub struct AuthUserRecord {
    pub avatar: String,
    pub home_path: String,
    pub id: String,
    pub password: String,
    pub real_name: String,
    pub role_ids: Vec<String>,
    pub status: i32,
    pub user_id: String,
    pub username: String,
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
