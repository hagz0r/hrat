from fastapi import APIRouter, Request
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
from connection_manager import manager
import os

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
