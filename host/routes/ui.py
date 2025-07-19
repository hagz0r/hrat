from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.templating import Jinja2Templates
from connection_manager import manager
import os
import json
import asyncio

router = APIRouter()

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TEMPLATE_DIR = os.path.join(BASE_DIR, "..", "templates")
COMPONENTS_DIR = os.path.join(TEMPLATE_DIR, "components")
UPLOADS_DIR = os.path.join(BASE_DIR, "..", "uploads")

os.makedirs(UPLOADS_DIR, exist_ok=True)

templates = Jinja2Templates(directory=TEMPLATE_DIR)


@router.get("/", response_class=HTMLResponse, tags=["UI"])
async def get_main_page(request: Request):
    clients_data = {cid: conn["info"] for cid, conn in manager.active_connections.items()}
    return templates.TemplateResponse("index.html", {"request": request, "clients": clients_data})

@router.get("/components/{component_name}/{client_id}", response_class=HTMLResponse)
async def get_component(request: Request, component_name: str, client_id: str):
    if client_id not in manager.active_connections:
        raise HTTPException(status_code=404, detail="Client not found")

    if component_name not in get_available_components():
        raise HTTPException(status_code=404, detail="Component not found")

    template_path = f"components/{component_name}.html"
    return templates.TemplateResponse(template_path, {"request": request, "ip": client_id})

@router.get("/controls/{client_id}", response_class=HTMLResponse, tags=["UI"])
async def get_client_controls(request: Request, client_id: str):
    if client_id not in manager.active_connections:
        return HTMLResponse("<div class='text-red-500 p-4'>Client not found.</div>")

    available_components = get_available_components()
    context = {
        "request": request,
        "ip": client_id,
        "components": available_components
    }
    return templates.TemplateResponse("controls.html", context)


def get_available_components():
    if not os.path.isdir(COMPONENTS_DIR):
        return []

    components = [
        f.replace(".html", "")
        for f in os.listdir(COMPONENTS_DIR)
        if f.endswith(".html")
    ]
    # components.sort()
    return components


@router.post("/api/command/{client_id}", response_class=HTMLResponse)
async def handle_api_command(request: Request, client_id: str):
    if client_id not in manager.active_connections:
        raise HTTPException(status_code=404, detail="Client not found")

    try:
        form_data = await request.form()
        module_name = form_data.get("module")
        command_payload = { "module": module_name, "args": {} }
        for key, value in form_data.items():
            if key.startswith("args."):
                arg_key = key.split(".", 1)[1]
                command_payload["args"][arg_key] = value

        no_response_modules = ["WC", "RS"]
        if module_name in no_response_modules:
            command_str = json.dumps(command_payload)
            await manager.send_personal_message(command_str, client_id)
            return HTMLResponse(f"<p class='text-gray-400'>Command for '{module_name}' sent (no direct output expected).</p>")

        results_q = manager.active_connections[client_id]["results_queue"]
        while not results_q.empty():
            results_q.get_nowait()

        command_str = json.dumps(command_payload)
        success = await manager.send_personal_message(command_str, client_id)
        if not success:
            return HTMLResponse(f"<pre class='text-red-500'>Failed to send command for '{module_name}'.</pre>")

        try:
            result_text = await asyncio.wait_for(results_q.get(), timeout=15.0)
            return HTMLResponse(f"<pre class='text-green-300'>{result_text}</pre>")
        except asyncio.TimeoutError:
            return HTMLResponse("<pre class='text-yellow-500'>Command sent, but no response received (timeout).</pre>")

    except Exception as e:
        print(f"[HTTP Handler] Error: {e}")
        return HTMLResponse(f"<div class='text-red-500'>An error occurred: {e}</div>")
