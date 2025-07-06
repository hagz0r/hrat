import asyncio

import websockets

connected_client = None

async def echo(websocket, path):
    global connected_client
    if connected_client is not None:
        await websocket.send("A client is already connected. Connection rejected.")
        await websocket.close()
        return

    connected_client = websocket
    try:
        while True:
            message = await websocket.recv()
            print(f"Received message: {message}")
    except websockets.exceptions.ConnectionClosed:
        print("Client disconnected")
    finally:
        connected_client = None

async def send_messages():
    global connected_client
    while True:
        if connected_client is not None:
            response = input("Enter message to send to the client: ")
            await connected_client.send(response)
        await asyncio.sleep(1)  # Small delay to prevent busy-waiting


start_server = websockets.serve(echo, "localhost", 4040)

loop = asyncio.get_event_loop()
loop.run_until_complete(start_server)
print("Server started on ws://localhost:4040")

# Run both the server and the message sending coroutine
loop.run_until_complete(send_messages())
loop.run_forever()
