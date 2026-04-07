---
description: Quy trình thêm và viết chương mới cho truyện với sự hỗ trợ của AI.
---

1. Chọn hoặc Khởi tạo mới một thư mục Truyện trên giao diện Desktop (Sử dụng Hộp thoại chọn thư mục OS qua Tauri).
2. Nhấn nút "Tạo chương" (Add Chapter) để khởi tạo file lưu trữ chuẩn (`.txt` hoặc `.md`) vào hệ thống File cục bộ bằng Rust Core.
3. Sử dụng khung chat Chatbot Trợ lý Tác giả (Giao diện Frontend) để lên kịch bản, ý tưởng hoặc tóm tắt nội dung chương đó.
4. Backend Tauri (khối `ai_agent`) nhận dữ liệu, tuỳ chỉnh các thông số trong request (như thiết lập Cấu hình Thinking model nếu yêu cầu luồng lý luận cao độ) và gửi đi.
5. Khi Agent "Viết truyện" bắt đầu làm việc, nó sẽ đẩy nội dung mới (Streaming chunks) qua event xuống để frontend hiển thị theo thời gian thực.
6. Tác giả tự do xem lại nội dung, sửa thủ công vào hệ thống tệp và tiếp tục với chu kỳ này.
