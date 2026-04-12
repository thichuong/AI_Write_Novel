# Skill: Mở rộng Agent (Nodes & Tools) 🤖

Kỹ năng này hướng dẫn cách thêm hoặc sửa đổi các thành phần cốt lõi của hệ thống AI Agent, bao gồm các bước xử lý (Nodes) và các công cụ thực thi (Tools).

---

## 1. Mở rộng Nodes (Pipeline Nodes)

Hệ thống sử dụng kiến trúc Node-based. Các node nằm tại `src-tauri/src/ai/nodes/`.

### Bước 1: Tạo Node mới
Tạo một tệp Rust mới trong `src-tauri/src/ai/nodes/` (ví dụ: `review.rs`).
```rust
use crate::ai::nodes::{AgentState, run_agent_loop};
use crate::ai::cancellation::CancellationState;
use tauri::State;

pub async fn review_step(
    state: &mut AgentState,
    cancel_state: State<'_, CancellationState>,
) -> Result<(), String> {
    // Logic của node:
    // param 3: max_local_loops
    // param 4: phase name (hiển thị trên UI)
    // param 5: allow_tools (true/false)
    run_agent_loop(state, cancel_state, 1, "reviewing", false).await
}
```

### Bước 2: Đăng ký Node
Thêm module vào `src-tauri/src/ai/nodes/mod.rs`:
```rust
pub mod review;
```

### Bước 3: Tích hợp vào luồng xử lý
Cập nhật `src-tauri/src/ai/chat.rs` hoặc trình điều phối để gọi node mới.

---

## 2. Xử lý JSON & Tự động hóa

Khi một Node trả về dữ liệu cấu trúc (như `Thinking` node), bạn nên sử dụng helper để trích xuất JSON.

### Ví dụ trích xuất JSON:
```rust
if let Some(json_text) = crate::ai::nodes::extract_json_block(&full_text) {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_text) {
        let content = parsed["chapter_content"].as_str().unwrap_or("");
        // Xử lý tự động lưu file hoặc cập nhật state...
    }
}
```

---

## 3. Mở rộng Tools (Function Calling)

Tools là các hàm Rust mà AI có thể gọi trong các node có `allow_tools: true`.

### Bước 1: Định nghĩa logic tool
Tại `src-tauri/src/ai/tools.rs`, thêm hàm xử lý.

### Bước 2: Khai báo định dạng cho AI
Trong `src-tauri/src/ai/tools.rs`, bổ sung vào `get_tool_declarations()`.

### Bước 3: Thực thi tool
Tại `src-tauri/src/ai/nodes/mod.rs`, cập nhật hàm `execute_tool_calls`:
```rust
"new_action" => {
    let param = fc.args["param"].as_str().unwrap_or("");
    tools::tool_new_action(&state.root_path, param)
}
```

---

## 4. Quy tắc sử dụng Tools trong Node

> [!IMPORTANT]
> - **Nghiên cứu (Analyze)**: Nên đặt `allow_tools: true` để Agent có thể đọc file và tìm kiếm.
> - **Sáng tác/Tư duy (Thinking)**: Nên đặt `allow_tools: false`. Điều này ép Model tập trung vào khả năng suy luận sáng tạo thay vì lạm dụng tool. Kết quả nên trả về JSON.
> - **Thực thi (Execute)**: Phải đặt `allow_tools: true` để Agent thực hiện thay đổi hệ thống.

---

## Thư mục quan trọng
- `src-tauri/src/ai/nodes/`: Nơi định nghĩa các bước xử lý (Analyze, Thinking, Execute, Finalize).
- `src-tauri/src/ai/tools.rs`: Nơi định nghĩa và khai báo tools.
- `src-tauri/src/ai/nodes/mod.rs`: Chứa `run_agent_loop` và `execute_tool_calls`.
