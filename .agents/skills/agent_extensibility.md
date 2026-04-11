# Skill: Mở rộng Agent (Nodes & Tools) 🤖

Kỹ năng này hướng dẫn cách thêm hoặc sửa đổi các thành phần cốt lõi của hệ thống AI Agent, bao gồm các bước xử lý (Nodes) và các công cụ thực thi (Tools).

---

## 1. Mở rộng Nodes (Pipeline Nodes)

Hệ thống sử dụng kiến trúc Node-based để xử lý yêu cầu qua nhiều bước. Các node nằm tại `src-tauri/src/ai/nodes/`.

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
    // Logic của node: Có thể gọi AI bằng run_agent_loop hoặc xử lý logic riêng
    run_agent_loop(state, cancel_state, 1, "reviewing").await
}
```

### Bước 2: Đăng ký Node
Thêm module vào `src-tauri/src/ai/nodes/mod.rs`:
```rust
pub mod review;
```

### Bước 3: Tích hợp vào luồng xử lý
Cập nhật `src-tauri/src/ai/chat.rs` trong hàm `ai_chat` để gọi node mới tại vị trí mong muốn:
```rust
app_handle.emit("ai-agent-step", "review").ok();
review_step(&mut state, cancel_state.clone()).await?;
```

---

## 2. Mở rộng Tools (Function Calling)

Tools là các hàm Rust mà AI có thể tự động gọi để thực hiện hành động.

### Bước 1: Định nghĩa logic tool
Tại `src-tauri/src/ai/tools.rs`, thêm hàm xử lý:
```rust
pub fn tool_new_action(root_path: &str, param: &str) -> Result<String, String> {
    // Xử lý logic...
    Ok("Thành công".to_string())
}
```

### Bước 2: Khai báo định dạng cho AI
Trong `src-tauri/src/ai/tools.rs`, thêm khai báo vào `get_tool_declarations()`:
```rust
FunctionDecl {
    name: "new_action".to_string(),
    description: "Mô tả khi nào AI nên dùng tool này".to_string(),
    parameters: Schema {
        field_type: "object".to_string(),
        properties: Some({
            let mut p = HashMap::new();
            p.insert("param".to_string(), Schema {
                field_type: "string".to_string(),
                description: Some("Mục đích của tham số"),
                ..Default::default()
            });
            p
        }),
        required: Some(vec!["param".to_string()]),
        ..Default::default()
    },
}
```

### Bước 3: Xử lý thực thi tool
Tại `src-tauri/src/ai/nodes/mod.rs`, cập nhật hàm `execute_tool_calls`:
```rust
"new_action" => {
    let param = fc.args["param"].as_str().unwrap_or("");
    tools::tool_new_action(&state.root_path, param)
}
```

---

## 3. Cập nhật Coordinator

Để AI biết khi nào cần dùng Agent hoặc logic mới, hãy cập nhật `COORDINATOR_SYSTEM_PROMPT` tại `src-tauri/src/ai/nodes/coordinate.rs`.

> [!IMPORTANT]
> - **Thought Stream**: Luôn đảm bảo Node hoặc Tool của bạn phát ra các sự kiện `ai-chat-stream-thought` để người dùng biết AI đang làm gì.
> - **Cancellation**: Luôn truyền `cancel_state` và kiểm tra `cancel_state.is_cancelled()` trong các vòng lặp nặng.
> - **Error Handling**: Sử dụng `Result<Ok, Err>` để hệ thống có thể báo lỗi về UI một cách thân thiện.

---

## Thư mục quan trọng
- `src-tauri/src/ai/nodes/`: Các bước xử lý.
- `src-tauri/src/ai/tools.rs`: Định nghĩa tools.
- `src-tauri/src/ai/chat.rs`: Điều phối luồng chính.
- `src-tauri/src/ai/instructions.rs`: Prompts cho các Agent.
