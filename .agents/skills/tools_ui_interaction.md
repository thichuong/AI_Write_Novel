# Skill: Cập nhật Tools và Tương tác UI-Rust 🛠️

Kỹ năng này hướng dẫn cách mở rộng khả năng của Agent bằng cách thêm công cụ mới trong Rust và đồng bộ hóa với giao diện người dùng (Frontend).

## 1. Thêm Tool mới trong Rust Backend

Khi cần thêm một khả năng mới (ví dụ: tìm kiếm nâng cao, xuất PDF), hãy thực hiện tại `src-tauri/src/ai/tools.rs`:

### Bước 1: Định nghĩa logic thực thi
Tạo một hàm Rust xử lý công việc cụ thể.
```rust
pub fn tool_custom_action(arg: &str) -> Result<String, String> {
    // Logic xử lý ở đây
    Ok(format!("Kết quả: {}", arg))
}
```

### Bước 2: Khai báo JSON Schema cho Gemini
Thêm hàm khai báo (Declaration) tương ứng để Gemini có thể hiểu cách gọi.
```rust
fn decl_custom_action() -> FunctionDecl {
    FunctionDecl {
        name: "custom_action".to_string(),
        description: "Mô tả hành động này để AI biết khi nào cần dùng.".to_string(),
        parameters: Schema {
            field_type: "object".to_string(),
            properties: Some({
                let mut p = HashMap::new();
                p.insert("arg".to_string(), Schema {
                    field_type: "string".to_string(),
                    description: Some("Mô tả tham số.".to_string()),
                    // ... các trường khác
                });
                p
            }),
            required: Some(vec!["arg".to_string()]),
            // ...
        },
    }
}
```

### Bước 3: Đăng ký vào `get_tool_declarations`
Thêm khai báo vào danh sách trả về trong hàm `get_tool_declarations()`.

---

## 2. Tương tác với UI qua Events

Để UI cập nhật ngay khi Agent thay đổi dữ liệu (ví dụ: write file), sử dụng `Emitter`.

- **Trong Rust**:
```rust
use tauri::Emitter;
// ... trong tool function
app_handle.emit("event-name", json!({ "data": "value" })).ok();
```

- **Trong UI (JS)**:
```javascript
import { listen } from '@tauri-apps/api/event';

listen('event-name', (event) => {
  console.log("Nhận dữ liệu từ Rust:", event.payload);
  // Cập nhật DOM hoặc State ở đây
});
```

---

## 3. Quy trình "Bridge" mẫu

Nếu bạn muốn tạo một nút bấm trên UI kích hoạt Agent làm gì đó:
1. **Frontend**: `invoke('tauri_command_name', { args })`.
2. **Backend (lib.rs)**: Gọi `ai::chat::ai_chat`.
3. **Agent Loop**: AI quyết định gọi `tool_xyz`.
4. **Tool xyz**: Thực thi logic -> `emit('refresh-ui')`.
5. **Frontend**: Nhận event và làm mới giao diện.

> [!IMPORTANT]
> Luôn đảm bảo tên tool trong `FunctionDecl` khớp chính xác với tên mà bạn xử lý trong vòng lặp Agent (`run_agent_loop`).
