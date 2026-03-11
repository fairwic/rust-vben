use axum::http::StatusCode;

use crate::{
    models::system::{
        CreateDeptRequest, CreateMenuRequest, CreateRoleRequest, DeptListResponse, MenuListResponse,
        RoleListQuery, RoleListResponse, SystemDept, SystemMenu, SystemRole, SystemUser,
        UpdateDeptRequest, UpdateMenuRequest, UpdateRoleRequest, UpdateUserRequest,
        UserListQuery, UserListResponse, UserRecord, CreateUserRequest,
    },
    repositories::{system as system_repo, user as user_repo},
    state::AppState,
};

pub async fn list_roles(
    state: &AppState,
    query: RoleListQuery,
) -> Result<RoleListResponse, (StatusCode, &'static str)> {
    let (items, total) = system_repo::list_roles(&state.pool, &query)
        .await
        .map_err(internal_error)?;
    Ok(RoleListResponse { items, total })
}

pub async fn create_role(
    state: &AppState,
    payload: CreateRoleRequest,
) -> Result<SystemRole, (StatusCode, &'static str)> {
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Role name is required"));
    }

    let role = SystemRole {
        create_time: String::new(),
        id: generate_id("role"),
        name: payload.name,
        permissions: payload.permissions,
        remark: payload.remark.unwrap_or_default(),
        status: payload.status,
    };

    system_repo::insert_role(&state.pool, &role)
        .await
        .map_err(map_role_write_error)
}

