# AI_Write_Novel 🚀 - Trợ lý Sáng tác Truyện Thông minh

Chào mừng bạn đến với **AI_Write_Novel**, một ứng dụng Desktop chuyên nghiệp hỗ trợ sáng tác truyện được phát triển bằng cấu trúc **Tauri (Rust + Vanilla JS)** và tích hợp các mô hình ngôn ngữ tiên tiến nhất (Gemini 2.0 Flash / Gemma). Hệ thống được thiết kế để trở thành một "đồng tác giả" thực thụ, không chỉ dừng lại ở việc gợi ý text mà còn có khả năng quản lý toàn bộ cấu trúc truyện, nhân vật và kiến thức thế giới.

---

## 🌟 Tính năng chính

- **Luồng Agentic Đa bước**: AI không trả lời ngay lập tức mà tuân thủ quy trình: Phân tích yêu cầu -> Thực thi công cụ -> Tổng hợp kết quả.
- **Tùy chọn Mô hình Linh hoạt**: Cho phép chuyển đổi giữa các mô hình mạnh mẽ (như Gemini 2.0 Flash, Gemma) tùy theo nhu cầu sáng tác.
- **Tự động hóa IDE**: AI có khả năng tự động tạo, sửa và mở các chương truyện trong Editor ngay khi người dùng yêu cầu.
- **Wiki Graph Knowledge Base**: Quản lý kiến thức nhân vật, thế giới, cốt truyện một cách logic trong thư mục `wiki/` dưới dạng Markdown với Frontmatter.
- **Long-term Memory**: Agent duy trì bộ nhớ dài hạn thông qua tệp `memory.md`, tự động cập nhật sau mỗi phiên làm việc.
- **Giao diện Chat Cải tiến**: Frame chat rộng rãi, hỗ trợ Thinking blocks, footer tương tác gộp nhóm và tùy chọn phím tắt gửi tin nhắn (Enter/Shift+Enter).
- **Quản lý Cấu hình Tập trung**: Tab Settings mới cho phép thiết lập API Key và Model một cách trực quan.

---

## 🏗️ Kiến trúc Hệ thống

Xem chi tiết tại [architecture.md](architecture.md). Ứng dụng tuân thủ mô hình **Layered Event-Driven Architecture**:
- **Frontend**: Vanilla JS (Phản ứng với các sự kiện từ Backend để cập nhật UI/Editor, quản lý State tập trung).
- **Bridge**: Tauri Commands & Events (Cơ chế đẩy dữ liệu real-time từ Rust sang JS).
- **Backend**: Rust Core (Điều phối Agent Loop, quản lý File System và gọi API AI).

---

## 🛠️ Công nghệ sử dụng

- **Core**: [Rust](https://www.rust-lang.org/) & [Tauri](https://tauri.app/)
- **Frontend**: HTML / Vanilla JS / CSS (Modern Glassmorphism Design)
- **AI Models**: Google Gemini API (Tích hợp đa mô hình: Gemini 2.0 Flash, Gemma, v.v.)
- **State Management**: File-based System (Native Rust FS Manager)

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
- Khi ứng dụng mở ra, vào tab **Cài đặt** (biểu tượng bánh răng) trên sidebar.
- Nhập **Gemini API Key** và chọn **Model** mong muốn.
- Nhấp "Mở Truyện" hoặc chọn một thư mục trống để làm không gian làm việc.

---

## 📂 Cấu trúc dự án

- `src-tauri/`: Mã nguồn Rust Core.
  - `src/ai/`: Agentic logic, API client và hệ thống Nodes/Tools.
  - `src/fs/`: Xử lý tệp tin và dữ liệu truyện.
- `src/`: Mã nguồn Frontend (UI/UX).
- `.wiki/`: Cơ sở dữ liệu kiến thức (Được đồng bộ và hiển thị trực tiếp trên File Explorer).
- `chapters/`: Nơi lưu trữ các chương truyện (.md).
- `memory.md`: Nhật ký công việc và bộ nhớ của Agent.
- `.chat_history.json`: Lưu trữ lịch sử trò chuyện của phiên hiện tại.

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
