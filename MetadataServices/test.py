import socket
import json


def interactive_test():
    host = '127.0.0.1'
    port = 5005

    print(f"[*] Conectando a {host}:{port}. Escribe tu búsqueda o 'exit' para salir.")

    while True:
        query = input("\n[YouTube Search] > ").strip()
        if not query or query.lower() == 'exit':
            break

        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client:
                client.settimeout(10.0)
                client.connect((host, port))

                payload = {"action": "search", "query": query}
                client.sendall((json.dumps(payload) + "\n").encode('utf-8'))

                # Leer respuesta completa (NDJSON)
                buffer = b""
                while b"\n" not in buffer:
                    chunk = client.recv(4096)
                    if not chunk: break
                    buffer += chunk

                response = json.loads(buffer.decode('utf-8'))
                print(json.dumps(response, indent=2, ensure_ascii=False))

        except ConnectionRefusedError:
            print("[!] Error: El Hub no está corriendo.")
        except Exception as e:
            print(f"[!] Error inesperado: {e}")


if __name__ == "__main__":
    interactive_test()