pub async fn update_role(
    state: &AppState,
    id: &str,
    payload: UpdateRoleRequest,
) -> Result<SystemRole, (StatusCode, &'static str)> {
    let mut role = system_repo::get_role(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Role not found"))?;

    if let Some(name) = payload.name {
        role.name = name;
    }
    if let Some(permissions) = payload.permissions {
        role.permissions = permissions;
    }
    if let Some(remark) = payload.remark {
        role.remark = remark;
    }
    if let Some(status) = payload.status {
        role.status = status;
    }

    system_repo::update_role(&state.pool, &role)
        .await
        .map_err(map_role_write_error)
}

pub async fn delete_role(
    state: &AppState,
    id: &str,
) -> Result<SystemRole, (StatusCode, &'static str)> {
    system_repo::delete_role(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Role not found"))
}

pub async fn list_menus(state: &AppState) -> Result<MenuListResponse, (StatusCode, &'static str)> {
    let menus = system_repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;
    Ok(build_menu_tree(&menus))
}

pub async fn menu_name_exists(
    state: &AppState,
    current_id: Option<&str>,
    name: &str,
) -> Result<bool, (StatusCode, &'static str)> {
    system_repo::menu_name_exists(&state.pool, current_id, name)
        .await
        .map_err(internal_error)
}

pub async fn menu_path_exists(
    state: &AppState,
    current_id: Option<&str>,
    path: &str,
) -> Result<bool, (StatusCode, &'static str)> {
    system_repo::menu_path_exists(&state.pool, current_id, path)
        .await
        .map_err(internal_error)
}

pub async fn create_menu(
    state: &AppState,
    payload: CreateMenuRequest,
) -> Result<SystemMenu, (StatusCode, &'static str)> {
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Menu name is required"));
    }
    if payload.path.trim().is_empty() && payload.menu_type != "button" {
        return Err((StatusCode::BAD_REQUEST, "Menu path is required"));
    }
    if system_repo::menu_name_exists(&state.pool, None, &payload.name)
        .await
        .map_err(internal_error)?
    {
        return Err((StatusCode::CONFLICT, "Menu name already exists"));
    }
    if !payload.path.is_empty()
        && system_repo::menu_path_exists(&state.pool, None, &payload.path)
            .await
            .map_err(internal_error)?
    {
        return Err((StatusCode::CONFLICT, "Menu path already exists"));
    }

    let mut meta = payload.meta;
    if payload.active_path.is_some() {
        meta.active_path = payload.active_path;
    }
    let menu = SystemMenu {
        auth_code: payload.auth_code,
        children: None,
        component: payload.component,
        id: generate_id("menu"),
        meta,
        name: payload.name,
        path: payload.path,
        pid: payload.pid.unwrap_or_else(|| "0".to_string()),
        redirect: payload.redirect,
        status: payload.status,
        menu_type: payload.menu_type,
    };

    system_repo::insert_menu(&state.pool, &menu)
        .await
        .map_err(internal_error)
}

pub async fn update_menu(
    state: &AppState,
    id: &str,
    payload: UpdateMenuRequest,
) -> Result<SystemMenu, (StatusCode, &'static str)> {
    let mut menu = system_repo::get_menu(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Menu not found"))?;

    if let Some(name) = payload.name.as_ref() {
        if system_repo::menu_name_exists(&state.pool, Some(id), name)
            .await
            .map_err(internal_error)?
        {
            return Err((StatusCode::CONFLICT, "Menu name already exists"));
        }
    }
    if let Some(path) = payload.path.as_ref() {
        if !path.is_empty()
            && system_repo::menu_path_exists(&state.pool, Some(id), path)
                .await
                .map_err(internal_error)?
        {
            return Err((StatusCode::CONFLICT, "Menu path already exists"));
        }
    }

    if let Some(name) = payload.name {
        menu.name = name;
    }
    if let Some(path) = payload.path {
        menu.path = path;
    }
    if let Some(pid) = payload.pid {
        menu.pid = pid;
    }
    if let Some(component) = payload.component {
        menu.component = Some(component);
    }
    if let Some(auth_code) = payload.auth_code {
        menu.auth_code = Some(auth_code);
    }
    if let Some(redirect) = payload.redirect {
        menu.redirect = Some(redirect);
    }
    if let Some(status) = payload.status {
        menu.status = status;
    }
    if let Some(menu_type) = payload.menu_type {
        menu.menu_type = menu_type;
    }
    menu.meta = payload.meta;
    if payload.active_path.is_some() {
        menu.meta.active_path = payload.active_path;
    }

    system_repo::update_menu(&state.pool, &menu)
        .await
        .map_err(internal_error)
}

pub async fn delete_menu(
    state: &AppState,
    id: &str,
) -> Result<SystemMenu, (StatusCode, &'static str)> {
    let menus = system_repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;
    let removed = menus
        .iter()
        .find(|menu| menu.id == id)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Menu not found"))?;

    let mut ids = vec![removed.id.clone()];
    ids.extend(collect_menu_descendants(&menus, &removed.id));

    system_repo::delete_menus(&state.pool, &ids)
        .await
        .map_err(internal_error)?;

    let roles = system_repo::list_all_roles(&state.pool)
        .await
        .map_err(internal_error)?;
    for mut role in roles {
        let original_len = role.permissions.len();
        role.permissions
            .retain(|permission| !ids.iter().any(|id| id == permission));
        if role.permissions.len() != original_len {
            system_repo::update_role(&state.pool, &role)
                .await
                .map_err(internal_error)?;
        }
    }

    Ok(removed)
}

pub async fn list_depts(state: &AppState) -> Result<DeptListResponse, (StatusCode, &'static str)> {
    let depts = system_repo::list_all_depts(&state.pool)
        .await
        .map_err(internal_error)?;
    Ok(build_dept_tree(&depts))
}

pub async fn create_dept(
    state: &AppState,
    payload: CreateDeptRequest,
) -> Result<SystemDept, (StatusCode, &'static str)> {
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Dept name is required"));
    }

    let dept = SystemDept {
        children: None,
        create_time: String::new(),
        id: generate_id("dept"),
        name: payload.name,
        pid: payload.pid.unwrap_or_else(|| "0".to_string()),
        remark: payload.remark.unwrap_or_default(),
        status: payload.status,
    };

    system_repo::insert_dept(&state.pool, &dept)
        .await
        .map_err(internal_error)
}

pub async fn update_dept(
    state: &AppState,
    id: &str,
    payload: UpdateDeptRequest,
) -> Result<SystemDept, (StatusCode, &'static str)> {
    let mut dept = system_repo::get_dept(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Dept not found"))?;

    if let Some(name) = payload.name {
        dept.name = name;
    }
    if let Some(pid) = payload.pid {
        dept.pid = pid;
    }
    if let Some(remark) = payload.remark {
        dept.remark = remark;
    }
    if let Some(status) = payload.status {
        dept.status = status;
    }

    system_repo::update_dept(&state.pool, &dept)
        .await
        .map_err(internal_error)
}

pub async fn delete_dept(
    state: &AppState,
    id: &str,
) -> Result<SystemDept, (StatusCode, &'static str)> {
    system_repo::delete_dept(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Dept not found"))
}

pub async fn list_users(
    state: &AppState,
    query: UserListQuery,
) -> Result<UserListResponse, (StatusCode, &'static str)> {
    let (items, total) = user_repo::list_users(&state.pool, &query)
        .await
        .map_err(internal_error)?;
    Ok(UserListResponse { items, total })
}

pub async fn create_user(
    state: &AppState,
    payload: CreateUserRequest,
) -> Result<SystemUser, (StatusCode, &'static str)> {
    validate_user_payload(
        state,
        &payload.username,
        &payload.password,
        &payload.real_name,
        payload.dept_id.as_deref(),
        &payload.role_ids,
    )
    .await?;

    let user = UserRecord {
        avatar: payload.avatar.unwrap_or_default(),
        dept_id: payload.dept_id.filter(|value| !value.trim().is_empty()),
        desc: String::new(),
        email: payload.email.unwrap_or_default(),
        home_path: payload.home_path.unwrap_or_else(|| "/analytics".to_string()),
        id: generate_id("user"),
        password: payload.password,
        phone: payload.phone.unwrap_or_default(),
        real_name: payload.real_name,
        remark: payload.remark.unwrap_or_default(),
        role_ids: payload.role_ids,
        status: payload.status,
        timezone: "Asia/Shanghai".to_string(),
        username: payload.username,
    };

    user_repo::insert_user(&state.pool, &user)
        .await
        .map_err(map_user_write_error)
}

pub async fn update_user(
    state: &AppState,
    id: &str,
    payload: UpdateUserRequest,
) -> Result<SystemUser, (StatusCode, &'static str)> {
    let mut user = user_repo::get_user(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))?;
    let auth_user = user_repo::get_auth_user_by_username(&state.pool, &user.username)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))?;

    let next_username = payload.username.unwrap_or(user.username.clone());
    let next_password = payload.password.unwrap_or(auth_user.password);
    let next_real_name = payload.real_name.unwrap_or(user.real_name.clone());
    let next_dept_id = payload
        .dept_id
        .unwrap_or_else(|| user.dept_id.clone().unwrap_or_default());
    let next_role_ids = payload.role_ids.unwrap_or(user.role_ids.clone());

    validate_user_payload(
        state,
        &next_username,
        &next_password,
        &next_real_name,
        if next_dept_id.trim().is_empty() {
            None
        } else {
            Some(next_dept_id.as_str())
        },
        &next_role_ids,
    )
    .await?;

    user.avatar = payload.avatar.unwrap_or(user.avatar);
    user.dept_id = if next_dept_id.trim().is_empty() {
        None
    } else {
        Some(next_dept_id)
    };
    user.email = payload.email.unwrap_or(user.email);
    user.home_path = payload.home_path.unwrap_or(user.home_path);
    user.phone = payload.phone.unwrap_or(user.phone);
    user.real_name = next_real_name;
    user.remark = payload.remark.unwrap_or(user.remark);
    user.role_ids = next_role_ids;
    user.status = payload.status.unwrap_or(user.status);
    user.username = next_username;

    let record = UserRecord {
        avatar: user.avatar,
        dept_id: user.dept_id,
        desc: user.desc,
        email: user.email,
        home_path: user.home_path,
        id: user.id,
        password: next_password,
        phone: user.phone,
        real_name: user.real_name,
        remark: user.remark,
        role_ids: user.role_ids,
        status: user.status,
        timezone: auth_user.timezone,
        username: user.username,
    };

    user_repo::update_user(&state.pool, &record)
        .await
        .map_err(map_user_write_error)
}

