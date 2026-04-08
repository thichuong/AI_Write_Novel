use super::api_client::{get_api_key, get_model, stream_gemini_response};
use super::gemini_types::{
    GeminiContent, GeminiPart, GeminiRequest, GenerationConfig, ThinkingConfig,
};
use crate::fs;
use tauri::AppHandle;

/// AI viết truyện (streaming) — có function calling
#[tauri::command]
pub async fn ai_write(
    app_handle: AppHandle,
    root_path: String,
    current_file: String,
    instruction: String,
    current_content: String,
    selection: Option<String>,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let model = get_model();

    // Lấy context
    let kb_context = fs::get_story_context(root_path.clone())?;
    let prev_chapters = fs::get_previous_chapters(root_path.clone(), current_file.clone())?;

    let mut full_context = kb_context;
    if !prev_chapters.is_empty() {
        full_context.push_str(&format!(
            "\n# TÓM TẮT CÁC CHƯƠNG TRƯỚC\n{}\n",
            prev_chapters
        ));
    }
    full_context.push_str(&format!(
        "\n# NỘI DUNG HIỆN TẠI ({})\n{}\n",
        current_file, current_content
    ));

    let agent_role = if current_file.starts_with("Chương/") || current_file.starts_with("Chương\\")
    {
        "Bạn là một NHÀ VĂN CHUYÊN NGHIỆP. Nhiệm vụ của bạn là VIẾT TIẾP hoặc CHỈNH SỬA nội dung CHƯƠNG TRUYỆN.\n\
         - Phong cách văn phong phải mạch lạc, lôi cuốn.\n\
         - Mô tả hành động, cảm xúc và bối cảnh sống động.\n\
         - Phát triển cốt truyện và giữ đúng tính cách nhân vật."
    } else if current_file.starts_with("Nhân vật/") || current_file.starts_with("Nhân vật\\")
    {
        "Bạn là một CHUYÊN GIA THIẾT KẾ NHÂN VẬT. Nhiệm vụ của bạn là TẠO MỚI hoặc CẬP NHẬT HỒ SƠ NHÂN VẬT.\n\
         - Trình bày rõ ràng: Ngoại hình, Tính cách, Tiểu sử, Kỹ năng/Sức mạnh, và Mục tiêu.\n\
         - Đảm bảo nhân vật có chiều sâu và phù hợp với toàn bộ thế giới của truyện."
    } else if current_file.starts_with("Cốt truyện/") || current_file.starts_with("Cốt truyện\\")
    {
        "Bạn là một CHUYÊN GIA LÊN KẾ HOẠCH CỐT TRUYỆN. Nhiệm vụ của bạn là PHÁT TRIỂN CỐT TRUYỆN (PLOT).\n\
         - Đề xuất các tình tiết hấp dẫn, cao trào, thắt nút và mở nút logic.\n\
         - Đảm bảo mạch truyện liên kết chặt chẽ từ đầu đến cuối."
    } else if current_file.starts_with("Bối cảnh/") || current_file.starts_with("Bối cảnh\\")
    {
        "Bạn là một CHUYÊN GIA XÂY DỰNG THẾ GIỚI (World Builder). Nhiệm vụ của bạn là MÔ TẢ BỐI CẢNH, ĐỊA DANH, VẬT PHẨM, HỆ THỐNG PHÉP THUẬT.\n\
         - Miêu tả chi tiết, mang lại cảm giác chân thực và sống động.\n\
         - Giữ vững logic của thế giới (world-building rules)."
    } else {
        "Bạn là một TRỢ LÝ SÁNG TÁC ĐA NĂNG. Nhiệm vụ của bạn là HỖ TRỢ XỬ LÝ NỘI DUNG theo chỉ dẫn của người dùng."
    };

    let system_prompt = format!(
        "{}\n\
         Hãy thực hiện dựa trên các kiến thức và chỉ dẫn dưới đây. \
         Tuyệt đối bám sát các Quy tắc, Nhân vật và thông tin bối cảnh đã cung cấp.\n\n\
         {}\n",
        agent_role, full_context
    );

    let user_prompt = if let Some(sel) = &selection {
        format!(
            "Phần văn bản được chọn: \"{}\"\n\nChỉ dẫn: {}\n\nChỉ trả về nội dung mới, không giải thích.",
            sel, instruction
        )
    } else {
        format!(
            "Chỉ dẫn viết tiếp: {}\n\nChỉ trả về nội dung mới, không giải thích.",
            instruction
        )
    };

    let contents = vec![
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: system_prompt,
            }],
        },
        GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart::Text {
                text: "Tôi đã nắm được toàn bộ ngữ cảnh. Sẵn sàng viết.".to_string(),
            }],
        },
        GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text { text: user_prompt }],
        },
    ];

    let request = GeminiRequest {
        contents,
        generation_config: Some(GenerationConfig {
            temperature: 0.9,
            max_output_tokens: 16384,
            thinking_config: Some(ThinkingConfig {
                thinking_level: "HIGH".to_string(),
            }),
        }),
        tools: None, // Writing mode — trả text trực tiếp để user preview
    };

    stream_gemini_response(&app_handle, &api_key, &model, &request, "ai-write-stream").await
}
