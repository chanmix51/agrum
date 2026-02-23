use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub async fn get_pool() -> Pool<PostgresConnectionManager<NoTls>> {
    // Load .env if present; existing env vars override .env values
    let _ = dotenvy::dotenv();
    let pg_dsn = match std::env::var("PG_DSN").ok().filter(|s| !s.is_empty()) {
        Some(dsn) => dsn,
        None => panic!("PG_DSN is not set (set it in the environment or in a .env file)"),
    };
    let pg_mgr =
        PostgresConnectionManager::new_from_stringlike(pg_dsn, tokio_postgres::NoTls).unwrap();

    Pool::builder().build(pg_mgr).await.unwrap()
}
