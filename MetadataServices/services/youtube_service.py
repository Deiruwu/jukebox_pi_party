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