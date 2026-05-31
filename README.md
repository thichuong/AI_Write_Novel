# AI_Write_Novel 🚀 - Trợ lý Sáng tác Truyện Thông minh

Chào mừng bạn đến với **AI_Write_Novel**, một ứng dụng Desktop chuyên nghiệp hỗ trợ sáng tác truyện được phát triển bằng cấu trúc **Tauri (Rust + Vanilla JS)** và tích hợp các mô hình ngôn ngữ tiên tiến nhất (Gemini 1.5/2.0/3.5). Hệ thống được thiết kế để trở thành một "đồng tác giả" thực thụ, có khả năng điều phối các Agent chuyên biệt để quản lý cốt truyện, nhân vật và sáng tác.

---

## 🌟 Tính năng chính

- **3 Chế độ Chuyên biệt Tinh gọn**:
  - **Chat (Trò chuyện & Tra cứu - 1 bước)**: Hỏi đáp, trò chuyện tự nhiên, tra cứu kiến thức thực tế hoặc đọc chương truyện cũ.
  - **Ide (Lên ý tưởng - 1 bước thông minh)**: Brainstorm ý tưởng, thiết lập thế giới, tự động tạo mới hoặc cập nhật các trang Wiki nhân vật/thế giới/lore/cốt truyện và `memory.md` trực tiếp thông qua gọi tool.
  - **Writing (Sáng tác & Đồng bộ - 2 bước)**: Sáng tác chương truyện mới chất lượng cao và tự động lưu file bằng Rust (Bước 1), sau đó tự động rà soát đồng bộ hóa Wiki và tóm tắt tiến trình vào `memory.md` (Bước 2).
- **Cơ chế Tự động Nạp Bối cảnh (Auto-Context Injection)**: Backend Rust tự động gọi thu thập cấu trúc thư mục, danh sách thực thể Wiki hiện có và nội dung file `memory.md` để nạp trực tiếp vào System Instruction của AI. Không cần bước phân tích trung gian cồng kềnh, tăng tốc độ phản hồi lên **2.5x đến 5x**!
- **Suy nghĩ thời gian thực (Real-time Reasoning)**: Hiển thị minh bạch quá trình tư duy qua các "Thought Blocks" khi AI đang brainstorm, sáng tác hoặc đồng bộ hóa Wiki.
- **Tích hợp Google Search**: Kết hợp sức mạnh tìm kiếm thực tế trực tiếp từ Gemini khi giải đáp các thắc mắc của tác giả.
- **Wiki Graph Knowledge Base**: Quản lý kiến thức nhân vật, bối cảnh thế giới, lore truyền thuyết và cốt truyện một cách logic trong thư mục `wiki/`.
- **Long-term Memory & Context Optimization**: Tự động duy trì bộ nhớ dự án qua `memory.md` và tối ưu hóa Token qua cơ chế Pruning.

---

## 🏗️ Kiến trúc Hệ thống

Xem chi tiết tại [architecture.md](architecture.md). Ứng dụng tuân thủ mô hình **Layered Event-Driven Architecture**:
- **Frontend**: Vanilla JS (Phân tách UI/UX, lắng nghe các sự kiện stream và thought).
- **Backend**: Rust Core (Điều phối Multi-Agent, tự động nạp bối cảnh, quản lý File System và Tool Calling).

---

## 🛠️ Công nghệ sử dụng

- **Core**: [Rust](https://www.rust-lang.org/) & [Tauri](https://tauri.app/)
- **Frontend**: HTML / Vanilla JS / CSS (Modern Glassmorphism Design)
- **AI Models**: Google Gemini API (Tích hợp Thinking Level & Search).

---

## 🚀 Hướng dẫn cài đặt

### 1. Chuẩn bị môi trường
- **Node.js** (LTS recommend)
- **Rust & Cargo**

### 2. Cài đặt và Khởi chạy
```bash
npm install
npm run tauri:dev
```

### 3. Thiết lập ban đầu
- Vào tab **Cài đặt** trên sidebar để nhập **API Key** và chọn **Model**.
- Sử dụng nút "Mở Truyện" để chọn không gian làm việc của bạn.

---

## 📂 Cấu trúc dự án

- `src-tauri/src/ai/`: Cốt lõi hệ thống AI.
  - `nodes/`: Các bước xử lý chính của Agentic Flow.
    - `thinking.rs`: Bước Sáng tác chương truyện của Writing.
    - `finalize.rs`: Bước Đồng bộ thực thể Wiki & Memory của Writing.
    - `mod.rs`: Quản lý AgentState, run_agent_loop, prune_history và dọn dẹp bộ nhớ.
  - `instructions.rs`: Tập hợp các chỉ dẫn hệ thống (System Instructions) chuyên biệt cho từng Agent.
  - `tools.rs`: Định nghĩa các công cụ tương tác hệ thống (Wiki, Google Search, File System).
- `wiki/`: Cơ sở dữ liệu kiến thức (Nhân vật, Thế giới, Lore, Cốt truyện).
- `chapters/`: Lưu trữ các chương truyện.
- `memory.md`: Bộ nhớ dài hạn lưu trữ tóm tắt trạng thái của tác phẩm.

---

**Chúc bạn có những giây phút sáng tác tuyệt vời cùng vị phụ tá AI thông minh!** ✨
