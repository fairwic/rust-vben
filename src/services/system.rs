use axum::http::StatusCode;

use crate::{
    models::system::{
        CreateDeptRequest, CreateMenuRequest, CreateRoleRequest, DeptListResponse, MenuListResponse,
        RoleListQuery, RoleListResponse, SystemDept, SystemMenu, SystemRole, UpdateDeptRequest,
        UpdateMenuRequest, UpdateRoleRequest,
    },
    repositories::system as repo,
    state::AppState,
};

pub async fn list_roles(
    state: &AppState,
    query: RoleListQuery,
) -> Result<RoleListResponse, (StatusCode, &'static str)> {
    let (items, total) = repo::list_roles(&state.pool, &query)
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

    repo::insert_role(&state.pool, &role)
        .await
        .map_err(map_role_write_error)
}

pub async fn update_role(
    state: &AppState,
    id: &str,
    payload: UpdateRoleRequest,
) -> Result<SystemRole, (StatusCode, &'static str)> {
    let mut role = repo::get_role(&state.pool, id)
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

    repo::update_role(&state.pool, &role)
        .await
        .map_err(map_role_write_error)
}

pub async fn delete_role(
    state: &AppState,
    id: &str,
) -> Result<SystemRole, (StatusCode, &'static str)> {
    repo::delete_role(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Role not found"))
}

pub async fn list_menus(state: &AppState) -> Result<MenuListResponse, (StatusCode, &'static str)> {
    let menus = repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;
    Ok(build_menu_tree(&menus))
}

pub async fn menu_name_exists(
    state: &AppState,
    current_id: Option<&str>,
    name: &str,
) -> Result<bool, (StatusCode, &'static str)> {
    repo::menu_name_exists(&state.pool, current_id, name)
        .await
        .map_err(internal_error)
}

pub async fn menu_path_exists(
    state: &AppState,
    current_id: Option<&str>,
    path: &str,
) -> Result<bool, (StatusCode, &'static str)> {
    repo::menu_path_exists(&state.pool, current_id, path)
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
    if repo::menu_name_exists(&state.pool, None, &payload.name)
        .await
        .map_err(internal_error)?
    {
        return Err((StatusCode::CONFLICT, "Menu name already exists"));
    }
    if !payload.path.is_empty()
        && repo::menu_path_exists(&state.pool, None, &payload.path)
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

    repo::insert_menu(&state.pool, &menu)
        .await
        .map_err(internal_error)
}

pub async fn update_menu(
    state: &AppState,
    id: &str,
    payload: UpdateMenuRequest,
) -> Result<SystemMenu, (StatusCode, &'static str)> {
    let mut menu = repo::get_menu(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Menu not found"))?;

    if let Some(name) = payload.name.as_ref() {
        if repo::menu_name_exists(&state.pool, Some(id), name)
            .await
            .map_err(internal_error)?
        {
            return Err((StatusCode::CONFLICT, "Menu name already exists"));
        }
    }
    if let Some(path) = payload.path.as_ref() {
        if !path.is_empty()
            && repo::menu_path_exists(&state.pool, Some(id), path)
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

    repo::update_menu(&state.pool, &menu)
        .await
        .map_err(internal_error)
}

pub async fn delete_menu(
    state: &AppState,
    id: &str,
) -> Result<SystemMenu, (StatusCode, &'static str)> {
    let menus = repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;
    let removed = menus
        .iter()
        .find(|menu| menu.id == id)
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "Menu not found"))?;

    let mut ids = vec![removed.id.clone()];
    ids.extend(collect_menu_descendants(&menus, &removed.id));

    repo::delete_menus(&state.pool, &ids)
        .await
        .map_err(internal_error)?;

    let roles = repo::list_all_roles(&state.pool)
        .await
        .map_err(internal_error)?;
    for mut role in roles {
        let original_len = role.permissions.len();
        role.permissions
            .retain(|permission| !ids.iter().any(|id| id == permission));
        if role.permissions.len() != original_len {
            repo::update_role(&state.pool, &role)
                .await
                .map_err(internal_error)?;
        }
    }

    Ok(removed)
}

pub async fn list_depts(state: &AppState) -> Result<DeptListResponse, (StatusCode, &'static str)> {
    let depts = repo::list_all_depts(&state.pool)
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

    repo::insert_dept(&state.pool, &dept)
        .await
        .map_err(internal_error)
}

pub async fn update_dept(
    state: &AppState,
    id: &str,
    payload: UpdateDeptRequest,
) -> Result<SystemDept, (StatusCode, &'static str)> {
    let mut dept = repo::get_dept(&state.pool, id)
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

    repo::update_dept(&state.pool, &dept)
        .await
        .map_err(internal_error)
}

pub async fn delete_dept(
    state: &AppState,
    id: &str,
) -> Result<SystemDept, (StatusCode, &'static str)> {
    repo::delete_dept(&state.pool, id)
        .await
        .map_err(internal_error)?
        .ok_or((StatusCode::NOT_FOUND, "Dept not found"))
}

pub async fn navigation_menus(
    state: &AppState,
) -> Result<Vec<SystemMenu>, (StatusCode, &'static str)> {
    let menus = repo::list_all_menus(&state.pool)
        .await
        .map_err(internal_error)?;
    Ok(build_menu_tree(
        &menus
            .into_iter()
            .filter(|menu| menu.menu_type != "button")
            .collect::<Vec<_>>(),
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

fn generate_id(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{prefix}-{nanos}")
}

fn map_role_write_error(error: sqlx::Error) -> (StatusCode, &'static str) {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.is_unique_violation() {
            return (StatusCode::CONFLICT, "Role name already exists");
        }
    }
    internal_error(error)
}

fn internal_error(_error: sqlx::Error) -> (StatusCode, &'static str) {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
}
