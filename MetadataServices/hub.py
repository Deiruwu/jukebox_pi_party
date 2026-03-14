import asyncio
import json
from services.youtube_service import YouTubeService

class MetadataHub:
    def __init__(self, host='127.0.0.1', port=5010):
        self.host = host
        self.port = port
        self.youtube_service = YouTubeService()

    async def _handle_request(self, reader, writer):
        try:
            data = await reader.readline()
            if not data:
                return

            request = json.loads(data.decode('utf-8').strip())
            action = request.get('action')

            if action == "search":
                query = request.get('query', '')
                loop = asyncio.get_running_loop()
                result = await loop.run_in_executor(None, self.youtube_service.search, query, 5)
                response = {"status": "ok", "data": result}
            else:
                response = {"status": "error", "message": "Unknown action"}

        except json.JSONDecodeError:
            response = {"status": "error", "message": "Invalid JSON"}
        except Exception as e:
            response = {"status": "error", "message": str(e)}

        writer.write((json.dumps(response) + "\n").encode('utf-8'))
        await writer.drain()
        writer.close()
        await writer.wait_closed()

    async def start(self):
        server = await asyncio.start_server(self._handle_request, self.host, self.port)

        print(f"[HUB] Escuchando {self.host}:{self.port}", flush=True)

        async with server:
            await server.serve_forever()

if __name__ == "__main__":
    hub = MetadataHub()
    asyncio.run(hub.start())