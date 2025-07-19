from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from connection_manager import manager
import asyncio

router = APIRouter()

@router.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: str):
    await manager.connect(websocket, client_id)
    cmd_queue = manager.active_connections[client_id]["queue"]

    async def sender():
        while True:
            try:
                msg = await cmd_queue.get()
                await websocket.send_text(msg)
                cmd_queue.task_done()
            except (asyncio.CancelledError, WebSocketDisconnect): break
            except Exception: break

    sender_task = asyncio.create_task(sender())
    try:
        sysinfo = await websocket.receive_text()
        manager.update_client_info(client_id, sysinfo)
        while True:
            message = await websocket.receive()
            if "text" in message:
                print(f"\n--- Text from {client_id}: {message['text']} ---")
            elif "bytes" in message:
                await manager.forward_video_frame(message["bytes"], client_id)
    except (WebSocketDisconnect, RuntimeError): pass
    finally:
        sender_task.cancel()
        manager.disconnect(client_id)
        print(f"Cleanup for client {client_id} complete.")

@router.websocket("/ws/feed/{client_id}")
async def websocket_feed_endpoint(websocket: WebSocket, client_id: str):
    await websocket.accept()
    manager.add_feed_viewer(websocket, client_id)
    try:
        while True:
            await websocket.receive_text()
    except WebSocketDisconnect:
        pass
    finally:
        manager.remove_feed_viewer(websocket, client_id)
