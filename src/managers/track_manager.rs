use crate::model::Track;
use crate::repository::TrackRepository;
use crate::services::MetadataClient;
use crate::services::{DownloadService, DownloadError};

#[derive(Debug)]
pub enum TrackManagerError {
    MetadataError(String),
    NoResults,
    DownloadError(DownloadError),
    DatabaseError(sqlx::Error),
}

impl std::fmt::Display for TrackManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TrackManagerError::MetadataError(e) => write!(f, "Metadata error: {}", e),
            TrackManagerError::NoResults        => write!(f, "No results found for query"),
            TrackManagerError::DownloadError(e) => write!(f, "Download error: {}", e),
            TrackManagerError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for TrackManagerError {}

impl From<sqlx::Error> for TrackManagerError {
    fn from(e: sqlx::Error) -> Self { TrackManagerError::DatabaseError(e) }
}

impl From<DownloadError> for TrackManagerError {
    fn from(e: DownloadError) -> Self { TrackManagerError::DownloadError(e) }
}

#[derive(Clone)]
pub struct TrackManager {
    metadata:   MetadataClient,
    repo:       TrackRepository,
    downloader: DownloadService,
}

impl TrackManager {
    pub fn new(metadata: MetadataClient, repo: TrackRepository, downloader: DownloadService) -> Self {
        Self { metadata, repo, downloader }
    }

    pub async fn resolve(&self, query: &str) -> Result<Track, TrackManagerError> {
        self.resolve_with_status(query, |_| {}).await
    }

    pub async fn resolve_with_status(
        &self,
        query: &str,
        on_status: impl Fn(String) + Send + 'static + Clone,
    ) -> Result<Track, TrackManagerError> {
        let query = query.trim();

        // ── 1. ¿Es un ID directo o un link? ──────────────────────────────────
        if let Some(id) = extract_video_id(query) {
            // ── 2. Cache hit → devolver directo ──────────────────────────────
            if let Some(cached) = self.repo.get_by_id(&id).await? {
                if cached.path.is_some() {
                    return Ok(cached);
                }
            }
            // ── 3. No está en DB → buscar metadata por ID y descargar ────────
            let track = self.fetch_by_id(&id).await?;
            return self.download_and_save(track, on_status).await;
        }

        // ── 4. Búsqueda por texto → metadata primero ──────────────────────────
        let track = self.fetch_first_result(query).await?;

        // ── 5. Cache hit tras resolver el ID real ─────────────────────────────
        if let Some(cached) = self.repo.get_by_id(&track.id).await? {
            if cached.path.is_some() {
                return Ok(cached);
            }
        }

        // ── 6. No existe → descargar ──────────────────────────────────────────
        self.download_and_save(track, on_status).await
    }

    // ─── Públicos ─────────────────────────────────────────────────────────────

    pub async fn fetch_all_result(&self, query: &str) -> Result<Vec<Track>, TrackManagerError> {
        self.metadata
            .call("search", query)
            .await
            .map_err(|e| TrackManagerError::MetadataError(e.to_string()))
    }

    // ─── Internos ─────────────────────────────────────────────────────────────

    async fn fetch_first_result(&self, query: &str) -> Result<Track, TrackManagerError> {
        let mut results = self.metadata
            .call("search", query)
            .await
            .map_err(|e| TrackManagerError::MetadataError(e.to_string()))?;

        results.drain(..).next().ok_or(TrackManagerError::NoResults)
    }

    async fn fetch_by_id(&self, id: &str) -> Result<Track, TrackManagerError> {
        let mut results = self.metadata
            .call("video", id)
            .await
            .map_err(|e| TrackManagerError::MetadataError(e.to_string()))?;

        results.drain(..).next().ok_or(TrackManagerError::NoResults)
    }
    /// Descarga un track y lo persiste en la base de datos.
    async fn download_and_save(
        &self,
        track: Track,
        on_status: impl Fn(String) + Send + 'static,
    ) -> Result<Track, TrackManagerError> {
        let path  = self.downloader.download(&track, on_status).await?;
        let track = Track { path: Some(path), ..track };
        self.repo.insert(&track).await?;
        Ok(track)
    }
}

// ─── HELPERS ─────────────────────────────────────────────────────────────────

fn extract_video_id(query: &str) -> Option<String> {
    // ID directo: exactamente 11 caracteres alfanuméricos + _ -
    if query.len() == 11 && query.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Some(query.to_string());
    }

    // Link youtube.com/watch?v=
    if let Some(pos) = query.find("v=") {
        let id = &query[pos + 2..];
        let id = id.split('&').next().unwrap_or("");
        if id.len() == 11 {
            return Some(id.to_string());
        }
    }

    // Link youtu.be/
    if let Some(pos) = query.find("youtu.be/") {
        let id = &query[pos + 9..];
        let id = id.split('?').next().unwrap_or("");
        if id.len() == 11 {
            return Some(id.to_string());
        }
    }

    None
}