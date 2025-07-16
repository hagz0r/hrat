from fastapi import APIRouter, Request, Form
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
from connection_manager import manager
from fastapi import HTTPException
import asyncio
import os
import json

router = APIRouter()

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TEMPLATE_DIR = os.path.join(BASE_DIR, "..", "templates")
templates = Jinja2Templates(directory=TEMPLATE_DIR)


@router.get("/", response_class=HTMLResponse, tags=["UI"])
async def get_main_page(request: Request):
    clients_data = {cid: conn["info"] for cid, conn in manager.active_connections.items()}
    return templates.TemplateResponse("index.html", {"request": request, "clients": clients_data})

@router.get("/controls/{client_ip}", response_class=HTMLResponse, tags=["UI"])
async def get_client_controls(request: Request, client_ip: str):
    if client_ip not in manager.active_connections:
        return HTMLResponse("<div class='text-red-500'>Client not found or disconnected</div>")
    return templates.TemplateResponse("controls.html", {"request": request, "ip": client_ip})


@router.post("/command/run/{client_id}", response_class=HTMLResponse)
async def run_command(request: Request, client_id: str, command: str = Form(...)):
    if client_id not in manager.active_connections:
        raise HTTPException(status_code=404, detail="Client not found")

    shell = "bash"

    payload = {
        "module": "RSH",
        "args": {
            "command": command,
            "shell": shell
        }
    }

    # Отправляем JSON как строку
    await manager.send_personal_message(json.dumps(payload), client_id)

    websocket = manager.active_connections[client_id]["websocket"]
    try:
        # Ожидаем текстовый ответ от клиента
        response_data = await asyncio.wait_for(websocket.receive_text(), timeout=15.0)
        return HTMLResponse(f"<pre class='whitespace-pre-wrap text-gray-300'>{response_data}</pre>")
    except asyncio.TimeoutError:
        return HTMLResponse("<div class='text-red-500'>Error: Timed out waiting for response from client.</div>")
    except Exception as e:
        return HTMLResponse(f"<div class='text-red-500'>Error receiving response: {e}</div>")

@router.post("/fs/list/{client_id}", response_class=HTMLResponse)
async def list_directory(request: Request, client_id: str, path: str = Form(...)):
    if client_id not in manager.active_connections:
        raise HTTPException(status_code=404, detail="Client not found")

    # Формируем JSON для листинга директории
    # Примечание: В `file_system.rs` используется "GET", а не "list", как в документации. Используем то, что в коде.
    payload = {
        "module": "FS",
        "args": {
            "operation": "GET",
            "path": path
        }
    }
    await manager.send_personal_message(json.dumps(payload), client_id)

    websocket = manager.active_connections[client_id]["websocket"]
    try:
        response_data = await asyncio.wait_for(websocket.receive_text(), timeout=15.0)
        return HTMLResponse(f"<pre class='whitespace-pre-wrap text-gray-300'>{response_data}</pre>")
    except asyncio.TimeoutError:
        return HTMLResponse("<div class='text-red-500'>Error: Timed out waiting for response from client.</div>")
    except Exception as e:
        return HTMLResponse(f"<div class='text-red-500'>Error receiving response: {e}</div>")
