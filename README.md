# AI_Write_Novel 🚀 - Trợ lý Sáng tác Truyện Thông minh

Chào mừng bạn đến với **AI_Write_Novel**, một ứng dụng Desktop chuyên nghiệp hỗ trợ sáng tác truyện được phát triển bằng cấu trúc **Tauri (Rust + Vanilla JS)** và tích hợp các mô hình ngôn ngữ tiên tiến nhất (Gemma 4). Hệ thống được thiết kế để trở thành một "đồng tác giả" thực thụ, có khả năng điều phối các Agent chuyên biệt để quản lý cốt truyện, nhân vật và sáng tác.

---

## 🌟 Tính năng chính

- **Bộ điều phối thông minh (Smart Coordinator)**: Tự động phân tích yêu cầu để chọn Agent chuyên trách (Viết lách, Lên ý tưởng, hoặc Chat giải đáp).
- **Luồng Agentic Đa bước**: Quy trình xử lý chuyên sâu 4 giai đoạn: Phân tích bối cảnh -> Tư duy sáng tạo (Thinking) -> Thực thi hệ thống -> Tổng kết & Ghi nhớ.
- **Suy nghĩ thời gian thực (Real-time Reasoning)**: Hiển thị minh bạch quá trình tư duy qua các "Thought Blocks", bao gồm cả phác thảo nội dung và cập nhật Wiki.
- **Tích hợp Google Search**: Kết hợp sức mạnh tìm kiếm thực tế trong bước Phân tích để nạp dữ liệu chính xác nhất.
- **Wiki Graph Knowledge Base**: Quản lý kiến thức nhân vật, thế giới, cốt truyện một cách logic trong thư mục `.wiki/`.
- **Long-term Memory & Context Optimization**: Tự động duy trì bộ nhớ dự án qua `memory.md` và tối ưu hóa Token qua cơ chế Pruning.
- **Tự động hóa Hệ thống**: AI tự động lưu chương truyện và cập nhật Wiki dựa trên kết quả tư duy chuyển đổi qua JSON.

---

## 🏗️ Kiến trúc Hệ thống

Xem chi tiết tại [architecture.md](architecture.md). Ứng dụng tuân thủ mô hình **Layered Event-Driven Architecture**:
- **Frontend**: Vanilla JS (Phân tách UI/UX, lắng nghe sự kiện stream và thought).
- **Backend**: Rust Core (Điều phối Multi-Agent, quản lý File System và Tool Calling).

---

## 🛠️ Công nghệ sử dụng

- **Core**: [Rust](https://www.rust-lang.org/) & [Tauri](https://tauri.app/)
- **Frontend**: HTML / Vanilla JS / CSS (Modern Glassmorphism Design)
- **AI Models**: Google Gemini API (Hỗ trợ Gemini 2.0 Flash, Gemma, tích hợp Thinking Level & Search).

---

## 🚀 Hướng dẫn cài đặt

### 1. Chuẩn bị môi trường
- **Node.js** (LTS recommend)
- **Rust & Cargo**

### 2. Cài đặt và Khởi chạy
```bash
npm install
npm run tauri dev
```

### 3. Thiết lập ban đầu
- Vào tab **Cài đặt** trên sidebar để nhập **API Key** và chọn **Model**.
- Sử dụng nút "Mở Truyện" để chọn không gian làm việc của bạn.

---

## 📂 Cấu trúc dự án

- `src-tauri/src/ai/`: Cốt lõi hệ thống AI.
  - `nodes/`: Các bước xử lý trong Pipeline (Analyze, Coordinate, Execute, v.v.).
  - `instructions.rs`: Tập hợp các chỉ dẫn chuyên biệt cho từng Agent.
  - `tools.rs`: Định nghĩa các công cụ tương tác hệ thống.
- `.wiki/`: Cơ sở dữ liệu kiến thức (Nhân vật, Thế giới).
- `chapters/`: Lưu trữ các chương truyện.
- `memory.md`: Bộ nhớ dài hạn của Agent.

---

**Chúc bạn có những giây phút sáng tác tuyệt vời cùng vị phụ tá AI thông minh!** ✨
