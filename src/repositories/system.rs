use sqlx::{
    types::Json,
    FromRow, PgPool, Postgres, QueryBuilder,
};

use crate::models::system::{
    MenuMeta, RoleListQuery, SystemDept, SystemMenu, SystemRole,
};

#[derive(Debug, FromRow)]
struct RoleRow {
    create_time: String,
    id: String,
    name: String,
    permissions: Json<Vec<String>>,
    remark: String,
    status: i16,
}

#[derive(Debug, FromRow)]
struct MenuRow {
    auth_code: Option<String>,
    component: Option<String>,
    id: String,
    meta: Json<MenuMeta>,
    name: String,
    path: String,
    pid: Option<String>,
    redirect: Option<String>,
    status: i16,
    menu_type: String,
}

#[derive(Debug, FromRow)]
struct DeptRow {
    create_time: String,
    id: String,
    name: String,
    pid: Option<String>,
    remark: String,
    status: i16,
}

impl From<RoleRow> for SystemRole {
    fn from(row: RoleRow) -> Self {
        Self {
            create_time: row.create_time,
            id: row.id,
            name: row.name,
            permissions: row.permissions.0,
            remark: row.remark,
            status: i32::from(row.status),
        }
    }
}

impl From<MenuRow> for SystemMenu {
    fn from(row: MenuRow) -> Self {
        Self {
            auth_code: row.auth_code,
            children: None,
            component: row.component,
            id: row.id,
            meta: row.meta.0,
            name: row.name,
            path: row.path,
            pid: row.pid.unwrap_or_else(|| "0".to_string()),
            redirect: row.redirect,
            status: i32::from(row.status),
            menu_type: row.menu_type,
        }
    }
}

impl From<DeptRow> for SystemDept {
    fn from(row: DeptRow) -> Self {
        Self {
            children: None,
            create_time: row.create_time,
            id: row.id,
            name: row.name,
            pid: row.pid.unwrap_or_else(|| "0".to_string()),
            remark: row.remark,
            status: i32::from(row.status),
        }
    }
}

pub async fn list_roles(
    pool: &PgPool,
    query: &RoleListQuery,
) -> Result<(Vec<SystemRole>, usize), sqlx::Error> {
    let total = build_role_count_query(query)
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await? as usize;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1);
    let offset = ((page - 1) * page_size) as i64;

    let rows = build_role_select_query(query)
        .push(" ORDER BY created_at ASC LIMIT ")
        .push_bind(page_size as i64)
        .push(" OFFSET ")
        .push_bind(offset)
        .build_query_as::<RoleRow>()
        .fetch_all(pool)
        .await?;

    Ok((rows.into_iter().map(Into::into).collect(), total))
}

