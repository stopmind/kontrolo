from time import sleep

import uvicorn
from fastapi import FastAPI
from starlette.websockets import WebSocket
from client.client import Client, ClientsManager

app = FastAPI()

clients_manager = ClientsManager()

@app.websocket("/client/socket")
async def client_socket_endpoint(websocket: WebSocket):
    cl = await clients_manager.new_client(websocket)

    await cl.scripts_update("aboba", """
log("Aboba")
    """)
    await cl.scripts_exec("aboba")
    sleep(1)
    await cl.scripts_remove("aboba")


if __name__ == "__main__":
    uvicorn.run("main:app", ws="websockets")