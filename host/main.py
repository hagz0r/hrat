from fastapi import FastAPI
from fastapi.staticfiles import StaticFiles
from routes import websocket, ui, builder
import os

app = FastAPI(title="hrat C&C")

# Mount static files
static_dir = os.path.join(os.path.dirname(__file__), "static")
if os.path.exists(static_dir):
    app.mount("/static", StaticFiles(directory=static_dir), name="static")

app.include_router(websocket.router)
app.include_router(ui.router, tags=["User Interface"])
app.include_router(builder.router, prefix="/builder", tags=["Client Builder"])

@app.get("/ping")
def ping():
    return {"status": "ok"}
