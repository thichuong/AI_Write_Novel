import { state } from './state.js';
import { invoke, listen } from './services/tauri.js';
import { showStatus } from './utils.js';

export function setupAIListeners() {
    listen('ai-agent-step', (event) => {
        const phase = event.payload;
        getOrCreatePhaseContainer(phase);
    });

    listen('ai-chat-stream-thought', (event) => {
        const { text, phase } = event.payload;
        const container = getOrCreatePhaseContainer(phase);
        let thoughtBox = container.querySelector('.thinking-box');
        if (!thoughtBox) {
            thoughtBox = document.createElement('div');
            thoughtBox.className = 'thinking-box';
            thoughtBox.onclick = () => {
                if (thoughtBox.classList.contains('collapsed')) {
                    thoughtBox.classList.remove('collapsed');
                } else {
                    thoughtBox.classList.add('collapsed');
                }
            };
            container.appendChild(thoughtBox);
        }
        thoughtBox.innerText += text;
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
    });

    listen('ai-chat-stream', (event) => {
        const { text, phase } = event.payload;
        const container = getOrCreatePhaseContainer(phase);
        let answerBox = container.querySelector('.answer-box');
        if (!answerBox) {
            answerBox = document.createElement('div');
            answerBox.className = 'answer-box';
            container.appendChild(answerBox);
        }
        answerBox.innerText += text;
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
    });

    listen('ai-chat-stream-done', async (event) => {
        const { phase } = event.payload;
        console.log(`Phase ${phase} done.`);
        
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;

        // Tự động thu gọn phần suy nghĩ khi phase hoàn tất
        const container = document.querySelector(`.phase-container[data-phase="${phase}"]`);
        if (container) {
            const thoughtBox = container.querySelector('.thinking-box');
            if (thoughtBox) {
                thoughtBox.classList.add('collapsed');
            }
        }

        // Chỉ xử lý kết thúc toàn bộ nếu phase là 'complete'
        if (phase === 'complete') {
            const aiMsgDiv = document.querySelector('.message.assistant.streaming');
            if (aiMsgDiv) {
                aiMsgDiv.classList.remove('streaming');
                const allAnswerBoxes = aiMsgDiv.querySelectorAll('.answer-box');
                let finalText = "";
                allAnswerBoxes.forEach(box => finalText += box.innerText + "\n");
                
                state.chatHistory.push({ role: "assistant", content: finalText.trim() });
                await saveChatHistory();
            }
            showStatus("Ready");
        }
    });

    listen('ai-chat-stream-tool', (event) => {
        const { name, args, phase } = event.payload;
        const container = getOrCreatePhaseContainer(phase);
        
        let toolBox = container.querySelector('.tool-status-box');
        if (!toolBox) {
            toolBox = document.createElement('div');
            toolBox.className = 'tool-status-box';
            container.appendChild(toolBox);
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
    });

    listen('ai-chat-stream-tool-done', () => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            const toolBoxes = aiMsgDiv.querySelectorAll('.tool-status-box');
            if (toolBoxes.length > 0) {
                const lastToolBox = toolBoxes[toolBoxes.length - 1];
                lastToolBox.innerText = lastToolBox.innerText.replace('🔍', '✅').replace('📝', '✅').replace('📂', '✅').replace('🗑️', '✅');
            }
        }
    });
}

function getOrCreatePhaseContainer(phase) {
    const aiMsgDiv = document.querySelector('.message.assistant.streaming');
    if (!aiMsgDiv) return null;

    let container = aiMsgDiv.querySelector(`.phase-container[data-phase="${phase}"]`);
    if (!container) {
        container = document.createElement('div');
        container.className = 'phase-container';
        container.setAttribute('data-phase', phase);
        
        const badge = document.createElement('div');
        badge.className = 'agent-step-badge';
        const phaseMap = {
            'analyze': '🔍 Phân tích ngữ cảnh',
            'execute': '⚙️ Đang thực thi',
            'summarize': '📝 Tổng hợp diễn biến',
            'memory': '💾 Cập nhật Memory',
            'complete': '✅ Hoàn tất & Phản hồi'
        };
        badge.innerText = phaseMap[phase] || phase;
        
        container.appendChild(badge);
        aiMsgDiv.appendChild(container);
    }
    return container;
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
