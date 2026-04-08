# Rust & Tauri Best Practices for AI Novelist

## Code Style & Quality (Rust)
- Sử dụng `snake_case` cho variables/functions/modules.
- Sử dụng `PascalCase` cho Structs/Enums.
- Chạy formatter chuẩn trong Rust.
- Xử lý lỗi tinh tế qua cấu trúc `Result<T, E>`.
- **Luôn chạy `cargo check` và `cargo clippy` sau khi sửa đổi code Rust để đảm bảo không có lỗi cú pháp và tuân thủ best practices.**
- Tuyệt đối không để lại code thừa, unused imports hoặc warnings không cần thiết.

## Tauri Architecture
- Tách biệt rõ ràng giao thức: giao diện (Vanilla JS) không gọi API trực tiếp, mà sẽ thông qua `invoke` (hoặc listen tới các events) để backend `Rust` thực hiện.
- Các API endpoints cho UI phải được đánh dấu bằng macro `#[tauri::command]`.
- Phân luồng luồng tải dài (Ví dụ gọi Gemini AI API) và xuất response dạng Streaming thông qua cơ chế event-emitting của Webview.

## File System (FS Management)
- Ứng dụng này sử dụng kiến trúc hoàn toàn dựa vào **File system** trên desktop thay cho cơ sở dữ liệu ảo. 
- Xử lý kỹ I/O Error khi đọc ghi file, thư mục truyện. Không ngầm định các folder đã tồn tại.

## AI / Gemini Integration
- Mọi logic API giao tiếp với Google Gemini nằm trong `src-tauri/src/ai/`.
- Serialize/Deserialize dữ liệu linh hoạt, mạnh mẽ với `serde` và `serde_json`.
- Chú ý cấu trúc cho luồng tư duy (thinking flow) của AI khi phân giải Response, cần kiểm tra block có hỗ trợ hiện luồng tư duy và ẩn chúng sau khi sinh ra final text.
