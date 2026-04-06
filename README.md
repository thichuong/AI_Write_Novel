# AI_Write_Novel 🚀 - Trợ lý Sáng tác Truyện Thông minh

Chào mừng bạn đến với **AI_Write_Novel**, một ứng dụng hỗ trợ sáng tác truyện được xây dựng bằng Python (FastAPI) và tích hợp các mô hình ngôn ngữ tiên tiến của Google (Gemini/Gemma). Hệ thống được thiết kế với kiến trúc đa Agent để giúp bạn từ khâu lên ý tưởng đến việc viết chi tiết từng chương.

---

## 🌟 Tính năng chính

- **Chat Trợ lý Tác giả**: Thảo luận ý tưởng, xây dựng cốt truyện, nhân vật và bối cảnh với AI có kiến thức về toàn bộ câu chuyện hiện tại.
- **Agent Viết truyện tự động**: Tự động sinh nội dung chương mới dựa trên tóm tắt các chương trước và lịch sử thảo luận trong chat.
- **Chỉnh sửa thông minh**: Sửa đổi các đoạn văn bản được chọn với các chỉ dẫn cụ thể (ví dụ: "Viết lại đoạn này cho cảm động hơn").
- **Quản lý Chương truyện**: Giao diện trực quan để tạo, lưu và quản lý danh sách các chương trong một bộ truyện.
- **Streaming Response**: Hiển thị phản hồi từ AI ngay lập tức dưới dạng stream, mang lại trải nghiệm mượt mà.

---

## 🛠️ Công nghệ sử dụng

- **Backend**: [FastAPI](https://fastapi.tiangolo.com/) (Python)
- **Database**: [SQLite](https://sqlite.org/) (Sử dụng thư viện `sqlite3`)
- **AI Model**: Google Gemini API (Gemma-4-31B-IT)
- **Frontend**: Vanilla HTML/JS/CSS (Phục vụ qua FastAPI Static Files)

---

## 🚀 Hướng dẫn cài đặt

### 1. Chuẩn bị môi trường
Đảm bảo bạn đã cài đặt Python 3.9+.

### 2. Cài đặt thư viện
```bash
pip install fastapi uvicorn google-genai python-dotenv pydantic
```

### 3. Cấu hình API Key
Tạo file `.env` tại thư mục gốc và thêm khóa API của bạn:
```env
GEMINI_API_KEY=YOUR_API_KEY_HERE
```

---

## 📖 Cách sử dụng

1. Chạy server backend:
   ```bash
   python main.py
   ```
2. Truy cập ứng dụng tại: `http://localhost:8000`
3. Bắt đầu tạo truyện mới, thêm chương và sử dụng khung chat bên trái để lên ý tưởng cùng AI.

---

## 📂 Cấu trúc dự án

- `main.py`: Entry point của ứng dụng, chứa các REST API endpoints.
- `agents.py`: Lớp `AIWriter` xử lý logic tương tác với Google GenAI.
- `database.py`: Quản lý kết nối và khởi tạo cơ sở dữ liệu SQLite.
- `static/`: Chứa mã nguồn frontend (HTML, CSS, JS).
- `.agents/`: Thư mục lưu trữ các bộ quy tắc (rules), quy trình (workflows) và kỹ năng (skills) cho AI Agent.
- `data/`: Lưu trữ dữ liệu bộ truyện (nếu có).

---

## 🧩 Agentic Configuration

Dự án này sử dụng mô hình Agentic để tối ưu hóa khả năng sáng tác:
- **Rules**: `.agents/rules/` chứa các tiêu chuẩn lập trình và hành vi của Agent.
- **Workflows**: `.agents/workflows/` định nghĩa các quy trình làm việc phức tạp.
- **Skills**: `.agents/skills/` lưu trữ các kỹ năng chuyên biệt mà Agent có thể học hỏi.

---

**Chúc bạn có những giây phút sáng tác tuyệt vời!** ✨
