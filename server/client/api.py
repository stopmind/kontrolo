from abc import abstractmethod, ABC
from pydantic import BaseModel
from client.processes_filters import ProcessesFilter


class Api(BaseModel, ABC):
    @abstractmethod
    async def _send(self, command: str, data: object): pass

    async def set_filter(self, filter: ProcessesFilter):
        await self._send("processes-watcher-set-filter", filter.to_dict())
