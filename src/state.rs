use sqlx::PgPool;

use crate::db::{create_pool, run_migrations};

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn from_database_url(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = create_pool(database_url).await?;
        run_migrations(&pool)
            .await
            .map_err(|error| sqlx::Error::Migrate(Box::new(error)))?;
        Ok(Self::new(pool))
    }
}
