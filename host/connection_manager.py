from typing import Dict, Any
from fastapi import WebSocket

class ConnectionManager:
    def __init__(self):
        self.active_connections: Dict[str, Dict[str, Any]] = {}

    async def connect(self, websocket: WebSocket, client_id: str):
        await websocket.accept()
        self.active_connections[client_id] = {"websocket": websocket, "info": {}}
        print(f"New client: {client_id}")

    def disconnect(self, client_id: str):
        if client_id in self.active_connections:
            del self.active_connections[client_id]
            print(f"Client disconnected: {client_id}")


    def update_client_info(self, client_id: str, info_str: str):
        if client_id in self.active_connections:
            try:
                parts = [p.strip() for p in info_str.split(',')]
                self.active_connections[client_id]["info"] = {
                    "host_name": parts[0],
                    "os": parts[1],
                    "status": "Online"
                }
                print(f"Got info from {client_id}: {self.active_connections[client_id]['info']}")
            except IndexError:
                print(f"Sysinfo parsing error{client_id}: {info_str}")

    async def send_personal_message(self, message: str, client_id: str):
        if client_id in self.active_connections:
            websocket = self.active_connections[client_id]["websocket"]
            await websocket.send_text(message)
            print(f"Sending {message}' to client {client_id}")
            return True
        else:
            print(f"Error: client {client_id} is not connected")
            return False

manager = ConnectionManager()
