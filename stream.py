import asyncio
import websockets
import cv2
import numpy as np

# Create a VideoWriter object to save the video stream
fourcc = cv2.VideoWriter_fourcc(*'MJPG')
out = cv2.VideoWriter('output.avi', fourcc, 20.0, (1280, 720))

async def handle_client(websocket, path):
    print(f"Client connected: {path}")
    try:
        async for message in websocket:
            if isinstance(message, bytes):
                img = cv2.imdecode(np.frombuffer(message, dtype=np.uint8), cv2.IMREAD_COLOR)
                if img is not None:
                    cv2.imshow('Video Stream', img)
                    out.write(img)
                    if cv2.waitKey(1) & 0xFF == ord('q'):
                        break
    finally:
        out.release()
        cv2.destroyAllWindows()
        print("Client disconnected")

start_server = websockets.serve(handle_client, "localhost", 4043)

print("Server started, waiting for connections...")
asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()
