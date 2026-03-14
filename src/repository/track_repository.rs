use sqlx::SqlitePool;
use crate::model::Track;

#[derive(Clone)]
pub struct TrackRepository {
    pool: SqlitePool,
}

impl TrackRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, track: &Track) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO tracks (id, title, artist, album, duration, thumbnail, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title      = excluded.title,
                artist     = excluded.artist,
                album      = excluded.album,
                duration = excluded.duration,
                thumbnail  = excluded.thumbnail,
                path       = COALESCE(excluded.path, tracks.path)",
        )
            .bind(&track.id)
            .bind(&track.title)
            .bind(&track.artist)
            .bind(&track.album)
            .bind(track.duration.to_string())
            .bind(&track.thumbnail)
            .bind(&track.path)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_all(&self) -> Result<Vec<Track>, sqlx::Error> {
        sqlx::query_as::<_, Track>(
            "SELECT id, title, artist, album, duration, thumbnail, path FROM tracks"
        )
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Track>, sqlx::Error> {
        sqlx::query_as::<_, Track>(
            "SELECT id, title, artist, album, duration, thumbnail, path
             FROM tracks
             WHERE id = ?1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
}