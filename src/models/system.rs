use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemRole {
    #[serde(rename = "createTime")]
    pub create_time: String,
    pub id: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub remark: String,
    pub status: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub permissions: Vec<String>,
    pub remark: Option<String>,
    pub status: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub remark: Option<String>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoleListQuery {
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
    pub page: Option<usize>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<usize>,
    pub remark: Option<String>,
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    pub status: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleListResponse {
    pub items: Vec<SystemRole>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MenuMeta {
    #[serde(
        rename = "activeIcon",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_icon: Option<String>,
    #[serde(
        rename = "activePath",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub active_path: Option<String>,
    #[serde(rename = "affixTab", default, skip_serializing_if = "Option::is_none")]
    pub affix_tab: Option<bool>,
    #[serde(
        rename = "affixTabOrder",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub affix_tab_order: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge: Option<String>,
    #[serde(rename = "badgeType", default, skip_serializing_if = "Option::is_none")]
    pub badge_type: Option<String>,
    #[serde(
        rename = "badgeVariants",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub badge_variants: Option<String>,
    #[serde(
        rename = "hideChildrenInMenu",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hide_children_in_menu: Option<bool>,
    #[serde(
        rename = "hideInBreadcrumb",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hide_in_breadcrumb: Option<bool>,
    #[serde(
        rename = "hideInMenu",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub hide_in_menu: Option<bool>,
    #[serde(rename = "hideInTab", default, skip_serializing_if = "Option::is_none")]
    pub hide_in_tab: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(rename = "iframeSrc", default, skip_serializing_if = "Option::is_none")]
    pub iframe_src: Option<String>,
    #[serde(rename = "keepAlive", default, skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(
        rename = "maxNumOfOpenTab",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub max_num_of_open_tab: Option<i32>,
    #[serde(
        rename = "noBasicLayout",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub no_basic_layout: Option<bool>,
    #[serde(
        rename = "openInNewWindow",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub open_in_new_window: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMenu {
    #[serde(rename = "authCode", default, skip_serializing_if = "Option::is_none")]
    pub auth_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<SystemMenu>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    pub id: String,
    #[serde(default)]
    pub meta: MenuMeta,
    pub name: String,
    pub path: String,
    pub pid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    pub status: i32,
    #[serde(rename = "type")]
    pub menu_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateMenuRequest {
    #[serde(rename = "activePath", default)]
    pub active_path: Option<String>,
    #[serde(rename = "authCode", default)]
    pub auth_code: Option<String>,
    pub component: Option<String>,
    #[serde(default)]
    pub meta: MenuMeta,
    pub name: String,
    pub path: String,
    pub pid: Option<String>,
    pub redirect: Option<String>,
    pub status: i32,
    #[serde(rename = "type")]
    pub menu_type: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateMenuRequest {
    #[serde(rename = "activePath", default)]
    pub active_path: Option<String>,
    #[serde(rename = "authCode", default)]
    pub auth_code: Option<String>,
    pub component: Option<String>,
    #[serde(default)]
    pub meta: MenuMeta,
    pub name: Option<String>,
    pub path: Option<String>,
    pub pid: Option<String>,
    pub redirect: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "type")]
    pub menu_type: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MenuExistsQuery {
    pub id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
}

pub type MenuListResponse = Vec<SystemMenu>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDept {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<SystemDept>>,
    #[serde(rename = "createTime")]
    pub create_time: String,
    pub id: String,
    pub name: String,
    pub pid: String,
    pub remark: String,
    pub status: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDeptRequest {
    pub name: String,
    pub pid: Option<String>,
    pub remark: Option<String>,
    pub status: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateDeptRequest {
    pub name: Option<String>,
    pub pid: Option<String>,
    pub remark: Option<String>,
    pub status: Option<i32>,
}

pub type DeptListResponse = Vec<SystemDept>;
