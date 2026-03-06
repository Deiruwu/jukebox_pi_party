use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
pub mod track_repository;
pub use crate::repository::track_repository::TrackRepository;

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// Inicializa la conexión y aplica migraciones. Solo debe llamarse UNA vez
    /// al arrancar la aplicación.
    pub async fn connect(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // Las migraciones son responsabilidad de la infraestructura, no de un repositorio
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }
}