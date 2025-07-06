from fastapi import FastAPI
from routes import websocket, ui, builder

app = FastAPI(title="hrat C&C")

app.include_router(websocket.router)
app.include_router(ui.router, tags=["User Interface"])
app.include_router(builder.router, prefix="/builder", tags=["Client Builder"])

@app.get("/ping")
def ping():
    return {"status": "ok"}
