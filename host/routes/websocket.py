from fastapi import APIRouter, WebSocket, WebSocketDisconnect
import asyncio
from connection_manager import manager

router = APIRouter()

@router.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: str):
    await manager.connect(websocket, client_id)
    try:
        sysinfo = await websocket.receive_text()
        manager.update_client_info(client_id, sysinfo)
        while True:
            await asyncio.sleep(3600)
    except WebSocketDisconnect:
        manager.disconnect(client_id)
    except Exception as e:
        print(f"An error occurred with client {client_id}: {e}")
        manager.disconnect(client_id)
