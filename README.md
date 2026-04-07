# AI_Write_Novel 🚀 - Trợ lý Sáng tác Truyện Thông minh

Chào mừng bạn đến với **AI_Write_Novel**, một ứng dụng Desktop hỗ trợ sáng tác truyện được phát triển bằng cấu trúc **Tauri (Rust + Vanilla JS)** và tích hợp các mô hình ngôn ngữ tiên tiến nhất của Google (Gemini 3.1 Pro/Gemma). Hệ thống lưu trữ sử dụng thư mục và tệp văn bản (file-based) thay vì CSDL, giúp bạn dễ dàng quản lý theo phong cách thư mục cây chuẩn mực.

---

## 🌟 Tính năng chính

- **Chat Trợ lý Tác giả**: Thảo luận ý tưởng, xây dựng cốt truyện, nhân vật và bối cảnh.
- **Agent Viết truyện tự động**: Tự động sinh nội dung chương mới.
- **Quản lý Truyện theo Thư mục**: Truyện và các chương truyện được lưu trữ và kết xuất trực tiếp dưới dạng tệp thư mục/file gốc, thay cho SQLite, giúp kiểm soát dễ dàng.
- **Tauri Desktop App**: Giao diện Desktop nhẹ nhàng nhưng vô cùng mạnh mẽ trong quản lý dữ liệu nhờ tốc độ của Rust Backend.
- **Streaming Response**: Hiển thị phản hồi từ AI ngay lập tức dưới dạng stream, có hỗ trợ xem luồng "suy nghĩ" (Thinking blocks) của Gemini.

---

## 🛠️ Công nghệ sử dụng

- **Backend / Core**: [Rust](https://www.rust-lang.org/) & [Tauri](https://tauri.app/)
- **Frontend**: Vanilla HTML / JS / CSS
- **Lưu trữ**: File-based (Native Rust FS Manager)
- **AI Model**: Google Gemini API (Có hỗ trợ cho Thinking workflow)

---

## 🚀 Hướng dẫn cài đặt

### 1. Chuẩn bị môi trường
Đảm bảo bạn đã cài đặt:
- **Node.js**
- **Rust & Cargo** (cài từ rustup)
- Các công cụ C++ / WebView được Tauri yêu cầu.

### 2. Cài đặt thư viện
Tại thư mục chứa dự án:
```bash
npm install
```

### 3. Cấu hình
Khóa API và các thiết lập mô hình có thể được thiết lập qua giao diện sử dụng hoặc tệp cấu hình sinh ra trong trình quản lý ứng dụng, tùy vào phiên bản ứng dụng hiện tại.

---

## 📖 Cách sử dụng

1. Khởi chạy ứng dụng phát triển:
   ```bash
   npm run tauri dev
   ```
2. Giao diện Desktop sẽ mở ra. Tại đây, hãy nhấp vào nút "Mở Truyện" hoặc khởi tạo thư mục mới để làm "Không gian viết" (Workspace).
3. Thêm chương và tương tác cùng AI trên khung chat để sáng tác trải nghiệm.

---

## 📂 Cấu trúc dự án

- `src-tauri/`: Chứa mã nguồn cho Core Rust và Tauri Configuration.
  - `src/ai/`: Cấu trúc request đa Agent, cấu hình Gemini API (`api_client`, `gemini_types`, `tools`).
  - `src/fs_manager.rs`: (hoặc nhánh fs structure) chuyên xử lý file/thư mục cứng của máy.
- `src/`: Giao diện ứng dụng cung cấp cho WebView (index.html, JS, CSS).
- `.agents/`: Các quy định về hành vi tạo sinh code dành cho hệ thống AI Development (mô hình nội bộ).

---

## 🧩 Agentic Configuration

Dự án này sử dụng mô hình Agentic để tối ưu hóa khả năng sáng tác và phát triển của AI phụ tá:
- **Rules**: `.agents/rules/` chứa các tiêu chuẩn Code.
- **Workflows**: `.agents/workflows/` định nghĩa các quy trình làm việc/thao tác cho công việc sinh truyện.
- **Skills**: `.agents/skills/` lưu trữ các kỹ năng cốt lõi.

---

**Chúc bạn có những giây phút sáng tác tuyệt vời!** ✨
