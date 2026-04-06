from fastapi import FastAPI, HTTPException, Body
from fastapi.staticfiles import StaticFiles
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from typing import List, Optional
import database
import agents
import uvicorn
import json

app = FastAPI()
writer = agents.AIAgent()

# Models
class Story(BaseModel):
    id: Optional[int] = None
    title: str

class Chapter(BaseModel):
    id: Optional[int] = None
    story_id: int
    title: str
    content: str
    order_index: int

class Node(BaseModel):
    id: Optional[int] = None
    story_id: int
    parent_id: Optional[int] = None
    name: str
    type: str # 'file' or 'folder'
    content: Optional[str] = None
    category: Optional[str] = None
    order_index: Optional[int] = 0

class ChatRequest(BaseModel):
    story_id: int
    message: str
    story_context: str

class WriteRequest(BaseModel):
    story_id: int
    current_chapter_id: int
    instruction: str
    story_context: str
    selection_context: Optional[str] = None

# Initialize DB on start
@app.on_event("startup")
def startup():
    database.init_db()
    # Migration: Move existing chapters to nodes if they haven't been moved yet
    conn = database.get_db_connection()
    cursor = conn.cursor()
    
    stories = conn.execute("SELECT id FROM stories").fetchall()
    for s in stories:
        sid = s["id"]
        # Check if default folders exist
        folder_check = conn.execute("SELECT id FROM nodes WHERE story_id = ? AND name = 'Chương' AND type = 'folder'", (sid,)).fetchone()
        if not folder_check:
            # Create default folders
            cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (sid, "Chương", "folder", "chapter"))
            chapters_folder_id = cursor.lastrowid
            cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (sid, "Quy tắc", "folder", "rule"))
            cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (sid, "Nhân vật", "folder", "character"))
            cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (sid, "Vật phẩm", "folder", "item"))
            
            # Move existing chapters
            old_chapters = conn.execute("SELECT title, content, order_index FROM chapters WHERE story_id = ?", (sid,)).fetchall()
            for c in old_chapters:
                cursor.execute(
                    "INSERT INTO nodes (story_id, parent_id, name, type, content, category, order_index) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    (sid, chapters_folder_id, c["title"], "file", c["content"], "chapter", c["order_index"])
                )
    
    conn.commit()
    conn.close()

# Endpoints
@app.get("/api/stories", response_model=List[Story])
async def get_stories():
    conn = database.get_db_connection()
    stories = conn.execute("SELECT id, title FROM stories ORDER BY created_at DESC").fetchall()
    conn.close()
    return [dict(s) for s in stories]

@app.post("/api/stories", response_model=Story)
async def create_story(story: Story):
    conn = database.get_db_connection()
    cursor = conn.cursor()
    cursor.execute("INSERT INTO stories (title) VALUES (?)", (story.title,))
    story_id = cursor.lastrowid
    
    # Create default folders
    cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (story_id, "Chương", "folder", "chapter"))
    chapters_folder_id = cursor.lastrowid
    
    cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (story_id, "Quy tắc", "folder", "rule"))
    cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (story_id, "Nhân vật", "folder", "character"))
    cursor.execute("INSERT INTO nodes (story_id, name, type, category) VALUES (?, ?, ?, ?)", (story_id, "Vật phẩm", "folder", "item"))
    
    # Create first chapter file
    cursor.execute("INSERT INTO nodes (story_id, parent_id, name, type, content, category, order_index) VALUES (?, ?, ?, ?, ?, ?, ?)", 
                   (story_id, chapters_folder_id, "Chương 1", "file", "", "chapter", 1))
    
    conn.commit()
    conn.close()
    return {"id": story_id, "title": story.title}

@app.get("/api/stories/{story_id}/nodes", response_model=List[Node])
async def get_nodes(story_id: int):
    conn = database.get_db_connection()
    nodes = conn.execute("SELECT * FROM nodes WHERE story_id = ? ORDER BY type DESC, order_index ASC, name ASC", (story_id,)).fetchall()
    conn.close()
    return [dict(n) for n in nodes]

@app.post("/api/nodes", response_model=Node)
async def create_node(node: Node):
    conn = database.get_db_connection()
    cursor = conn.cursor()
    cursor.execute(
        "INSERT INTO nodes (story_id, parent_id, name, type, content, category, order_index) VALUES (?, ?, ?, ?, ?, ?, ?)",
        (node.story_id, node.parent_id, node.name, node.type, node.content or "", node.category, node.order_index)
    )
    node.id = cursor.lastrowid
    conn.commit()
    conn.close()
    return node

