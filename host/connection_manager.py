from typing import Dict, Any
from fastapi import WebSocket
import asyncio

class ConnectionManager:
    def __init__(self):
        # Структура: {"client_id": {"websocket": ws, "info": {...}, "queue": Queue()}}
        self.active_connections: Dict[str, Dict[str, Any]] = {}

    async def connect(self, websocket: WebSocket, client_id: str):
        await websocket.accept()
        self.active_connections[client_id] = {
            "websocket": websocket,
            "info": {},
            "queue": asyncio.Queue() # <-- ДОБАВЛЕНО
        }
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
                    "os": f"{parts[1]} ({parts[2]})",
                    "status": "Online"
                }
                print(f"Got info from {client_id}: {self.active_connections[client_id]['info']}")
            except IndexError:
                print(f"Sysinfo parsing error for {client_id}: {info_str}")


    async def send_personal_message(self, message: str, client_id: str):
        if client_id not in self.active_connections:
            print("send_personal_message: no such client", client_id)
            return False

        q = self.active_connections[client_id]["queue"]
        await q.put(message)
        print("send_personal_message: queued", message)
        return True

manager = ConnectionManager()
