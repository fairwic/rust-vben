use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

use crate::{
    models::{
        auth::AuthUserRecord,
        system::{SystemUser, UserListQuery, UserRecord},
    },
};

#[derive(Debug, FromRow)]
struct UserRow {
    avatar: String,
    create_time: String,
    dept_id: Option<String>,
    dept_name: Option<String>,
    email: String,
    home_path: String,
    id: String,
    phone: String,
    real_name: String,
    remark: String,
    role_ids: Vec<String>,
    role_names: Vec<String>,
    status: i16,
    username: String,
}

#[derive(Debug, FromRow)]
struct AuthUserRow {
    avatar: String,
    home_path: String,
    id: String,
    password: String,
    real_name: String,
    role_ids: Vec<String>,
    status: i16,
    username: String,
}

impl From<UserRow> for SystemUser {
    fn from(row: UserRow) -> Self {
        Self {
            avatar: row.avatar,
            create_time: row.create_time,
            dept_id: row.dept_id,
            dept_name: row.dept_name,
            email: row.email,
            home_path: row.home_path,
            id: row.id,
            phone: row.phone,
            real_name: row.real_name,
            remark: row.remark,
            role_ids: row.role_ids,
            role_names: row.role_names,
            status: i32::from(row.status),
            username: row.username,
        }
    }
}

impl From<AuthUserRow> for AuthUserRecord {
    fn from(row: AuthUserRow) -> Self {
        Self {
            avatar: row.avatar,
            home_path: row.home_path,
            id: row.id.clone(),
            password: row.password,
            real_name: row.real_name,
            role_ids: row.role_ids,
            status: i32::from(row.status),
            user_id: row.id,
            username: row.username,
        }
    }
}

pub async fn list_users(
    pool: &PgPool,
    query: &UserListQuery,
) -> Result<(Vec<SystemUser>, usize), sqlx::Error> {
    let total = build_user_count_query(query)
        .build_query_scalar::<i64>()
        .fetch_one(pool)
        .await? as usize;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).max(1);
    let offset = ((page - 1) * page_size) as i64;

    let rows = build_user_select_query(query)
        .push(" GROUP BY u.id, d.name ORDER BY u.created_at ASC LIMIT ")
        .push_bind(page_size as i64)
        .push(" OFFSET ")
        .push_bind(offset)
        .build_query_as::<UserRow>()
        .fetch_all(pool)
        .await?;

    Ok((rows.into_iter().map(Into::into).collect(), total))
}

