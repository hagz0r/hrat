from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
import asyncio

app = FastAPI()

templates = Jinja2Templates(directory="templates")

connected_clients = {
    "192.168.1.101": {"os": "Windows 11", "status": "Online", "country": "RU"},
    "10.0.2.15": {"os": "Windows 10", "status": "Online", "country": "US"},
    "192.168.1.102": {"os": "Windows 10", "status": "Offline", "country": "DE"},
}

@app.get("/", response_class=HTMLResponse)
async def get_main_page(request: Request):
    context = {
        "request": request,
        "clients": connected_clients,
    }
    return templates.TemplateResponse("index.html", context)


@app.post("/run_command/{client_ip}", response_class=HTMLResponse)
async def run_command(request: Request, client_ip: str):
    form_data = await request.form()
    command = form_data.get("command", "Empty command")

    print(f"Got command '{command}' for client: {client_ip}")


    await asyncio.sleep(1)
    response_from_client = f"Response to '{command}': ... OK"

    return HTMLResponse(f"""
        <div class='mt-3 p-3 bg-gray-700 border border-green-500 rounded-lg animate-pulse'>
            <p><strong>Response from {client_ip}:</strong></p>
            <pre class='text-green-300'>{response_from_client}</pre>
        </div>
    """)

@app.get("/controls/{client_ip}", response_class=HTMLResponse)
async def get_client_controls(request: Request, client_ip: str):
    client_info = connected_clients.get(client_ip)
    if not client_info:
        return HTMLResponse("<div class='text-red-500'>Client not found</div>")

    context = {"request": request, "ip": client_ip}
    return templates.TemplateResponse("controls.html", context)
