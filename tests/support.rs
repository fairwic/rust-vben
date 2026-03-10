use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::PgPool;

static SCHEMA_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TestDatabase {
    admin_pool: PgPool,
    schema_name: String,
    schema_url: String,
}

impl TestDatabase {
    pub async fn new() -> Self {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres123@localhost:5432/ministore_sqlx_prepare".to_string()
        });
        let admin_pool = PgPool::connect(&base_url)
            .await
            .expect("connect postgres");
        let schema_name = format!(
            "test_{}_{}_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_nanos(),
            std::process::id(),
            SCHEMA_COUNTER.fetch_add(1, Ordering::Relaxed)
        );

        sqlx::query(&format!("CREATE SCHEMA \"{schema_name}\""))
            .execute(&admin_pool)
            .await
            .expect("create schema");

        let schema_url = format!("{base_url}?options=-csearch_path%3D{schema_name}");

        Self {
            admin_pool,
            schema_name,
            schema_url,
        }
    }

    pub async fn app(&self) -> axum::Router {
        rust_vben::build_app_with_database_url(&self.schema_url)
            .await
            .expect("build app")
    }

    pub async fn cleanup(&self) {
        sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", self.schema_name))
            .execute(&self.admin_pool)
            .await
            .expect("drop schema");
    }
}