@app.patch("/api/nodes/{node_id}")
async def update_node(node_id: int, data: dict = Body(...)):
    conn = database.get_db_connection()
    # Dynamic update
    allowed_fields = ["name", "content", "parent_id", "order_index"]
    items = [(k, v) for k, v in data.items() if k in allowed_fields]
    if items:
        set_clause = ", ".join([f"{k} = ?" for k, v in items])
        values = [v for k, v in items] + [node_id]
        conn.execute(f"UPDATE nodes SET {set_clause} WHERE id = ?", values)
        conn.commit()
    conn.close()
    return {"status": "success"}

@app.delete("/api/nodes/{node_id}")
async def delete_node(node_id: int):
    conn = database.get_db_connection()
    # Recursive delete could be complex in SQLite without triggers, but let's do a simple one for now
    # or just delete the node and orphans stay? (Better to use cascade or just a simple delete for now)
    conn.execute("DELETE FROM nodes WHERE id = ? OR parent_id = ?", (node_id, node_id)) # Simple 1-level deep delete
    conn.commit()
    conn.close()
    return {"status": "success"}

@app.post("/api/chapters/{story_id}", response_model=Chapter)
async def add_chapter(story_id: int):
    conn = database.get_db_connection()
    cursor = conn.cursor()
    # Get max order_index
    max_order = conn.execute("SELECT MAX(order_index) as max_idx FROM chapters WHERE story_id = ?", (story_id,)).fetchone()
    new_idx = (max_order["max_idx"] or 0) + 1
    new_title = f"Chương {new_idx}"
    
    cursor.execute("INSERT INTO chapters (story_id, title, content, order_index) VALUES (?, ?, ?, ?)", 
                   (story_id, new_title, "", new_idx))
    chap_id = cursor.lastrowid
    conn.commit()
    conn.close()
    return {"id": chap_id, "story_id": story_id, "title": new_title, "content": "", "order_index": new_idx}

@app.get("/api/chapters/{story_id}", response_model=List[Chapter])
async def get_chapters(story_id: int):
    conn = database.get_db_connection()
    chapters = conn.execute("SELECT id, story_id, title, content, order_index FROM chapters WHERE story_id = ? ORDER BY order_index ASC", (story_id,)).fetchall()
    conn.close()
    return [dict(c) for c in chapters]

@app.patch("/api/chapters/{chapter_id}")
async def update_chapter(chapter_id: int, content: str = Body(..., embed=True)):
    conn = database.get_db_connection()
    conn.execute("UPDATE chapters SET content = ? WHERE id = ?", (content, chapter_id))
    conn.commit()
    conn.close()
    return {"status": "success"}

@app.post("/api/chat")
async def chat(req: ChatRequest):
    # Fetch chat history for this story
    conn = database.get_db_connection()
    history = conn.execute("SELECT role, content FROM messages WHERE story_id = ? ORDER BY created_at ASC LIMIT 10", (req.story_id,)).fetchall()
    chat_history = [{"role": row["role"], "content": row["content"]} for row in history]
    
    # Save user message
    conn.execute("INSERT INTO messages (story_id, role, content) VALUES (?, ?, ?)", (req.story_id, "user", req.message))
    conn.commit()
    conn.close()

    async def stream_chat():
        generator = await writer.chat_node(
            req.story_id, 
            req.message, 
            chat_history
        )
        full_response = ""
        async for chunk in generator:
            full_response += chunk
            yield chunk
        
        # Save assistant message at the end
        conn_save = database.get_db_connection()
        conn_save.execute("INSERT INTO messages (story_id, role, content) VALUES (?, ?, ?)", (req.story_id, "assistant", full_response))
        conn_save.commit()
        conn_save.close()

    return StreamingResponse(stream_chat(), media_type="text/plain")

@app.post("/api/write")
async def write(req: WriteRequest):
    # Fetch previous chapters
    conn = database.get_db_connection()
    current_chap = conn.execute("SELECT order_index FROM chapters WHERE id = ?", (req.current_chapter_id,)).fetchone()
    
    prev_chaps_content = ""
    if current_chap:
        prev_chaps = conn.execute(
            "SELECT title, content FROM chapters WHERE story_id = ? AND order_index < ? ORDER BY order_index ASC", 
            (req.story_id, current_chap["order_index"])
        ).fetchall()
        prev_chaps_content = "\n\n".join([f"## {c['title']}\n{c['content']}" for c in prev_chaps])

    # Fetch chat history
    history = conn.execute("SELECT role, content FROM messages WHERE story_id = ? ORDER BY created_at ASC LIMIT 10", (req.story_id,)).fetchall()
    chat_history = [{"role": row["role"], "content": row["content"]} for row in history]
    conn.close()

    async def stream_write():
        generator = await writer.writing_node(
            req.story_id, 
            req.story_context,
            req.instruction, 
            selection=req.selection_context,
            previous_chapters=prev_chaps_content,
            chat_history=chat_history
        )
        async for chunk in generator:
            yield chunk

    return StreamingResponse(stream_write(), media_type="text/plain")

# Static files should be served at the end
app.mount("/", StaticFiles(directory="static", html=True), name="static")

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)
