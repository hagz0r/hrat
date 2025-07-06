from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from connection_manager import manager

router = APIRouter()

@router.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: str):
    await manager.connect(websocket, client_id)
    try:
        sysinfo = await websocket.receive_text()
        manager.update_client_info(client_id, sysinfo)
        while True:
            response_data = await websocket.receive_text()
            print(f"Got answer from {client_id}: {response_data}")
    except WebSocketDisconnect:
        manager.disconnect(client_id)