pub async fn get_user(pool: &PgPool, id: &str) -> Result<Option<SystemUser>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT
            u.avatar,
            to_char(u.created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            u.dept_id,
            d.name AS dept_name,
            u.email,
            u.home_path,
            u.id,
            u.phone,
            u.real_name,
            u.remark,
            COALESCE(array_agg(DISTINCT r.id) FILTER (WHERE r.id IS NOT NULL), '{}'::text[]) AS role_ids,
            COALESCE(array_agg(DISTINCT r.name) FILTER (WHERE r.name IS NOT NULL), '{}'::text[]) AS role_names,
            u.status,
            u.username
        FROM admin_users u
        LEFT JOIN depts d ON d.id = u.dept_id
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE u.id = $1
        GROUP BY u.id, d.name
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn insert_user(pool: &PgPool, user: &UserRecord) -> Result<SystemUser, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO admin_users (
            id, username, password, real_name, avatar, home_path, email, phone, dept_id, remark, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.password)
    .bind(&user.real_name)
    .bind(&user.avatar)
    .bind(&user.home_path)
    .bind(&user.email)
    .bind(&user.phone)
    .bind(user.dept_id.as_deref())
    .bind(&user.remark)
    .bind(user.status as i16)
    .execute(&mut *tx)
    .await?;

    replace_user_roles(&mut tx, &user.id, &user.role_ids).await?;
    tx.commit().await?;

    get_user(pool, &user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_user(pool: &PgPool, user: &UserRecord) -> Result<SystemUser, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        UPDATE admin_users
        SET username = $2,
            password = $3,
            real_name = $4,
            avatar = $5,
            home_path = $6,
            email = $7,
            phone = $8,
            dept_id = $9,
            remark = $10,
            status = $11,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(&user.password)
    .bind(&user.real_name)
    .bind(&user.avatar)
    .bind(&user.home_path)
    .bind(&user.email)
    .bind(&user.phone)
    .bind(user.dept_id.as_deref())
    .bind(&user.remark)
    .bind(user.status as i16)
    .execute(&mut *tx)
    .await?;

    replace_user_roles(&mut tx, &user.id, &user.role_ids).await?;
    tx.commit().await?;

    get_user(pool, &user.id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn delete_user(pool: &PgPool, id: &str) -> Result<Option<SystemUser>, sqlx::Error> {
    let user = get_user(pool, id).await?;
    if user.is_none() {
        return Ok(None);
    }

    sqlx::query("DELETE FROM admin_users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(user)
}

pub async fn get_auth_user_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<AuthUserRecord>, sqlx::Error> {
    let row = sqlx::query_as::<_, AuthUserRow>(
        r#"
        SELECT
            u.avatar,
            u.home_path,
            u.id,
            u.password,
            u.real_name,
            COALESCE(array_agg(DISTINCT r.id) FILTER (WHERE r.id IS NOT NULL), '{}'::text[]) AS role_ids,
            u.status,
            u.username
        FROM admin_users u
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE u.username = $1
        GROUP BY u.id
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn list_user_permission_ids(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT jsonb_array_elements_text(r.permissions)
        FROM user_roles ur
        JOIN roles r ON r.id = ur.role_id
        WHERE ur.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn list_permission_auth_codes(
    pool: &PgPool,
    permission_ids: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    if permission_ids.is_empty() {
        return Ok(Vec::new());
    }

    sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT auth_code
        FROM menus
        WHERE id = ANY($1)
          AND auth_code IS NOT NULL
          AND auth_code <> ''
        ORDER BY auth_code ASC
        "#,
    )
    .bind(permission_ids)
    .fetch_all(pool)
    .await
}

async fn replace_user_roles(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    user_id: &str,
    role_ids: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

    if role_ids.is_empty() {
        return Ok(());
    }

    let mut builder = QueryBuilder::<Postgres>::new(
        "INSERT INTO user_roles (user_id, role_id) ",
    );
    builder.push_values(role_ids, |mut separated, role_id| {
        separated.push_bind(user_id).push_bind(role_id);
    });
    builder.build().execute(&mut **tx).await?;

    Ok(())
}

fn build_user_count_query(query: &UserListQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT COUNT(DISTINCT u.id)
        FROM admin_users u
        LEFT JOIN depts d ON d.id = u.dept_id
        WHERE 1 = 1
        "#,
    );
    apply_user_filters(&mut builder, query);
    builder
}

fn build_user_select_query(query: &UserListQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::<Postgres>::new(
        r#"
        SELECT
            u.avatar,
            to_char(u.created_at, 'YYYY/MM/DD HH24:MI:SS') AS create_time,
            u.dept_id,
            d.name AS dept_name,
            u.email,
            u.home_path,
            u.id,
            u.phone,
            u.real_name,
            u.remark,
            COALESCE(array_agg(DISTINCT r.id) FILTER (WHERE r.id IS NOT NULL), '{}'::text[]) AS role_ids,
            COALESCE(array_agg(DISTINCT r.name) FILTER (WHERE r.name IS NOT NULL), '{}'::text[]) AS role_names,
            u.status,
            u.username
        FROM admin_users u
        LEFT JOIN depts d ON d.id = u.dept_id
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE 1 = 1
        "#,
    );
    apply_user_filters(&mut builder, query);
    builder
}

fn apply_user_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, query: &'a UserListQuery) {
    if let Some(username) = query.username.as_ref() {
        builder
            .push(" AND u.username ILIKE ")
            .push_bind(format!("%{username}%"));
    }
    if let Some(real_name) = query.real_name.as_ref() {
        builder
            .push(" AND u.real_name ILIKE ")
            .push_bind(format!("%{real_name}%"));
    }
    if let Some(status) = query.status {
        builder.push(" AND u.status = ").push_bind(status as i16);
    }
    if let Some(dept_id) = query.dept_id.as_ref() {
        builder.push(" AND u.dept_id = ").push_bind(dept_id);
    }
    if let Some(email) = query.email.as_ref() {
        builder
            .push(" AND u.email ILIKE ")
            .push_bind(format!("%{email}%"));
    }
    if let Some(phone) = query.phone.as_ref() {
        builder
            .push(" AND u.phone ILIKE ")
            .push_bind(format!("%{phone}%"));
    }
    if let Some(start_time) = query.start_time.as_ref() {
        builder
            .push(" AND to_char(u.created_at, 'YYYY/MM/DD HH24:MI:SS') >= ")
            .push_bind(start_time);
    }
    if let Some(end_time) = query.end_time.as_ref() {
        builder
            .push(" AND to_char(u.created_at, 'YYYY/MM/DD HH24:MI:SS') <= ")
            .push_bind(end_time);
    }
}
