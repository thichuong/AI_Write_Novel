import os
from google import genai
from google.genai import types
from dotenv import load_dotenv

load_dotenv()

class AIWriter:
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

    async def generate_chat_response(self, story_context, chat_history):
        """
        Agent Chat Bot: Hỗ trợ lên ý tưởng, thấy rõ phần viết truyện.
        """
        system_prompt = (
            "Bạn là một trợ lý sáng tác truyện chuyên nghiệp. Nhiệm vụ của bạn là hỗ trợ tác giả lên ý tưởng, "
            "xây dựng cốt truyện, nhân vật và bối cảnh. Bạn có quyền truy cập vào nội dung truyện hiện tại để tham khảo.\n\n"
            f"Nội dung truyện hiện tại:\n{story_context}\n"
        )
        
        contents = [types.Content(role="system", parts=[types.Part.from_text(text=system_prompt)])]
        for msg in chat_history:
            contents.append(types.Content(role=msg['role'], parts=[types.Part.from_text(text=msg['content'])]))

        # Generator for streaming
        def stream_gen():
            for chunk in self.client.models.generate_content_stream(
                model=self.model,
                contents=contents,
                config=self._get_config(),
            ):
                if text := chunk.text:
                    yield text
        
        return stream_gen()

    async def generate_writing_response(self, story_context, instruction, selection_context=None, previous_chapters=None, chat_history=None):
        """
        AI Agent Viết truyện: Viết tiếp hoặc sửa lại dựa trên chỉ dẫn và ngữ cảnh.
        Bao gồm tất cả các chương trước đó và lịch sử chat nếu được cung cấp.
        """
        
        context_parts = []
        if previous_chapters:
            context_parts.append("# TÓM TẮT CÁC CHƯƠNG TRƯỚC ĐÓ\n" + previous_chapters)
        
        if chat_history:
            chat_str = "\n".join([f"{m['role'].upper()}: {m['content']}" for m in chat_history])
            context_parts.append("# CÁC Ý TƯỞNG ĐÃ THẢO LUẬN TRONG CHAT\n" + chat_str)
        
        context_parts.append("# NỘI DUNG CHƯƠNG HIỆN TẠI\n" + story_context)
        
        full_context_str = "\n\n".join(context_parts)

        system_prompt = (
            "Bạn là một nhà văn chuyên nghiệp. Nhiệm vụ của bạn là viết tiếp hoặc sửa đổi văn bản dựa trên chỉ dẫn.\n"
            "Hãy đảm bảo sự nhất quán với các chương trước đó và các ý tưởng đã thảo luận trong chat.\n\n"
            f"{full_context_str}\n"
        )
        
        if selection_context:
            prompt = f"Dựa trên ngữ cảnh trên, phần văn bản được chọn là:\n\"{selection_context}\"\n\nChỉ dẫn: {instruction}\n\nLưu ý: Chỉ trả về nội dung văn bản mới, không thêm lời giải thích."
        else:
            prompt = f"Dựa trên tất cả các chương trước và ý tưởng thảo luận, hãy viết tiếp chương hiện tại.\n\nChỉ dẫn: {instruction}\n\nLưu ý: Chỉ trả về nội dung văn bản mới, không thêm lời giải thích."

        contents = [
            types.Content(role="system", parts=[types.Part.from_text(text=system_prompt)]),
            types.Content(role="user", parts=[types.Part.from_text(text=prompt)])
        ]

        def stream_gen():
            for chunk in self.client.models.generate_content_stream(
                model=self.model,
                contents=contents,
                config=self._get_config(),
            ):
                if text := chunk.text:
                    yield text
        
        return stream_gen()
