import uvicorn
from fastapi import FastAPI
from starlette.websockets import WebSocket

app = FastAPI()

@app.websocket("/client/socket")
async def client_socket_endpoint(websocket: WebSocket):
    await websocket.accept()
    await websocket.receive()
    await websocket.send_text("""
{
    "command": "say-hi",
    "data": "User"
}    
    """)

    await websocket.close()
    

if __name__ == "__main__":
    uvicorn.run("main:app", reload=True, ws="websockets")