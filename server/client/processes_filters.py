from abc import abstractmethod, ABC
from pydantic import BaseModel


class ProcessesFilter(BaseModel, ABC):
    @abstractmethod
    def to_dict(self) -> dict: pass


class BlacklistFilter(ProcessesFilter):
    paths: list[str] = []
    def to_dict(self) -> dict:
        return {
            "type": "blacklist",
            "list": self.paths
        }
