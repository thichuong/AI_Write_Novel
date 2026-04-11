# AI_Write_Novel 🚀 - Trợ lý Sáng tác Truyện Thông minh

Chào mừng bạn đến với **AI_Write_Novel**, một ứng dụng Desktop chuyên nghiệp hỗ trợ sáng tác truyện được phát triển bằng cấu trúc **Tauri (Rust + Vanilla JS)** và tích hợp các mô hình ngôn ngữ tiên tiến nhất (Gemini 2.0 Flash / Gemma). Hệ thống được thiết kế để trở thành một "đồng tác giả" thực thụ, không chỉ dừng lại ở việc gợi ý text mà còn có khả năng quản lý toàn bộ cấu trúc truyện, nhân vật và kiến thức thế giới.

---

## 🌟 Tính năng chính

- **Luồng Agentic Đa bước**: AI không trả lời ngay lập tức mà tuân thủ quy trình: Phân tích yêu cầu -> Thực thi công cụ -> Tổng hợp kết quả.
- **Tự động hóa IDE**: AI có khả năng tự động tạo, sửa và mở các chương truyện trong Editor ngay khi người dùng yêu cầu.
- **Wiki Graph Knowledge Base**: Quản lý kiến thức nhân vật, thế giới, cốt truyện một cách logic trong thư mục `wiki/` dưới dạng Markdown với Frontmatter.
- **Long-term Memory**: Agent duy trì bộ nhớ dài hạn thông qua tệp `memory.md`, giúp ghi nhớ các quyết định và tiến độ sáng tác.
- **Streaming & Thought Block**: Hiển thị luồng suy nghĩ của AI (Thinking blocks) và phản hồi thời gian thực, mang lại trải nghiệm tương tác minh bạch.
- **Quản lý API Key an toàn**: Thiết lập API Key trực tiếp qua giao diện ứng dụng, tự động lưu trữ vào cấu hình môi trường.

---

## 🏗️ Kiến trúc Hệ thống

Xem chi tiết tại [architecture.md](architecture.md). Ứng dụng tuân thủ mô hình **Layered Event-Driven Architecture**:
- **Frontend**: Vanilla JS (Phản ứng với các sự kiện từ Backend để cập nhật UI/Editor).
- **Bridge**: Tauri Commands & Events (Cơ chế đẩy dữ liệu real-time từ Rust sang JS).
- **Backend**: Rust Core (Điều phối Agent Loop và quản lý File System).

---

## 🛠️ Công nghệ sử dụng

- **Core**: [Rust](https://www.rust-lang.org/) & [Tauri](https://tauri.app/)
- **Frontend**: HTML / Vanilla JS / CSS (Modern Glassmorphism Design)
- **AI Models**: Google Gemini API (Hỗ trợ Gemini 2.0 Flash, Gemma, và tính năng Google Search/Thinking)
- **State Management**: File-based (Native Rust FS Manager)

---

## 🚀 Hướng dẫn cài đặt

### 1. Chuẩn bị môi trường
- **Node.js** (LTS recommend)
- **Rust & Cargo**
- Cấu hình WebView tùy theo hệ điều hành (Linux/Windows/macOS).

### 2. Cài đặt và Khởi chạy
Tại thư mục dự án:
```bash
npm install
npm run tauri dev
```

### 3. Thiết lập ban đầu
- Khi ứng dụng mở ra, nếu thiếu API Key, một khung nhập liệu sẽ xuất hiện. Nhập **Gemini API Key** của bạn để bắt đầu.
- Nhấp "Mở Truyện" hoặc chọn một thư mục trống để làm không gian làm việc.

---

## 📂 Cấu trúc dự án

- `src-tauri/`: Mã nguồn Rust Core.
  - `src/ai/`: Agentic logic, API client và hệ thống Tools.
  - `src/fs/`: Xử lý tệp tin và dữ liệu truyện.
- `src/`: Mã nguồn Frontend (UI/UX).
- `wiki/`: Cơ sở dữ liệu kiến thức (Nhân vật, Thế giới, v.v.).
- `chapters/`: Nơi lưu trữ các chương truyện (.md).
- `memory.md`: Nhật ký công việc và bộ nhớ của Agent.

---

## 🧩 Agentic Configuration

Hệ thống Agent được xây dựng dựa trên các quy định tại `.agents/`:
- **Rules**: `.agents/rules/` - Tiêu chuẩn và phong cách viết code.
- **Workflows**: `.agents/workflows/` - Quy trình làm việc tự động.
- **Skills**: Các kỹ năng cốt lõi được tài liệu hóa:
  - [Cập nhật Tools & UI Interaction](.agents/skills/tools_ui_interaction.md)
  - [Quản lý Wiki Graph](.agents/skills/wiki_graph_agent.md)

---

**Chúc bạn có những giây phút sáng tác tuyệt vời cùng vị phụ tá AI thông minh!** ✨
