import os
import database
from google import genai
from google.genai import types
from dotenv import load_dotenv

load_dotenv()

class AIAgent:
    def __init__(self):
        self.api_key = os.environ.get("GEMINI_API_KEY")
        if not self.api_key:
            raise ValueError("GEMINI_API_KEY environment variable is not set")
        self.client = genai.Client(api_key=self.api_key)
        self.model = "gemma-4-31b-it"

    def _get_config(self, thinking_level="HIGH"):
        return types.GenerateContentConfig(
            thinking_config=types.ThinkingConfig(
                thinking_level=thinking_level,
            ),
            tools=[types.Tool(googleSearch=types.GoogleSearch())],
        )

    def context_node(self, story_id):
        """
        Quét database để lấy các thông tin về nhân vật, vật phẩm, quy tắc giúp AI bám sát nội dung.
        """
        conn = database.get_db_connection()
        nodes = conn.execute("SELECT name, content, category FROM nodes WHERE story_id = ? AND type = 'file' AND category IN ('rule', 'character', 'item', 'plot')", (story_id,)).fetchall()
        
        context_data = {
            "rules": [],
            "characters": [],
            "items": [],
            "plot": []
        }
        
        for n in nodes:
            cat = n["category"] + "s"
            if cat in context_data or n["category"] in context_data:
                key = n["category"] if n["category"] in context_data else cat
                context_data[key].append(f"### {n['name']}\n{n['content']}")
        
        conn.close()
        
        context_str = "# KIẾN THỨC VỀ TRUYỆN\n\n"
        if context_data["rules"]:
            context_str += "## QUY TẮC TRUYỆN\n" + "\n".join(context_data["rules"]) + "\n\n"
        if context_data["characters"]:
            context_str += "## NHÂN VẬT\n" + "\n".join(context_data["characters"]) + "\n\n"
        if context_data["items"]:
            context_str += "## VẬT PHẨM & BỐI CẢNH\n" + "\n".join(context_data["items"]) + "\n\n"
        if context_data["plot"]:
            context_str += "## CỐT TRUYỆN TỔNG THỂ\n" + "\n".join(context_data["plot"]) + "\n\n"
            
        return context_str

    async def chat_node(self, story_id, message, chat_history):
        """
        Node xử lý Chat: Hỗ trợ lên ý tưởng dựa trên kiến thức đã có.
        """
        kb_context = self.context_node(story_id)
        
        system_prompt = (
            "Bạn là một trợ lý sáng tác chuyên nghiệp. Hãy sử dụng KIẾN THỨC VỀ TRUYỆN dưới đây để trả lời.\n\n"
            f"{kb_context}\n"
        )
        
        contents = [types.Content(role="system", parts=[types.Part.from_text(text=system_prompt)])]
        for msg in chat_history:
            contents.append(types.Content(role=msg['role'], parts=[types.Part.from_text(text=msg['content'])]))
        
        contents.append(types.Content(role="user", parts=[types.Part.from_text(text=message)]))

        async def stream_gen():
            for chunk in self.client.models.generate_content_stream(
                model=self.model,
                contents=contents,
                config=self._get_config(),
            ):
                if text := chunk.text:
                    yield text
        
        return stream_gen()

    async def writing_node(self, story_id, current_content, instruction, selection=None, previous_chapters=None, chat_history=None):
        """
        Node xử lý Viết truyện: Viết tiếp hoặc sửa đổi nội dung.
        """
        kb_context = self.context_node(story_id)
        
        full_context_parts = [kb_context]
        if previous_chapters:
            full_context_parts.append("# TÓM TẮT CÁC CHƯƠNG TRƯỚC\n" + previous_chapters)
        
        if chat_history:
            chat_str = "\n".join([f"{m['role'].upper()}: {m['content']}" for m in chat_history])
            full_context_parts.append("# Ý TƯỞNG TỪ CHAT\n" + chat_str)

        full_context_parts.append("# NỘI DUNG HIỆN TẠI\n" + current_content)
        
        system_prompt = (
            "Bạn là nhà văn chuyên nghiệp. Hãy viết tiếp hoặc sửa đổi dựa trên các kiến thức và chỉ dẫn sau.\n"
            "Tuyệt đối bám sát các Quy tắc, Nhân vật và thông tin bối cảnh đã cung cấp.\n\n"
            "\n\n".join(full_context_parts)
        )
        
        if selection:
            prompt = f"Phần văn bản được chọn: \"{selection}\"\n\nChỉ dẫn: {instruction}\n\nChỉ trả về nội dung mới, không giải thích."
        else:
            prompt = f"Chỉ dẫn viết tiếp: {instruction}\n\nChỉ trả về nội dung mới, không giải thích."

        contents = [
            types.Content(role="system", parts=[types.Part.from_text(text=system_prompt)]),
            types.Content(role="user", parts=[types.Part.from_text(text=prompt)])
        ]

        async def stream_gen():
            for chunk in self.client.models.generate_content_stream(
                model=self.model,
                contents=contents,
                config=self._get_config(),
            ):
                if text := chunk.text:
                    yield text
        
        return stream_gen()
