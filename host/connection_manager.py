from typing import Dict, Any
from fastapi import WebSocket
import asyncio

class ConnectionManager:
    def __init__(self):
        self.active_connections: Dict[str, Dict[str, Any]] = {}

    async def connect(self, websocket: WebSocket, client_id: str):
            await websocket.accept()
            self.active_connections[client_id] = {
                "websocket": websocket,
                "info": {
                    "ip_address": websocket.client.host if websocket.client else "unknown",
                    "status" : "connecting..."
                },
                "queue": asyncio.Queue(),
                "results_queue" : asyncio.Queue(),
                "feed_viewers": set(),
                "screen_viewers": set()
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
                self.active_connections[client_id]["info"].update({
                    "host_name": parts[0],
                    "os": f"{parts[1]} ({parts[2]})",
                    "status": "Online"
                })
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



    async def forward_video_frame(self, frame_data: bytes, client_id: str):
        if client_id in self.active_connections:
            viewers = self.active_connections[client_id].get("feed_viewers", set())
            disconnected_viewers = []
            for viewer_ws in viewers:
                try:
                    await viewer_ws.send_bytes(frame_data)
                except Exception as e:
                    print(f"Failed to send video frame to viewer: {e}")
                    disconnected_viewers.append(viewer_ws)

            # Remove disconnected viewers
            for viewer_ws in disconnected_viewers:
                viewers.discard(viewer_ws)

    async def forward_screen_frame(self, frame_data: bytes, client_id: str):
        if client_id in self.active_connections:
            viewers = self.active_connections[client_id].get("screen_viewers", set())
            disconnected_viewers = []
            for viewer_ws in viewers:
                try:
                    await viewer_ws.send_bytes(frame_data)
                except Exception as e:
                    print(f"Failed to send screen frame to viewer: {e}")
                    disconnected_viewers.append(viewer_ws)

            # Remove disconnected viewers
            for viewer_ws in disconnected_viewers:
                viewers.discard(viewer_ws)

    def add_feed_viewer(self, websocket: WebSocket, client_id: str):
        if client_id in self.active_connections:
            self.active_connections[client_id]["feed_viewers"].add(websocket)
            print(f"New viewer for {client_id}")

    def remove_feed_viewer(self, websocket: WebSocket, client_id: str):
        if client_id in self.active_connections:
            self.active_connections[client_id]["feed_viewers"].discard(websocket)
            print(f"Viewer for {client_id} disconnected")

    def add_screen_viewer(self, websocket: WebSocket, client_id: str):
        if client_id in self.active_connections:
            self.active_connections[client_id]["screen_viewers"].add(websocket)
            print(f"New screen viewer for {client_id}")

    def remove_screen_viewer(self, websocket: WebSocket, client_id: str):
        if client_id in self.active_connections:
            self.active_connections[client_id]["screen_viewers"].discard(websocket)
            print(f"Screen viewer for {client_id} disconnected")

manager = ConnectionManager()
