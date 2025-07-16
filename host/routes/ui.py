from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
from connection_manager import manager
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


@router.get("/controls/{client_id}", response_class=HTMLResponse, tags=["UI"])
async def get_client_controls(request: Request, client_id: str):
    if client_id not in manager.active_connections:
        return HTMLResponse("<div class='text-red-500'>Client not found or disconnected</div>")
    return templates.TemplateResponse("controls.html", {"request": request, "ip": client_id})


@router.post("/api/command/{client_id}", response_class=HTMLResponse)
async def handle_api_command(request: Request, client_id: str):
    """
    Эта функция теперь принимает данные формы, а не JSON, и отправляет команду клиенту.
    """
    if client_id not in manager.active_connections:
        raise HTTPException(status_code=404, detail="Client not found")

    try:
        # --- ИЗМЕНЕНИЕ: Читаем данные из формы вместо JSON ---
        form_data = await request.form()

        # --- ИЗМЕНЕНИЕ: Вручную собираем полезную нагрузку ---
        command_payload = {
            "module": form_data.get("module"),
            "args": {}
        }
        for key, value in form_data.items():
            if key.startswith("args."):
                # Преобразуем 'args.command' в 'command'
                arg_key = key.split(".", 1)[1]
                command_payload["args"][arg_key] = value

        command_str = json.dumps(command_payload)
        # --- КОНЕЦ ИЗМЕНЕНИЙ ---

        print(f"[HTTP Handler] Prepared command for {client_id}: {command_str}") # Добавим лог

        success = await manager.send_personal_message(command_str, client_id)

        if success:
            return HTMLResponse(f"<p class='text-green-400'>Command sent to {command_payload['module']} module. Check server console.</p>")
        else:
            return HTMLResponse("<p class='text-red-500'>Failed to send command: client not found.</p>")

    except Exception as e:
        print(f"[HTTP Handler] An error occurred: {e}")
        return HTMLResponse(f"<div class='text-red-500'>An error occurred: {e}</div>")