pub async fn delete_user(
    state: &AppState,
    id: &str,
) -> Result<SystemUser, (StatusCode, &'static str)> {
    user_repo::delete_user(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "User not found"))
}

pub async fn navigation_menus(
    state: &AppState,
) -> Result<Vec<SystemMenu>, (StatusCode, &'static str)> {
    let menus = system_repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;
    Ok(build_menu_tree(
        &menus
            .into_iter()
            .filter(|menu| menu.menu_type != "button")
            .collect::<Vec<_>>(),
    ))
}

pub async fn navigation_menus_for_user(
    state: &AppState,
    user_id: &str,
) -> Result<Vec<SystemMenu>, (StatusCode, &'static str)> {
    let allowed_ids = user_repo::list_user_permission_ids(&state.pool, user_id)
        .await
        .map_err(internal_error)?;
    if allowed_ids.is_empty() {
        return Ok(Vec::new());
    }

    let menus = system_repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(build_filtered_menu_tree(
        &menus
            .into_iter()
            .filter(|menu| menu.menu_type != "button")
            .collect::<Vec<_>>(),
        &allowed_ids,
    ))
}

fn build_menu_tree(flat: &[SystemMenu]) -> Vec<SystemMenu> {
    build_menu_branch(flat, "0")
}

fn build_menu_branch(flat: &[SystemMenu], pid: &str) -> Vec<SystemMenu> {
    let mut nodes = flat
        .iter()
        .filter(|menu| menu.pid == pid)
        .cloned()
        .collect::<Vec<_>>();

    nodes.sort_by_key(|menu| menu.meta.order.unwrap_or_default());

    for node in &mut nodes {
        let children = build_menu_branch(flat, &node.id);
        node.children = if children.is_empty() {
            None
        } else {
            Some(children)
        };
    }

    nodes
}

