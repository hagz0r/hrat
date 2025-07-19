from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.templating import Jinja2Templates
from connection_manager import manager
import os
import json

router = APIRouter()

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TEMPLATE_DIR = os.path.join(BASE_DIR, "..", "templates")
UPLOADS_DIR = os.path.join(BASE_DIR, "..", "uploads")

os.makedirs(UPLOADS_DIR, exist_ok=True)

templates = Jinja2Templates(directory=TEMPLATE_DIR)


@router.get("/", response_class=HTMLResponse, tags=["UI"])
async def get_main_page(request: Request):
    clients_data = {cid: conn["info"] for cid, conn in manager.active_connections.items()}
    return templates.TemplateResponse("index.html", {"request": request, "clients": clients_data})


@router.get("/controls/{client_id}", response_class=HTMLResponse, tags=["UI"])
async def get_client_controls(request: Request, client_id: str):
    if client_id not in manager.active_connections:
        return HTMLResponse("<div class='text-red-500 p-4'>Client not found or disconnected. Please go back to the main page.</div>")
    return templates.TemplateResponse("controls.html", {"request": request, "ip": client_id})

import asyncio
@router.post("/api/command/{client_id}", response_class=HTMLResponse)
async def handle_api_command(request: Request, client_id: str):
    if client_id not in manager.active_connections:
        raise HTTPException(status_code=404, detail="Client not found")

    try:
        form_data = await request.form()
        module_name = form_data.get("module")

        command_payload = {
            "module": module_name,
            "args": {}
        }
        for key, value in form_data.items():
            if key.startswith("args."):
                arg_key = key.split(".", 1)[1]
                command_payload["args"][arg_key] = value

        command_str = json.dumps(command_payload)
        print(f"[HTTP Handler] Queuing command for {client_id}: {command_str}")
        success = await manager.send_personal_message(command_str, client_id)

        if not success:
            return HTMLResponse("<p class='text-red-500'>Failed to queue command: client not found.</p>")

        return HTMLResponse(f"<p class='text-green-400'>Command for '{module_name}' sent.</p>")

    except Exception as e:
        print(f"[HTTP Handler] An error occurred: {e}")
        return HTMLResponse(f"<div class='text-red-500'>An error occurred: {e}</div>")