pub async fn get_role(pool: &PgPool, id: &str) -> Result<Option<SystemRole>, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
        r#"
        SELECT
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            permissions,
            remark,
            status
        FROM roles
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn insert_role(pool: &PgPool, role: &SystemRole) -> Result<SystemRole, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
        r#"
        INSERT INTO roles (id, name, permissions, remark, status)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            permissions,
            remark,
            status
        "#,
    )
    .bind(&role.id)
    .bind(&role.name)
    .bind(Json(role.permissions.clone()))
    .bind(&role.remark)
    .bind(role.status as i16)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn update_role(pool: &PgPool, role: &SystemRole) -> Result<SystemRole, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
        r#"
        UPDATE roles
        SET name = $2,
            permissions = $3,
            remark = $4,
            status = $5,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            permissions,
            remark,
            status
        "#,
    )
    .bind(&role.id)
    .bind(&role.name)
    .bind(Json(role.permissions.clone()))
    .bind(&role.remark)
    .bind(role.status as i16)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn delete_role(pool: &PgPool, id: &str) -> Result<Option<SystemRole>, sqlx::Error> {
    let row = sqlx::query_as::<_, RoleRow>(
        r#"
        DELETE FROM roles
        WHERE id = $1
        RETURNING
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            permissions,
            remark,
            status
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn list_all_roles(pool: &PgPool) -> Result<Vec<SystemRole>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RoleRow>(
        r#"
        SELECT
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            permissions,
            remark,
            status
        FROM roles
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn list_all_menus(pool: &PgPool) -> Result<Vec<SystemMenu>, sqlx::Error> {
    let rows = sqlx::query_as::<_, MenuRow>(
        r#"
        SELECT
            auth_code,
            component,
            id,
            meta,
            name,
            path,
            pid,
            redirect,
            status,
            menu_type
        FROM menus
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_menu(pool: &PgPool, id: &str) -> Result<Option<SystemMenu>, sqlx::Error> {
    let row = sqlx::query_as::<_, MenuRow>(
        r#"
        SELECT
            auth_code,
            component,
            id,
            meta,
            name,
            path,
            pid,
            redirect,
            status,
            menu_type
        FROM menus
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn menu_name_exists(
    pool: &PgPool,
    current_id: Option<&str>,
    name: &str,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM menus
        WHERE name = $1
          AND ($2::text IS NULL OR id <> $2)
        "#,
    )
    .bind(name)
    .bind(current_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn menu_path_exists(
    pool: &PgPool,
    current_id: Option<&str>,
    path: &str,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM menus
        WHERE path = $1
          AND path <> ''
          AND ($2::text IS NULL OR id <> $2)
        "#,
    )
    .bind(path)
    .bind(current_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn insert_menu(pool: &PgPool, menu: &SystemMenu) -> Result<SystemMenu, sqlx::Error> {
    let row = sqlx::query_as::<_, MenuRow>(
        r#"
        INSERT INTO menus (id, pid, name, path, auth_code, component, redirect, menu_type, status, meta)
        VALUES ($1, NULLIF($2, '0'), $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            auth_code,
            component,
            id,
            meta,
            name,
            path,
            pid,
            redirect,
            status,
            menu_type
        "#,
    )
    .bind(&menu.id)
    .bind(&menu.pid)
    .bind(&menu.name)
    .bind(&menu.path)
    .bind(&menu.auth_code)
    .bind(&menu.component)
    .bind(&menu.redirect)
    .bind(&menu.menu_type)
    .bind(menu.status as i16)
    .bind(Json(menu.meta.clone()))
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn update_menu(pool: &PgPool, menu: &SystemMenu) -> Result<SystemMenu, sqlx::Error> {
    let row = sqlx::query_as::<_, MenuRow>(
        r#"
        UPDATE menus
        SET pid = NULLIF($2, '0'),
            name = $3,
            path = $4,
            auth_code = $5,
            component = $6,
            redirect = $7,
            menu_type = $8,
            status = $9,
            meta = $10,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            auth_code,
            component,
            id,
            meta,
            name,
            path,
            pid,
            redirect,
            status,
            menu_type
        "#,
    )
    .bind(&menu.id)
    .bind(&menu.pid)
    .bind(&menu.name)
    .bind(&menu.path)
    .bind(&menu.auth_code)
    .bind(&menu.component)
    .bind(&menu.redirect)
    .bind(&menu.menu_type)
    .bind(menu.status as i16)
    .bind(Json(menu.meta.clone()))
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn delete_menus(pool: &PgPool, ids: &[String]) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM menus WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await
        .map(|result| result.rows_affected())
}

pub async fn list_all_depts(pool: &PgPool) -> Result<Vec<SystemDept>, sqlx::Error> {
    let rows = sqlx::query_as::<_, DeptRow>(
        r#"
        SELECT
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            pid,
            remark,
            status
        FROM depts
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_dept(pool: &PgPool, id: &str) -> Result<Option<SystemDept>, sqlx::Error> {
    let row = sqlx::query_as::<_, DeptRow>(
        r#"
        SELECT
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            pid,
            remark,
            status
        FROM depts
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn insert_dept(pool: &PgPool, dept: &SystemDept) -> Result<SystemDept, sqlx::Error> {
    let row = sqlx::query_as::<_, DeptRow>(
        r#"
        INSERT INTO depts (id, pid, name, remark, status)
        VALUES ($1, NULLIF($2, '0'), $3, $4, $5)
        RETURNING
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            pid,
            remark,
            status
        "#,
    )
    .bind(&dept.id)
    .bind(&dept.pid)
    .bind(&dept.name)
    .bind(&dept.remark)
    .bind(dept.status as i16)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn update_dept(pool: &PgPool, dept: &SystemDept) -> Result<SystemDept, sqlx::Error> {
    let row = sqlx::query_as::<_, DeptRow>(
        r#"
        UPDATE depts
        SET pid = NULLIF($2, '0'),
            name = $3,
            remark = $4,
            status = $5,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            pid,
            remark,
            status
        "#,
    )
    .bind(&dept.id)
    .bind(&dept.pid)
    .bind(&dept.name)
    .bind(&dept.remark)
    .bind(dept.status as i16)
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn delete_dept(pool: &PgPool, id: &str) -> Result<Option<SystemDept>, sqlx::Error> {
    let row = sqlx::query_as::<_, DeptRow>(
        r#"
        DELETE FROM depts
        WHERE id = $1
        RETURNING
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            pid,
            remark,
            status
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

fn build_role_count_query(query: &RoleListQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM roles WHERE 1 = 1");
    apply_role_filters(&mut builder, query);
    builder
}

fn build_role_select_query(query: &RoleListQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            id,
            name,
            permissions,
            remark,
            status
        FROM roles
        WHERE 1 = 1
        "#,
    );
    apply_role_filters(&mut builder, query);
    builder
}

fn apply_role_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, query: &'a RoleListQuery) {
    if let Some(id) = query.id.as_ref() {
        builder.push(" AND id ILIKE ").push_bind(format!("%{id}%"));
    }
    if let Some(name) = query.name.as_ref() {
        builder
            .push(" AND name ILIKE ")
            .push_bind(format!("%{name}%"));
    }
    if let Some(status) = query.status {
        builder.push(" AND status = ").push_bind(status as i16);
    }
    if let Some(remark) = query.remark.as_ref() {
        builder
            .push(" AND remark ILIKE ")
            .push_bind(format!("%{remark}%"));
    }
    if let Some(start_time) = query.start_time.as_ref() {
        builder
            .push(" AND to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') >= ")
            .push_bind(start_time);
    }
    if let Some(end_time) = query.end_time.as_ref() {
        builder
            .push(" AND to_char(created_at, 'YYYY/MM/DD HH24:MI:SS') <= ")
            .push_bind(end_time);
    }
}
