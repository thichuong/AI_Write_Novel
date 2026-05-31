import { state } from './state.js';
import { invoke, listen } from './services/tauri.js';
import { showStatus } from './utils.js';

export function setupAIListeners() {
    listen('ai-agent-step', (event) => {
        const phase = event.payload;
        getOrCreatePhaseContainer(phase, true);
    });

    listen('ai-chat-stream-thought', (event) => {
        const { text, phase } = event.payload;
        // Thoughts luôn vào thoughts-section, kể cả phase complete
        const container = getOrCreatePhaseContainer(phase);
        if (!container) return;
        
        const thoughtsSection = document.querySelector('.message.assistant.streaming .thoughts-section');
        if (thoughtsSection) thoughtsSection.classList.remove('collapsed');

        const contentArea = container.querySelector('.phase-content');
        if (!contentArea) return;
        
        let thoughtBox = contentArea.querySelector('.thinking-box');
        if (!thoughtBox) {
            thoughtBox = document.createElement('div');
            thoughtBox.className = 'thinking-box';
            contentArea.appendChild(thoughtBox);
        }
        thoughtBox.innerText += text;
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
    });

    listen('ai-chat-stream', (event) => {
        const { text, phase } = event.payload;
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (!aiMsgDiv) return;

        if (phase !== 'complete') {
            const container = getOrCreatePhaseContainer(phase);
            if (!container) return;

            const thoughtsSection = document.querySelector('.message.assistant.streaming .thoughts-section');
            if (thoughtsSection) thoughtsSection.classList.remove('collapsed');

            const contentArea = container.querySelector('.phase-content');
            if (!contentArea) return;

            let thoughtBox = contentArea.querySelector('.thinking-box');
            if (!thoughtBox) {
                thoughtBox = document.createElement('div');
                thoughtBox.className = 'thinking-box';
                contentArea.appendChild(thoughtBox);
            }
            thoughtBox.innerText += text;
        } else {
            // Answer Phase (Final Response)
            let answerSection = aiMsgDiv.querySelector('.answer-section');
            if (!answerSection) {
                answerSection = document.createElement('div');
                answerSection.className = 'answer-section';
                const content = document.createElement('div');
                content.className = 'answer-content';
                answerSection.appendChild(content);
                aiMsgDiv.appendChild(answerSection);
            }
            const contentArea = answerSection.querySelector('.answer-content');
            contentArea.innerText += text;
        }
        
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;
    });

    listen('ai-chat-stream-done', async (event) => {
        const { phase } = event.payload;
        console.log(`Phase ${phase} done.`);
        
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages) chatMessages.scrollTop = chatMessages.scrollHeight;

        // Tự động thu gọn phần suy nghĩ khi phase hoàn tất (nếu không phải complete)
        const container = document.querySelector(`.phase-container[data-phase="${phase}"]`);
        if (container && phase !== 'complete') {
            container.classList.add('collapsed');
        }

        // Khi các phase quan trọng hoàn tất, thu gọn thoughts-section để tập trung vào câu trả lời
        if (phase === 'finalize' || phase === 'complete') {
            const aiMsgDiv = document.querySelector('.message.assistant.streaming');
            if (aiMsgDiv) {
                const thoughtsSection = aiMsgDiv.querySelector('.thoughts-section');
                if (thoughtsSection) {
                    thoughtsSection.classList.add('collapsed');
                    const status = thoughtsSection.querySelector('.thoughts-status');
                    if (status) status.innerText = "Đã xong";
                }
            }
        }

        // Chỉ xử lý kết thúc toàn bộ nếu phase là 'complete'
        if (phase === 'complete') {
            const aiMsgDiv = document.querySelector('.message.assistant.streaming');
            if (aiMsgDiv) {
                aiMsgDiv.classList.remove('streaming');
                const answerContent = aiMsgDiv.querySelector('.answer-content');
                let finalText = answerContent ? answerContent.innerText : "";
                
                state.chatHistory.push({ role: "assistant", content: finalText.trim() });
                await saveChatHistory();
            }
        }
    });

    listen('ai-chat-stream-tool', (event) => {
        const { name, args, phase } = event.payload;
        const container = getOrCreatePhaseContainer(phase);
        if (!container) return;
        
        const thoughtsSection = document.querySelector('.message.assistant.streaming .thoughts-section');
        if (thoughtsSection) thoughtsSection.classList.remove('collapsed');

        const contentArea = container.querySelector('.phase-content');
        if (!contentArea) return;

        let toolBox = contentArea.querySelector('.tool-status-box');
        if (!toolBox) {
            toolBox = document.createElement('div');
            toolBox.className = 'tool-status-box';
            contentArea.appendChild(toolBox);
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

    listen('ai-agent-selected', (event) => {
        const agent = event.payload;
        state.currentAgentType = agent; // Sync agent type with frontend state
        const agentMap = {
            'chat': 'Chat Agent',
            'ideation': 'Ide Agent',
            'writing': 'Writing Agent'
        };
        showStatus(`Đã điều phối tới: ${agentMap[agent] || agent}`);
    });

    // Khởi tạo UI API Key
    initAPIKeyUI();

    // Khởi tạo sự kiện nút gửi
    const sendBtn = document.getElementById('send-chat-btn');
    if (sendBtn) {
        sendBtn.onclick = () => {
            if (state.isAgentRunning) {
                stopChat();
            } else {
                sendChat();
            }
        };
    }

    // Cài đặt sự kiện xóa tất cả kiến thức đã chọn
    const clearKnowledgeBtn = document.getElementById('clear-knowledge-btn');
    if (clearKnowledgeBtn) {
        clearKnowledgeBtn.onclick = () => {
            clearKnowledgeFiles();
        };
    }

    // Khởi tạo hiển thị ban đầu cho các file kiến thức
    renderKnowledgeFiles();

    // Khởi tạo và ghi nhớ Agent Type từ localStorage
    const agentTypeSelect = document.getElementById('agent-type-select');
    if (agentTypeSelect) {
        const savedAgentType = localStorage.getItem('ai_agent_type') || 'chat';
        agentTypeSelect.value = savedAgentType;
        
        agentTypeSelect.addEventListener('sl-change', () => {
            localStorage.setItem('ai_agent_type', agentTypeSelect.value);
        });
    }
}

async function initAPIKeyUI() {
    const hasKey = await invoke('check_api_key');
    const apiKeyContainer = document.getElementById('api-key-container');
    const chatInputContainer = document.querySelector('.chat-input-container');

    if (!hasKey) {
        if (apiKeyContainer) apiKeyContainer.classList.remove('hidden');
        if (chatInputContainer) chatInputContainer.classList.add('hidden');
        if (window.lucide) window.lucide.createIcons();
    }

    const saveBtn = document.getElementById('save-api-key-btn');
    const keyInput = document.getElementById('api-key-input');

    if (saveBtn && keyInput) {
        saveBtn.onclick = async () => {
            const key = keyInput.value.trim();
            if (!key) {
                showStatus("Vui lòng nhập API Key!");
                return;
            }

            try {
                saveBtn.disabled = true;
                saveBtn.innerText = "Đang lưu...";
                await invoke('save_api_key', { apiKey: key });
                
                showStatus("Đã lưu API Key thành công!");
                if (apiKeyContainer) apiKeyContainer.classList.add('hidden');
                if (chatInputContainer) chatInputContainer.classList.remove('hidden');
            } catch (err) {
                console.error("Failed to save API key:", err);
                showStatus("Lỗi khi lưu key: " + err);
            } finally {
                saveBtn.disabled = false;
                saveBtn.innerText = "Lưu Key";
            }
        };

        // Hỗ trợ nhấn Enter để lưu
        keyInput.onkeydown = (e) => {
            if (e.key === 'Enter') saveBtn.click();
        };
    }
}

function getOrCreateThoughtsSection(aiMsgDiv) {
    let thoughtsSection = aiMsgDiv.querySelector('.thoughts-section');
    if (!thoughtsSection) {
        thoughtsSection = document.createElement('div');
        thoughtsSection.className = 'thoughts-section';
        
        const header = document.createElement('div');
        header.className = 'thoughts-header';
        
        const title = document.createElement('div');
        title.className = 'thoughts-title';
        title.innerHTML = '<i data-lucide="brain-circuit"></i> <span>QUÁ TRÌNH SUY NGHĨ</span>';
        
        const status = document.createElement('div');
        status.className = 'thoughts-status';
        status.innerText = "Đang xử lý...";
        
        header.appendChild(title);
        header.appendChild(status);
        
        const content = document.createElement('div');
        content.className = 'thoughts-content';
        
        thoughtsSection.appendChild(header);
        thoughtsSection.appendChild(content);
        
        // Chèn vào đầu tin nhắn assistant
        aiMsgDiv.prepend(thoughtsSection);
        
        header.onclick = () => {
            thoughtsSection.classList.toggle('collapsed');
        };
        
        if (window.lucide) window.lucide.createIcons();
    }
    return thoughtsSection;
}

function getOrCreatePhaseContainer(phase) {
    const aiMsgDiv = document.querySelector('.message.assistant.streaming');
    if (!aiMsgDiv) return null;

    const thoughtsSection = getOrCreateThoughtsSection(aiMsgDiv);
    thoughtsSection.classList.remove('collapsed');
    const thoughtsContent = thoughtsSection.querySelector('.thoughts-content');

    let container = thoughtsContent.querySelector(`.phase-container[data-phase="${phase}"]`);
    if (!container) {
        container = document.createElement('div');
        container.className = 'phase-container thought-node';
        container.setAttribute('data-phase', phase);
        
        const header = document.createElement('div');
        header.className = 'phase-header';
        
        const badge = document.createElement('div');
        badge.className = 'agent-step-badge';
        
        // Dynamically choose display labels based on current active agent type
        const agentType = state.currentAgentType || 'chat';
        let phaseMap = {};
        
        if (agentType === 'chat') {
            phaseMap = {
                'coordinating': '🧠 Điều phối Agent',
                'chatting': '💬 Đang kết nối',
                'complete': '💬 Trò chuyện & Tra cứu'
            };
        } else if (agentType === 'ideation' || agentType === 'ide') {
            phaseMap = {
                'coordinating': '🧠 Điều phối Agent',
                'thinking': '💡 Suy nghĩ & Lên ý tưởng',
                'complete': '💡 Brainstorm & Thiết lập Wiki'
            };
        } else if (agentType === 'writing' || agentType === 'writting') {
            phaseMap = {
                'coordinating': '🧠 Điều phối Agent',
                'thinking': '✍️ Sáng tác chương mới',
                'finalize': '⚙️ Đồng bộ Wiki & Memory',
                'complete': '✨ Hoàn tất sáng tác'
            };
        } else {
            phaseMap = {
                'coordinating': '🧠 Điều phối Agent',
                'thinking': '✍️ Sáng tác chương mới',
                'finalize': '⚙️ Đồng bộ Wiki & Memory',
                'complete': '✨ Hoàn tất phản hồi'
            };
        }
        
        badge.innerText = phaseMap[phase] || phase;
        header.appendChild(badge);

        const toggleIcon = document.createElement('i');
        toggleIcon.className = 'toggle-icon';
        toggleIcon.setAttribute('data-lucide', 'chevron-down');
        header.appendChild(toggleIcon);

        header.onclick = (e) => {
            e.stopPropagation();
            container.classList.toggle('collapsed');
        };

        const contentArea = document.createElement('div');
        contentArea.className = 'phase-content';
        
        container.appendChild(header);
        container.appendChild(contentArea);
        thoughtsContent.appendChild(container);

        if (window.lucide) window.lucide.createIcons();
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

    // Kiểm tra API Key lần nữa trước khi gửi (đề phòng)
    const hasKey = await invoke('check_api_key');
    if (!hasKey) {
        const apiKeyContainer = document.getElementById('api-key-container');
        const chatInputContainer = document.querySelector('.chat-input-container');
        if (apiKeyContainer) apiKeyContainer.classList.remove('hidden');
        if (chatInputContainer) chatInputContainer.classList.add('hidden');
        showStatus("Vui lòng cấu hình API Key!");
        return;
    }

    addChatMessage("user", msg);
    state.chatHistory.push({ role: "user", content: msg });
    chatInput.value = "";

    const aiMsgDiv = addChatMessage("assistant", "");
    aiMsgDiv.classList.add('streaming');

    setAgentRunning(true);
    showStatus("AI đang suy nghĩ...");

    try {
        const knowledgeFiles = state.selectedKnowledgeFiles || [];
        const agentTypeSelect = document.getElementById('agent-type-select');
        const selectedAgentType = agentTypeSelect ? agentTypeSelect.value : 'chat';
        state.currentAgentType = selectedAgentType; // Set active agent type before invoking chat

        await invoke('ai_chat', {
            rootPath: state.currentStoryPath,
            currentFile: state.activeFilePath || "",
            message: msg,
            chatHistory: state.chatHistory.slice(-10),
            selectedKnowledgeFiles: knowledgeFiles.map(f => f.path),
            agentType: selectedAgentType,
        });
    } catch (err) {
        console.error("AI chat failed:", err);
        aiMsgDiv.classList.remove('streaming');
        
        const errorBox = document.createElement('div');
        errorBox.className = 'error-box';
        errorBox.innerHTML = `<i data-lucide="alert-octagon"></i> <span><strong>${err === 'Agent stopped by user' ? 'Đã dừng Agent' : 'Lỗi hệ thống:'}</strong> ${err}</span>`;
        if (err === 'Agent stopped by user') {
            errorBox.classList.add('stopped-info');
        }
        aiMsgDiv.appendChild(errorBox);
        
        if (window.lucide) window.lucide.createIcons();
        showStatus(err === 'Agent stopped by user' ? "Đã dừng Agent" : "Lỗi AI Assistant", true);
    } finally {
        setAgentRunning(false);
    }
}

export async function stopChat() {
    try {
        showStatus("Đang dừng Agent...");
        await invoke('stop_ai_chat');
    } catch (err) {
        console.error("Failed to stop chat:", err);
        showStatus("Lỗi khi dừng Agent", true);
    }
}

function setAgentRunning(isRunning) {
    state.isAgentRunning = isRunning;
    const sendBtn = document.getElementById('send-chat-btn');
    if (sendBtn) {
        if (isRunning) {
            sendBtn.innerHTML = '<i data-lucide="square"></i>';
            sendBtn.title = "Dừng Agent";
            sendBtn.classList.add('running');
        } else {
            sendBtn.innerHTML = '<i data-lucide="send"></i>';
            sendBtn.title = "Gửi tin nhắn";
            sendBtn.classList.remove('running');
        }
        if (window.lucide) window.lucide.createIcons();
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
    
    if (role === 'user') {
        div.innerText = text;
    } else if (text) {
        // Nếu có text sẵn (ví dụ load từ history), tạo luôn answer-section
        const answerSection = document.createElement('div');
        answerSection.className = 'answer-section';
        const content = document.createElement('div');
        content.className = 'answer-content';
        content.innerText = text;
        answerSection.appendChild(content);
        div.appendChild(answerSection);
    }
    
    chatMessages.appendChild(div);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    return div;
}

export function addFileToKnowledge(filePath, fileName) {
    if (!state.selectedKnowledgeFiles) {
        state.selectedKnowledgeFiles = [];
    }
    
    // Kiểm tra trùng lặp
    const exists = state.selectedKnowledgeFiles.some(f => f.path === filePath);
    if (exists) {
        showStatus("Tệp này đã được nạp làm kiến thức!");
        return;
    }
    
    state.selectedKnowledgeFiles.push({ path: filePath, name: fileName });
    renderKnowledgeFiles();
    showStatus(`Đã nạp kiến thức: ${fileName}`);
}

export function removeFileFromKnowledge(filePath) {
    if (!state.selectedKnowledgeFiles) return;
    state.selectedKnowledgeFiles = state.selectedKnowledgeFiles.filter(f => f.path !== filePath);
    renderKnowledgeFiles();
}

export function clearKnowledgeFiles() {
    state.selectedKnowledgeFiles = [];
    renderKnowledgeFiles();
    showStatus("Đã xóa tất cả file kiến thức!");
}

export function renderKnowledgeFiles() {
    const container = document.getElementById('knowledge-files-container');
    const countSpan = document.getElementById('knowledge-files-count');
    const listDiv = document.getElementById('knowledge-files-list');
    
    if (!container || !listDiv) return;
    
    const files = state.selectedKnowledgeFiles || [];
    
    if (files.length === 0) {
        container.classList.add('hidden');
        return;
    }
    
    container.classList.remove('hidden');
    if (countSpan) countSpan.innerText = files.length;
    
    listDiv.innerHTML = "";
    files.forEach(file => {
        const tag = document.createElement('div');
        tag.className = 'knowledge-tag';
        
        const nameSpan = document.createElement('span');
        nameSpan.innerText = file.name;
        nameSpan.title = file.path;
        tag.appendChild(nameSpan);
        
        const removeBtn = document.createElement('button');
        removeBtn.className = 'knowledge-tag-remove';
        removeBtn.title = "Xóa";
        removeBtn.innerHTML = '<i data-lucide="x"></i>';
        removeBtn.onclick = (e) => {
            e.stopPropagation();
            removeFileFromKnowledge(file.path);
        };
        tag.appendChild(removeBtn);
        
        listDiv.appendChild(tag);
    });
    
    if (window.lucide) window.lucide.createIcons();
}
