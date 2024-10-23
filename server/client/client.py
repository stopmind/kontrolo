import json

from pydantic import BaseModel
from starlette.websockets import WebSocket, WebSocketDisconnect
from client.processes_filters import ProcessesFilter

class ClientHello(BaseModel):
    mac: str

async def get_connection(connection: WebSocket) -> ClientHello:
    await connection.accept()
    text = await connection.receive_text()
    return ClientHello.model_validate_json(text)

class Client:
    pass

class ClientsManager:
    clients: dict[str, Client] = {}
    async def new_client(self, connection: WebSocket) -> Client:
        hello_msg = await get_connection(connection)
        client = Client(connection, self, hello_msg.mac)
        self.clients[client.mac] = client
        return client

    def get_client(self, mac: str) -> Client:
        return self.clients[mac]

    def remove_client(self, mac: str):
        self.clients.pop(mac)


class Client:
    connection: WebSocket
    mac: str
    manager: ClientsManager

    async def _send(self, command: str, data: object):
        try:
            await self.connection.send_text(json.dumps({
                "command": command,
                "data": data
            }))
        except WebSocketDisconnect as disconnect:
            self.manager.remove_client(self.mac)
            raise disconnect

    async def set_filter(self, filter: ProcessesFilter):
        await self._send("processes-watcher-set-filter", filter.to_dict())

    async def shutdown(self):
        await self._send("shutdown", None)

    async def scripts_update(self, id: str, content: str):
        await self._send("scripts-update", {
            "id": id,
            "content": content
        })

    async def scripts_exec(self, id: str):
        await self._send("scripts-exec", id)

    async def scripts_remove(self, id: str):
        await self._send("scripts-remove", id)

    async def close(self):
        self.manager.remove_client(self.mac)
        await self.connection.close()

    def __init__(self, connection: WebSocket, manager: ClientsManager, mac: str):
        self.connection = connection
        self.manager = manager
        self.mac = mac
