from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from connection_manager import manager
import asyncio
import os

router = APIRouter()

UPLOADS_DIR = os.path.join(os.path.dirname(__file__), "..", "uploads")
os.makedirs(UPLOADS_DIR, exist_ok=True)


@router.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: str):
    await manager.connect(websocket, client_id)

    send_queue = manager.active_connections[client_id]["queue"]

    async def sender():
        print(f"[Sender Task] For {client_id} started. Waiting for messages in queue...")
        while True:
            try:
                msg_to_send = await send_queue.get()
                print(f"[Sender Task] For {client_id} got message from queue: {msg_to_send}")
                await websocket.send_text(msg_to_send)
                print(f"[Sender Task] For {client_id} successfully sent message.")
                send_queue.task_done()
            except asyncio.CancelledError:
                print(f"[Sender Task] For {client_id} was cancelled.")
                break
            except Exception as e:
                print(f"[Sender Task] For {client_id} encountered an error: {e}")
                break

    sender_task = asyncio.create_task(sender())

    try:
        sysinfo = await websocket.receive_text()
        manager.update_client_info(client_id, sysinfo)

        while True:
            message = await websocket.receive()

            if "text" in message:
                response_data = message["text"]
                print(f"\n--- Text Response from {client_id} ---\n{response_data}\n--------------------\n")

            elif "bytes" in message:
                image_data = message["bytes"]

                import time
                file_path = os.path.join(UPLOADS_DIR, f"webcam_{client_id}_{int(time.time())}.jpeg")

                try:
                    with open(file_path, "wb") as f:
                        f.write(image_data)
                    print(f"--- Binary data from {client_id} received ({len(image_data)} bytes). Saved to {file_path} ---")
                except IOError as e:
                    print(f"--- FAILED to save webcam frame for {client_id}: {e} ---")


    except WebSocketDisconnect:
        print(f"Client {client_id} disconnected.")
    except Exception as e:
        print(f"An error occurred with client {client_id}: {e}")
    finally:
        sender_task.cancel()
        manager.disconnect(client_id)
        print(f"Connection closed for {client_id}. Cleanup complete.")
