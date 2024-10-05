import json

from pydantic import ConfigDict
from starlette.websockets import WebSocket
from client.api import Api


class Client(Api):
    model_config = ConfigDict(arbitrary_types_allowed=True)
    connection: WebSocket

    async def _send(self, command: str, data: object):
        await self.connection.send_text(json.dumps({
            "command": command,
            "data": data
        }))
