from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from connection_manager import manager
import asyncio
import os

router = APIRouter()

@router.websocket("/ws/{client_id}")
async def websocket_endpoint(websocket: WebSocket, client_id: str):
    await manager.connect(websocket, client_id)

    # Убедимся, что мы используем правильную очередь
    send_queue = manager.active_connections[client_id]["queue"]

    async def sender():
        print(f"[Sender Task] For {client_id} started. Waiting for messages in queue...")
        while True:
            try:
                msg = await send_queue.get()
                print(f"[Sender Task] For {client_id} got message from queue: {msg}")
                await websocket.send_text(msg)
                print(f"[Sender Task] For {client_id} successfully sent message.")
                send_queue.task_done() # Сообщаем очереди, что задача выполнена
            except asyncio.CancelledError:
                print(f"[Sender Task] For {client_id} was cancelled.")
                break
            except Exception as e:
                print(f"[Sender Task] For {client_id} encountered an error: {e}")
                break

    sender_task = asyncio.create_task(sender())
    try:
        # Первый прием - системная информация
        sysinfo = await websocket.receive_text()
        manager.update_client_info(client_id, sysinfo)

        # Цикл для получения ответов от клиента
        while True:
            response_data = await websocket.receive_text()
            print(f"\n--- Response from {client_id} ---\n{response_data}\n--------------------\n")

    except WebSocketDisconnect:
        print(f"Client {client_id} disconnected.")
    except Exception as e:
        print(f"An error occurred with client {client_id}: {e}")
    finally:
        # Обязательно отменяем задачу-отправитель при любом выходе
        sender_task.cancel()
        manager.disconnect(client_id)
        print(f"Connection closed for {client_id}. Cleanup complete.")
