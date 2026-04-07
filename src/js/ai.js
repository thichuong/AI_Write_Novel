import { state } from './state.js';
import { invoke, listen } from './services/tauri.js';
import { showStatus } from './utils.js';
import { saveActiveFile } from './editor.js';

export function setupAIListeners() {
    listen('ai-chat-stream-thought', (event) => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            let thoughtBox = aiMsgDiv.querySelector('.thinking-box');
            if (!thoughtBox) {
                // Remove initial text node if any, though it shouldn't be there
                thoughtBox = document.createElement('div');
                thoughtBox.className = 'thinking-box';
                // Insert thought box at the top
                aiMsgDiv.insertBefore(thoughtBox, aiMsgDiv.firstChild);
            }
            thoughtBox.innerText += event.payload;
            const chatMessages = document.getElementById('chat-messages');
            if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
        }
    });

    listen('ai-chat-stream', (event) => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            let answerBox = aiMsgDiv.querySelector('.answer-box');
            if (!answerBox) {
                answerBox = document.createElement('div');
                answerBox.className = 'answer-box';
                aiMsgDiv.appendChild(answerBox);
            }
            answerBox.innerText += event.payload;
            const chatMessages = document.getElementById('chat-messages');
            if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
        }
    });

    listen('ai-chat-stream-done', async () => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            aiMsgDiv.classList.remove('streaming');
            let thoughtBox = aiMsgDiv.querySelector('.thinking-box');
            if (thoughtBox) {
                thoughtBox.style.display = 'none';
            }
            let answerBox = aiMsgDiv.querySelector('.answer-box');
            const finalText = answerBox ? answerBox.innerText : aiMsgDiv.innerText;
            state.chatHistory.push({ role: "assistant", content: finalText });
            await saveChatHistory();
        }
        showStatus("Ready");
    });

    listen('ai-write-stream', (event) => {
        const storyEditor = document.getElementById('story-editor');
        if (storyEditor) {
            storyEditor.innerText += event.payload;
            storyEditor.scrollTop = storyEditor.scrollHeight;
        }
    });

    listen('ai-write-stream-done', () => {
        showStatus("AI hoàn tất.");
        saveActiveFile(true);
    });
}

export async function sendChat() {
    const chatInput = document.getElementById('chat-input');
    if (!state.currentStoryPath) {
        showStatus("Vui lòng mở một thư mục truyện trước!");
        return;
    }
    
    const msg = chatInput.value.trim();
    if (!msg) return;

    addChatMessage("user", msg);
    state.chatHistory.push({ role: "user", content: msg });
    chatInput.value = "";

    const aiMsgDiv = addChatMessage("assistant", "");
    aiMsgDiv.classList.add('streaming');

    showStatus("AI đang suy nghĩ...");

    try {
        await invoke('ai_chat', {
            rootPath: state.currentStoryPath,
            message: msg,
            chatHistory: state.chatHistory.slice(-10),
        });
    } catch (err) {
        console.error("AI chat failed:", err);
        aiMsgDiv.innerText = "Lỗi: " + err;
        aiMsgDiv.classList.remove('streaming');
        showStatus("Lỗi AI");
    }
}

export async function runAiWriting(type) {
    if (!state.currentStoryPath) {
        showStatus("Vui lòng mở một thư mục!");
        return;
    }
    
    if (!state.activeFilePath) {
        showStatus("Vui lòng mở một file!");
        return;
    }

    let instruction = "";
    if (type === "rewrite") instruction = "Hãy sửa lại đoạn văn này cho tinh tế và giàu cảm xúc hơn.";
    else if (type === "continue") instruction = "Hãy viết tiếp đoạn văn này một cách tự nhiên.";
    else if (type === "full") instruction = "Dựa trên các quy tắc và nhân vật, hãy phát triển tiếp nội dung cho chương này.";

    const selection = window.getSelection().toString();
    const storyEditor = document.getElementById('story-editor');
    const content = storyEditor ? storyEditor.innerText : "";

    showStatus("AI đang viết...");
    if (!selection && storyEditor) storyEditor.innerText += "\n\n";

    try {
        await invoke('ai_write', {
            rootPath: state.currentStoryPath,
            currentFile: state.activeFilePath,
            instruction,
            currentContent: content,
            selection: selection || null,
        });
    } catch (err) {
        console.error("AI write failed:", err);
        alert("Lỗi AI: " + err);
        showStatus("Lỗi AI");
    }
}

export async function clearChat() {
    const chatMessages = document.getElementById('chat-messages');
    if (chatMessages) chatMessages.innerHTML = "";
    state.chatHistory = [];
    if (state.currentStoryPath) {
        await saveChatHistory();
    }
}

export async function loadChatHistory() {
    if (!state.currentStoryPath) return;
    try {
        state.chatHistory = await invoke('get_chat_history', {
            rootPath: state.currentStoryPath,
        });
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.innerHTML = "";
        state.chatHistory.forEach(msg => {
            addChatMessage(msg.role, msg.content);
        });
    } catch (err) {
        console.error("Failed to load chat history:", err);
        state.chatHistory = [];
    }
}

export async function saveChatHistory() {
    if (!state.currentStoryPath) return;
    try {
        await invoke('save_chat_history', {
            rootPath: state.currentStoryPath,
            history: state.chatHistory,
        });
    } catch (err) {
        console.error("Failed to save chat history:", err);
    }
}

export function addChatMessage(role, text) {
    const chatMessages = document.getElementById('chat-messages');
    if (!chatMessages) return null;
    
    const div = document.createElement('div');
    div.className = `message ${role === 'assistant' ? 'assistant' : 'user'}`;
    div.innerText = text;
    chatMessages.appendChild(div);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    return div;
}

// Map globals
window.runAiWriting = runAiWriting;
