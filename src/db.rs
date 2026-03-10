use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};
use url::Url;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
const DEFAULT_SCHEMA: &str = "rust_vben";

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let database_url = scoped_database_url(database_url)?;
    let schema_name = schema_name_from_database_url(&database_url)?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    ensure_schema_exists(&pool, &schema_name).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}

pub fn database_url_from_env() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres123@localhost:5432/ministore_sqlx_prepare".to_string()
    })
}

fn scoped_database_url(database_url: &str) -> Result<String, sqlx::Error> {
    let mut url = parse_database_url(database_url)?;
    let query_pairs = url.query_pairs().into_owned().collect::<Vec<_>>();
    let mut rewritten_pairs = Vec::with_capacity(query_pairs.len() + 1);
    let mut has_options = false;
    let mut has_search_path = false;

    for (key, mut value) in query_pairs {
        if key == "options" {
            has_options = true;
            if value.contains("search_path=") {
                has_search_path = true;
            } else {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(&format!("-csearch_path={DEFAULT_SCHEMA}"));
            }
        }
        rewritten_pairs.push((key, value));
    }

    if !has_options {
        rewritten_pairs.push((
            "options".to_string(),
            format!("-csearch_path={DEFAULT_SCHEMA}"),
        ));
    } else if has_search_path {
        return Ok(database_url.to_string());
    }

    url.query_pairs_mut().clear().extend_pairs(rewritten_pairs);
    Ok(url.into())
}

fn schema_name_from_database_url(database_url: &str) -> Result<String, sqlx::Error> {
    let url = parse_database_url(database_url)?;

    for (key, value) in url.query_pairs() {
        if key != "options" {
            continue;
        }

        for option in value.split_whitespace() {
            if let Some(search_path) = option.strip_prefix("-csearch_path=") {
                let schema_name = search_path.split(',').next().unwrap_or(DEFAULT_SCHEMA).trim();
                validate_schema_name(schema_name)?;
                return Ok(schema_name.to_string());
            }
        }
    }

    Ok(DEFAULT_SCHEMA.to_string())
}

async fn ensure_schema_exists(pool: &PgPool, schema_name: &str) -> Result<(), sqlx::Error> {
    validate_schema_name(schema_name)?;
    sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS \"{schema_name}\""))
        .execute(pool)
        .await?;
    Ok(())
}

fn validate_schema_name(schema_name: &str) -> Result<(), sqlx::Error> {
    let is_valid = !schema_name.is_empty()
        && schema_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');

    if is_valid {
        Ok(())
    } else {
        Err(sqlx::Error::Protocol(
            format!("invalid postgres schema name: {schema_name}").into(),
        ))
    }
}

fn parse_database_url(database_url: &str) -> Result<Url, sqlx::Error> {
    Url::parse(database_url)
        .map_err(|error| sqlx::Error::Configuration(Box::new(error)))
}
