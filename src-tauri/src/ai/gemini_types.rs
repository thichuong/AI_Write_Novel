use serde::{Deserialize, Serialize};

/// Cấu trúc request gửi lên Gemini API
#[derive(Debug, Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<GeminiContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDeclaration>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum GeminiPart {
    Text {
        text: String,
    },
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: FunctionCallData,
    },
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: FunctionResponseData,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCallData {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionResponseData {
    pub name: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerationConfig {
    pub temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
    #[serde(rename = "thinkingConfig", skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThinkingConfig {
    #[serde(rename = "thinkingLevel")]
    pub thinking_level: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolDeclaration {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDecl>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub description: String,
    pub parameters: Schema,
}

#[derive(Debug, Serialize, Clone)]
pub struct Schema {
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<std::collections::HashMap<String, Self>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Self>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response từ Gemini streaming API
#[derive(Debug, Deserialize)]
pub struct GeminiStreamResponse {
    pub candidates: Option<Vec<Candidate>>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Option<CandidateContent>,
}

#[derive(Debug, Deserialize)]
pub struct CandidateContent {
    pub parts: Option<Vec<CandidatePart>>,
}

#[derive(Debug, Deserialize)]
pub struct CandidatePart {
    pub text: Option<String>,
    #[serde(rename = "functionCall")]
    pub function_call: Option<FunctionCallData>,
    pub thought: Option<bool>,
}
