import uvicorn
from fastapi import FastAPI
from starlette.websockets import WebSocket

from client.client import Client, ClientsManager
from client.processes_filters import BlacklistFilter

app = FastAPI()

clients_manager = ClientsManager()

@app.websocket("/client/socket")
async def client_socket_endpoint(websocket: WebSocket):
    cl = await clients_manager.new_client(websocket)
    await cl.set_filter(BlacklistFilter(paths=[
        "C:\\Windows\\System32\\cmd.exe"
    ]))
    await cl.shutdown()


if __name__ == "__main__":
    uvicorn.run("main:app", ws="websockets")