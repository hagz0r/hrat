import asyncio
import websockets

async def echo(websocket, path):
    while True:
        message = await websocket.recv()
        print(f"Received message: {message}")
        response = input("Enter message: ")
        await websocket.send(response)

start_server = websockets.serve(echo, "localhost", 8765)

asyncio.get_event_loop().run_until_complete(start_server)
print("Server started on ws://localhost:8765")
asyncio.get_event_loop().run_forever()
