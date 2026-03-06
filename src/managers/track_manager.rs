use crate::model::Track;
use crate::repository::TrackRepository;
use crate::services::MetadataClient;
use crate::services::{DownloadService, DownloadError};

/**
 * =====================================================
 * DISPLAY Y ERRORES.
 * Implementación de Display para TrackManagerError
 * Implementación de Error y adaptamos el sqlx::Error
 * y DownloadError para qué se encargue nuestro enum
 * ======================================================
 */
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
            TrackManagerError::MetadataError(e)  => write!(f, "Metadata error: {}", e),
            TrackManagerError::NoResults         => write!(f, "No results found for query"),
            TrackManagerError::DownloadError(e)  => write!(f, "Download error: {}", e),
            TrackManagerError::DatabaseError(e)  => write!(f, "Database error: {}", e),
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

/**
 *  ======================================
 *  Struct del TrackManager y sus métodos
 *  ======================================
 */
pub struct TrackManager {
    metadata: MetadataClient,
    repo: TrackRepository,
    downloader: DownloadService,
}

impl TrackManager {
    pub fn new(metadata: MetadataClient, repo: TrackRepository, downloader: DownloadService) -> Self {
        Self { metadata, repo, downloader }
    }

    /**
     *  Conversión de una Query en Track
     *
     *  Esta función recibe un &str qué es una petición de una canción a un usuario
     *  Pasa por 4 puntos importantes
     *  1.- sí es un ID o un link lo busca en la base de datos directamente,
     *  2.- si se encuentra se construye el objeto Track y se retorna
     *  3.- sino, busca el nombre de la canción mediante un microservicio
     *  4.- gracias a la busqueda, encuentra el ID y busca en la db
     *  Si no existe, entonces se carga.
     * # Errors
     *
     * Devuelve [`TrackManagerError`] si falla la búsqueda, la base de datos
     * o el proceso de descarga.
     *
     * # Examples
     *
     * ```ignore
     * let track = track_manager.resolve("Terapia Lazaro Suplika Dopamina").await?;
     * println!("{}", track.title);
     * ```
     */
    pub async fn resolve(&self, query: &str) -> Result<Track, TrackManagerError> {
        self.resolve_with_status(query, |_| {}).await
    }

    pub async fn resolve_with_status(
        &self,
        query: &str,
        on_status: impl Fn(String) + Send + 'static + Clone,
    ) -> Result<Track, TrackManagerError> {
        let query = query.trim();

        // ── 1. ¿Es un ID directo o un link? → intentar cache hit rápido ──────
        if let Some(id) = extract_video_id(query) {
            if let Some(cached) = self.repo.get_by_id(&id).await? {
                if cached.path.is_some() {
                    return Ok(cached);
                }
            }
            return self.download_and_save_by_id(&id, None, on_status).await;
        }

        // ── 2. Búsqueda por texto → metadata primero ──────────────────────────
        let track = self.fetch_first_result(query).await?;

        // ── 3. Cache hit tras resolver el ID real ─────────────────────────────
        if let Some(cached) = self.repo.get_by_id(&track.id).await? {
            if cached.path.is_some() {
                return Ok(cached);
            }
        }

        // ── 4. No existe → descargar ──────────────────────────────────────────
        self.download_and_save_by_id(&track.id.clone(), Some(track), on_status).await
    }

    /**
     *  ==================
     *  FUNCIONES INTERNA
     *  ==================
     */

    /**
     *  Devuelve el primer resultado de la búsqueda del microservicio
     *
     *  Dado que el metodo [`metada::call`] devuelve una lista de n resultados
     *  esta función sirve para extraer únicamente el primer elemento
     *
     * # Examples
     *
     * ```ignore
     * let track = track_manager.resolve("Terapia Lazaro Suplika Dopamina").await?;
     * println!("{}", track.title);
     * ```
     */
    async fn fetch_first_result(&self, query: &str) -> Result<Track, TrackManagerError> {
        let mut results = self.metadata
            .call("search", query)
            .await
            .map_err(|e| TrackManagerError::MetadataError(e.to_string()))?;

        results.drain(..).next().ok_or(TrackManagerError::NoResults)
    }

    /**
     *  Descarga y guarda una canción por id
     *
     *  Llamada a [`fetch_first_result`] si no se le pasó la canción se descarga
     *  y finalmente se guarda en la base de datos.
     */
    async fn download_and_save_by_id(
        &self,
        id: &str,
        track: Option<Track>,
        on_status: impl Fn(String) + Send + 'static,
    ) -> Result<Track, TrackManagerError> {
        let track = match track {
            Some(t) => t,
            None => self.fetch_first_result(id).await?,
        };

        let path = self.downloader.download(&track, on_status).await?;

        let track = Track { path: Some(path), ..track };
        self.repo.insert(&track).await?;

        Ok(track)
    }
}

// ─── HELPERS ─────────────────────────────────────────────────────────────────

/// Extrae el video ID si la query es un ID de 11 chars o un link de YouTube.
/// Devuelve None si es una búsqueda de texto libre.
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