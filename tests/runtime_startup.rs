use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{Executor, PgPool};
use url::Url;

static DATABASE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn base_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres123@localhost:5432/ministore_sqlx_prepare".to_string()
    })
}

fn maintenance_database_url(database_url: &str) -> String {
    let mut url = Url::parse(database_url).expect("parse database url");
    url.set_path("/postgres");
    url.set_query(None);
    url.to_string()
}

fn database_url_with_name(database_url: &str, database_name: &str) -> String {
    let mut url = Url::parse(database_url).expect("parse database url");
    url.set_path(&format!("/{database_name}"));
    url.set_query(None);
    url.to_string()
}

fn unique_database_name() -> String {
    format!(
        "startup_test_{}_{}_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos(),
        std::process::id(),
        DATABASE_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

async fn drop_database(admin_pool: &PgPool, database_name: &str) {
    sqlx::query(
        r#"
        SELECT pg_terminate_backend(pid)
        FROM pg_stat_activity
        WHERE datname = $1
          AND pid <> pg_backend_pid()
        "#,
    )
    .bind(database_name)
    .execute(admin_pool)
    .await
    .expect("terminate database connections");

    admin_pool
        .execute(&*format!("DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"))
        .await
        .expect("drop database");
}

#[tokio::test]
async fn build_app_should_ignore_unrelated_public_migration_history_by_using_app_schema() {
    let base_url = base_database_url();
    let admin_pool = PgPool::connect(&maintenance_database_url(&base_url))
        .await
        .expect("connect maintenance database");
    let database_name = unique_database_name();

    admin_pool
        .execute(&*format!("CREATE DATABASE \"{database_name}\""))
        .await
        .expect("create temporary database");

    let temporary_database_url = database_url_with_name(&base_url, &database_name);
    let polluted_pool = PgPool::connect(&temporary_database_url)
        .await
        .expect("connect temporary database");

    polluted_pool
        .execute(
            r#"
            CREATE TABLE _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
                success BOOLEAN NOT NULL,
                checksum BYTEA NOT NULL,
                execution_time BIGINT NOT NULL
            )
            "#,
        )
        .await
        .expect("create legacy migration table");

    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(20260303133000_i64)
    .bind("legacy")
    .bind(true)
    .bind(Vec::<u8>::new())
    .bind(0_i64)
    .execute(&polluted_pool)
    .await
    .expect("insert legacy migration row");

    let build_result = rust_vben::build_app_with_database_url(&temporary_database_url).await;
    let build_error = build_result.as_ref().err().map(ToString::to_string);

    drop(build_result);
    polluted_pool.close().await;
    drop_database(&admin_pool, &database_name).await;
    admin_pool.close().await;

    assert!(
        build_error.is_none(),
        "expected startup to avoid unrelated public migration history, got {build_error:?}"
    );
}
