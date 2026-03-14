from ytmusicapi import YTMusic


class YouTubeService:
    def __init__(self):
        self.ytm = YTMusic()

    def search(self, query: str, limit: int = 5) -> list:
        try:
            # Forzamos filtro "songs" para evitar ruido de videos/álbumes
            raw = self.ytm.search(query, filter="songs", limit=limit)

            # Limpiamos y cortamos la lista a 'limit' por si la API devuelve de más
            results = []
            for r in raw[:limit]:
                results.append({
                    "title": r.get("title"),
                    "artist": r.get("artists", [{}])[0].get("name") if r.get("artists") else "Unknown",
                    "album": r.get("album", {}).get("name") if r.get("album") else "Single",
                    "id": r.get("videoId"),
                    "duration": r.get("duration"),
                    "thumbnail": r.get("thumbnails", [{}])[-1].get("url") if r.get("thumbnails") else ""
                })
            return results
        except Exception as e:
            # Log de error técnico sin adornos
            print(f"[YouTubeService] Error: {e}")
            return []

    def get_video(self, video_id: str) -> list:
        try:
            # get_song requiere el ID puro, no una URL.
            song = self.ytm.get_song(video_id)

            # get_song devuelve un diccionario complejo. La data útil suele estar en videoDetails.
            details = song.get("videoDetails")
            if not details:
                return []

            # Mapeo explícito para cumplir con tu struct Track / TrackDto en Rust.
            track = {
                "title": details.get("title"),
                "artist": details.get("author", "Unknown"),
                # ytmusicapi en get_song no siempre devuelve el álbum en la raíz igual que en search
                "album": "Single",
                "id": details.get("videoId", video_id),
                "duration": details.get("lengthSeconds"),
                "thumbnail": details.get("thumbnail", {}).get("thumbnails", [{}])[-1].get("url") if details.get(
                    "thumbnail") else ""
            }

            # Retornamos obligatoriamente una lista de un solo elemento.
            return [track]

        except Exception as e:
            print(f"[YouTubeService] Error en get_video para ID {video_id}: {e}")
            return []