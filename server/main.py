import uvicorn
from fastapi import FastAPI
from starlette.websockets import WebSocket

from client.client import Client
from client.processes_filters import BlacklistFilter

app = FastAPI()


@app.websocket("/client/socket")
async def client_socket_endpoint(websocket: WebSocket):
    await websocket.accept()
    await websocket.receive()
    cl = Client(connection=websocket)
    await cl.set_filter(BlacklistFilter(paths=[
        "C:\\Windows\\System32\\cmd.exe"
    ]))


if __name__ == "__main__":
    uvicorn.run("main:app", ws="websockets")