fn build_dept_tree(flat: &[SystemDept]) -> Vec<SystemDept> {
    build_dept_branch(flat, "0")
}

fn build_dept_branch(flat: &[SystemDept], pid: &str) -> Vec<SystemDept> {
    let mut nodes = flat
        .iter()
        .filter(|dept| dept.pid == pid)
        .cloned()
        .collect::<Vec<_>>();

    for node in &mut nodes {
        let children = build_dept_branch(flat, &node.id);
        node.children = if children.is_empty() {
            None
        } else {
            Some(children)
        };
    }

    nodes
}

fn collect_menu_descendants(flat: &[SystemMenu], parent_id: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let direct = flat
        .iter()
        .filter(|menu| menu.pid == parent_id)
        .map(|menu| menu.id.clone())
        .collect::<Vec<_>>();
    for id in direct {
        ids.push(id.clone());
        ids.extend(collect_menu_descendants(flat, &id));
    }
    ids
}

fn build_filtered_menu_tree(flat: &[SystemMenu], allowed_ids: &[String]) -> Vec<SystemMenu> {
    build_filtered_menu_branch(flat, "0", allowed_ids)
}

fn build_filtered_menu_branch(flat: &[SystemMenu], pid: &str, allowed_ids: &[String]) -> Vec<SystemMenu> {
    let mut nodes = Vec::new();

    for mut menu in flat.iter().filter(|menu| menu.pid == pid).cloned() {
        let children = build_filtered_menu_branch(flat, &menu.id, allowed_ids);
        let is_allowed = allowed_ids.iter().any(|id| id == &menu.id);

        if is_allowed || !children.is_empty() {
            menu.children = if children.is_empty() { None } else { Some(children) };
            nodes.push(menu);
        }
    }

    nodes.sort_by_key(|menu| menu.meta.order.unwrap_or_default());
    nodes
}

fn generate_id(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}-{nanos}")
}

async fn validate_user_payload(
    state: &AppState,
    username: &str,
    password: &str,
    real_name: &str,
    dept_id: Option<&str>,
    role_ids: &[String],
) -> Result<(), (StatusCode, &'static str)> {
    if username.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Username is required"));
    }
    if password.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Password is required"));
    }
    if real_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Real name is required"));
    }
    if role_ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "At least one role is required"));
    }
    if let Some(dept_id) = dept_id {
        if system_repo::get_dept(&state.pool, dept_id)
            .await
            .map_err(internal_error)?
            .is_none()
        {
            return Err((StatusCode::BAD_REQUEST, "Dept not found"));
        }
    }

    for role_id in role_ids {
        if system_repo::get_role(&state.pool, role_id)
            .await
            .map_err(internal_error)?
            .is_none()
        {
            return Err((StatusCode::BAD_REQUEST, "Role not found"));
        }
    }

    Ok(())
}

fn map_role_write_error(error: sqlx::Error) -> (StatusCode, &'static str) {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.is_unique_violation() {
            return (StatusCode::CONFLICT, "Role name already exists");
        }
    }
    internal_error(error)
}

fn map_user_write_error(error: sqlx::Error) -> (StatusCode, &'static str) {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.is_unique_violation() {
            if let Some(constraint) = db_error.constraint() {
                return match constraint {
                    "admin_users_username_key" => (StatusCode::CONFLICT, "Username already exists"),
                    "idx_admin_users_email_unique" => {
                        (StatusCode::CONFLICT, "Email already exists")
                    }
                    "idx_admin_users_phone_unique" => {
                        (StatusCode::CONFLICT, "Phone already exists")
                    }
                    _ => (StatusCode::CONFLICT, "User already exists"),
                };
            }
            return (StatusCode::CONFLICT, "User already exists");
        }
    }

    internal_error(error)
}

fn internal_error(_error: sqlx::Error) -> (StatusCode, &'static str) {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
}
