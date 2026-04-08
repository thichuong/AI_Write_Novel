import { state } from './state.js';
import { invoke, listen } from './services/tauri.js';
import { showStatus } from './utils.js';

export function setupAIListeners() {
    listen('ai-chat-stream-thought', (event) => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            let thoughtBox = aiMsgDiv.querySelector('.thinking-box');
            if (!thoughtBox) {
                thoughtBox = document.createElement('div');
                thoughtBox.className = 'thinking-box';
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

    listen('ai-chat-stream-tool', (event) => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            const { name, args } = event.payload;
            let toolBox = aiMsgDiv.querySelector('.tool-status-box');
            if (!toolBox) {
                toolBox = document.createElement('div');
                toolBox.className = 'tool-status-box';
                aiMsgDiv.appendChild(toolBox);
            }
            
            let statusText = "";
            if (name === "read_file") statusText = `🔍 Đang đọc: ${args.path}`;
            else if (name === "write_file") statusText = `📝 Đang ghi: ${args.path}`;
            else if (name === "list_directory") statusText = `📂 Đang xem thư mục: ${args.path}`;
            else if (name === "delete_file") statusText = `🗑️ Đang xóa: ${args.path}`;
            else statusText = `⚙️ Đang dùng: ${name}`;

            toolBox.innerText = `[${statusText}]`;
            const chatMessages = document.getElementById('chat-messages');
            if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
        }
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
            currentFile: state.activeFilePath || "",
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
