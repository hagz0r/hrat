import asyncio
import websockets
from PIL import Image
import io
import pygame

# Команды для управления стримингом
START_STREAM_CMD = "11"
STOP_STREAM_CMD = "10"

async def handle_streaming(websocket):
    pygame.init()
    screen = None

    while True:
        try:
            if screen is not None:
                # Получение данных изображения от клиента
                message = await websocket.recv()
                if isinstance(message, bytes):
                    buffer = message

                    if len(buffer) > 0:  # Если буфер не пустой, считаем, что кадр завершен
                        complete_message = bytes(buffer)
                        print(f"Received frame of size {len(complete_message)} bytes")  # Отладочная информация

                        try:
                            # Преобразуем байты в изображение
                            image = Image.open(io.BytesIO(complete_message))
                            image = image.convert("RGB")

                            # Преобразуем изображение в формат, подходящий для Pygame
                            mode = image.mode
                            size = image.size
                            data = image.tobytes()

                            # Создаем Pygame surface и отображаем его
                            pygame_image = pygame.image.fromstring(data, size, mode)
                            screen.blit(pygame_image, (0, 0))
                            pygame.display.flip()
                        except Exception as e:
                            print(f"Failed to process frame: {e}")  # Отладочная информация

                # Проверяем события Pygame (например, закрытие окна)
                for event in pygame.event.get():
                    if event.type == pygame.QUIT:
                        pygame.quit()
                        return

            await asyncio.sleep(0.01)  # Маленькая задержка для предотвращения 100% использования CPU

        except websockets.exceptions.ConnectionClosedError as e:
            print(f"Connection closed with error: {e}")
            break

async def send_commands(websocket):
    screen = None
    while True:
        try:
            command = input("Enter command (11 to start streaming, 10 to stop streaming): ").strip()
            if command == START_STREAM_CMD:
                if screen is None:
                    screen = pygame.display.set_mode((800, 600))
                    pygame.display.set_caption("Remote Screen Stream")
                await websocket.send(command.encode())
            elif command == STOP_STREAM_CMD:
                if screen is not None:
                    pygame.display.quit()
                    screen = None
                await websocket.send(command.encode())
            else:
                print("Unknown command")

        except websockets.exceptions.ConnectionClosedError as e:
            print(f"Connection closed with error: {e}")
            break

async def echo(websocket, path):
    await asyncio.gather(handle_streaming(websocket), send_commands(websocket))

start_server = websockets.serve(echo, "localhost", 8765, max_size=2**20)

asyncio.get_event_loop().run_until_complete(start_server)
print("Server started on ws://localhost:8765")
asyncio.get_event_loop().run_forever()
