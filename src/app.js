const invoke = window.__TAURI__?.core?.invoke || (async (cmd, args) => {
    console.warn(`Tauri invoke('${cmd}') failed: window.__TAURI__ is undefined.`, args);
    return Promise.reject("Tauri API not available");
});

const listen = window.__TAURI__?.event?.listen || (async (event, handler) => {
    console.warn(`Tauri listen('${event}') failed: window.__TAURI__ is undefined.`);
    return () => {}; // return dummy unlisten function
});

// Tauri Dialog API (v2)
const openDialog = async (options) => {
    if (window.__TAURI__?.dialog?.open) {
        return await window.__TAURI__.dialog.open(options);
    }
    if (window.__TAURI__?.core?.invoke) {
        return await window.__TAURI__.core.invoke('plugin:dialog|open', options);
    }
    return Promise.reject("Tauri Dialog API not available");
};

let state = {
    currentStoryPath: null,
    nodes: [],           // Tree of FileNode from Rust
    openTabs: [],        // Array of { name, path, content }
    activeFilePath: null, // relative path of active file
    chatHistory: [],
    isSaving: false,
};

// Selectors
const explorerTree = document.getElementById('explorer-tree');
const tabsList = document.getElementById('tabs-list');
const storyEditor = document.getElementById('story-editor');
const currentFileName = document.getElementById('current-file-name');
const chatMessages = document.getElementById('chat-messages');
const chatInput = document.getElementById('chat-input');
const statusStoryName = document.getElementById('status-story-name');
const modalOverlay = document.getElementById('modal-overlay');

// ═══════════════════════════════════════════════
// 1. Initialization
// ═══════════════════════════════════════════════
document.addEventListener('DOMContentLoaded', async () => {
    setupEventListeners();
    setupAIListeners();
    
    // Load last opened folder
    const lastPath = localStorage.getItem('last_story_path');
    if (lastPath) {
        await openStory(lastPath);
    }
    
    lucide.createIcons();
});

function setupEventListeners() {
    // Activity Bar
    const navItems = ['explorer', 'search', 'story-settings', 'profile'];
    navItems.forEach(id => {
        const el = document.getElementById(`nav-${id}`);
        if (el) el.onclick = () => switchActivity(id);
    });

    // Sidebar: Folder Actions
    const newStoryBtn = document.getElementById('new-story-btn');
    if (newStoryBtn) newStoryBtn.onclick = handleCreateStory;

    const openFolderBtn = document.getElementById('open-folder-btn');
    if (openFolderBtn) openFolderBtn.onclick = handleOpenFolder;

    const refreshBtn = document.getElementById('refresh-explorer');
    if (refreshBtn) refreshBtn.onclick = () => loadNodes(state.currentStoryPath);

    // Editor Actions
    document.getElementById('save-btn').onclick = () => saveActiveFile();
    document.getElementById('ai-full-write-btn').onclick = () => runAiWriting("full");

    // Chat Actions
    document.getElementById('send-chat-btn').onclick = sendChat;
    document.getElementById('clear-chat-btn').onclick = clearChat;
    chatInput.onkeydown = (e) => {
        if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendChat(); }
    };

    // Modal
    document.getElementById('modal-cancel').onclick = () => modalOverlay.classList.add('hidden');

    // Floating Toolbar
    document.getElementById('ai-rewrite-btn').onclick = () => runAiWriting("rewrite");
    document.getElementById('ai-continue-btn').onclick = () => runAiWriting("continue");

    // Click outside toolbar
    document.addEventListener('mousedown', (e) => {
        const toolbar = document.getElementById('floating-toolbar');
        if (!toolbar.contains(e.target) && !storyEditor.contains(e.target)) {
            toolbar.classList.add('hidden');
        }
    });

    // Editor Auto-save on input (debounced)
    storyEditor.addEventListener('input', debounce(() => {
        if (state.activeFilePath) saveActiveFile(true);
    }, 2000));

    // Keyboard shortcut: Ctrl+S to save
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey && e.key === 's') {
            e.preventDefault();
            saveActiveFile();
        }
    });
}

// Setup AI streaming event listeners
function setupAIListeners() {
    listen('ai-chat-stream', (event) => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            aiMsgDiv.innerText += event.payload;
            chatMessages.scrollTop = chatMessages.scrollHeight;
        }
    });

    listen('ai-chat-stream-done', async () => {
        const aiMsgDiv = document.querySelector('.message.assistant.streaming');
        if (aiMsgDiv) {
            aiMsgDiv.classList.remove('streaming');
            state.chatHistory.push({ role: "assistant", content: aiMsgDiv.innerText });
            await saveChatHistory();
        }
        showStatus("Ready");
    });

    listen('ai-write-stream', (event) => {
        storyEditor.innerText += event.payload;
        storyEditor.scrollTop = storyEditor.scrollHeight;
    });

    listen('ai-write-stream-done', () => {
        showStatus("AI hoàn tất.");
        saveActiveFile(true);
    });
}

function switchActivity(activity) {
    document.querySelectorAll('.activity-icon').forEach(el => el.classList.remove('active'));
    const target = document.getElementById(`nav-${activity}`);
    if (target) target.classList.add('active');
}

// ═══════════════════════════════════════════════
// 2. Folder Operations
// ═══════════════════════════════════════════════
async function handleOpenFolder() {
    try {
        const selected = await openDialog({
            directory: true,
            multiple: false,
            title: "Chọn thư mục truyện"
        });
        
        if (selected) {
            await openStory(selected);
        }
    } catch (err) {
        console.error("Failed to open dialog:", err);
        alert("Lỗi: " + err);
    }
}

async function handleCreateStory() {
    try {
        const selected = await openDialog({
            directory: true,
            multiple: false,
            title: "Chọn thư mục cha để tạo truyện mới"
        });
        
        if (selected) {
            showModal("Tên truyện mới", "create-story", { parentDir: selected });
        }
    } catch (err) {
        console.error("Failed to open dialog:", err);
        alert("Lỗi: " + err);
    }
}

async function openStory(path) {
    if (state.currentStoryPath && state.currentStoryPath !== path) {
        state.openTabs = [];
        state.activeFilePath = null;
        renderTabs();
        storyEditor.innerText = "";
        currentFileName.innerText = "Chọn một file để bắt đầu";
    }

    state.currentStoryPath = path;
    const folderName = path.split(/[/\\]/).pop() || path;
    statusStoryName.innerText = folderName;
    document.getElementById('story-title-display').innerText = folderName.toUpperCase();

    localStorage.setItem('last_story_path', path);

    await loadNodes(path);
    await loadChatHistory();
}

// ═══════════════════════════════════════════════
// 3. File Explorer
// ═══════════════════════════════════════════════
async function loadNodes(rootPath) {
    if (!rootPath) return;
    try {
        state.nodes = await invoke('list_nodes', { rootPath });
        renderExplorer();
    } catch (err) {
        console.error("Failed to load nodes:", err);
        alert("Không thể tải danh sách file: " + err);
    }
}

function renderExplorer() {
    explorerTree.innerHTML = "";
    state.nodes.forEach(node => {
        explorerTree.appendChild(createNodeElement(node));
    });
    lucide.createIcons();
}

function createNodeElement(node) {
    const container = document.createElement('div');
    container.className = 'explorer-node';

    const item = document.createElement('div');
    item.className = `node-item ${node.path === state.activeFilePath ? 'active' : ''} ${node.nodeType === 'folder' ? 'node-folder' : 'node-file'}`;

    const icon = node.nodeType === 'folder' ? 'folder' : 'file-text';
    item.innerHTML = `
        <i data-lucide="${icon}" class="node-icon"></i>
        <span class="node-name">${node.name}</span>
        <div class="node-actions">
            ${node.nodeType === 'folder' ? `
                <button onclick="event.stopPropagation(); showNodeModal('${escapeAttr(node.path)}', 'file')" title="Add File"><i data-lucide="file-plus"></i></button>
                <button onclick="event.stopPropagation(); showNodeModal('${escapeAttr(node.path)}', 'folder')" title="Add Folder"><i data-lucide="folder-plus"></i></button>
            ` : ''}
            <button onclick="event.stopPropagation(); renameNodePrompt('${escapeAttr(node.path)}', '${escapeAttr(node.name)}')" title="Rename"><i data-lucide="pencil"></i></button>
            <button onclick="event.stopPropagation(); deleteNode('${escapeAttr(node.path)}')" title="Delete"><i data-lucide="trash-2"></i></button>
        </div>
    `;

    item.onclick = async () => {
        if (node.nodeType === 'folder') {
            let contents = container.querySelector('.folder-contents');
            if (contents) {
                contents.classList.toggle('hidden');
            } else {
                try {
                    const children = await invoke('list_nodes', { 
                        rootPath: state.currentStoryPath, 
                        parentPath: node.path 
                    });
                    node.children = children;
                    contents = document.createElement('div');
                    contents.className = 'folder-contents';
                    children.forEach(child => {
                        contents.appendChild(createNodeElement(child));
                    });
                    container.appendChild(contents);
                } catch (err) {
                    console.error("Failed to load folder contents:", err);
                }
            }
        } else {
            openFile(node);
        }
    };

    container.appendChild(item);

    // Initial render if children exist (e.g. they were loaded before and we are re-rendering)
    if (node.nodeType === 'folder' && node.children && node.children.length > 0) {
        const contents = document.createElement('div');
        contents.className = 'folder-contents';
        node.children.forEach(child => {
            contents.appendChild(createNodeElement(child));
        });
        container.appendChild(contents);
    }

    return container;
}

// ═══════════════════════════════════════════════
// 4. Tabs & Editor
// ═══════════════════════════════════════════════
async function openFile(node) {
    try {
        const content = await invoke('read_file', {
            rootPath: state.currentStoryPath,
            filePath: node.path,
        });

        const tab = { name: node.name, path: node.path, content };

        if (!state.openTabs.find(t => t.path === node.path)) {
            state.openTabs.push(tab);
        } else {
            const existing = state.openTabs.find(t => t.path === node.path);
            if (existing) existing.content = content;
        }

        state.activeFilePath = node.path;
        renderTabs();
        loadEditorContent(tab);
        renderExplorer();
    } catch (err) {
        console.error("Failed to open file:", err);
        alert("Không thể mở file: " + err);
    }
}

function renderTabs() {
    tabsList.innerHTML = "";
    state.openTabs.forEach(tab => {
        const tabEl = document.createElement('div');
        tabEl.className = `tab ${tab.path === state.activeFilePath ? 'active' : ''}`;
        tabEl.innerHTML = `
            <span>${tab.name}</span>
            <i data-lucide="x" class="tab-close" onclick="event.stopPropagation(); closeTab('${escapeAttr(tab.path)}')"></i>
        `;
        tabEl.onclick = () => {
            state.activeFilePath = tab.path;
            renderTabs();
            loadEditorContent(tab);
            renderExplorer();
        };
        tabsList.appendChild(tabEl);
    });
    lucide.createIcons();
}

function closeTab(filePath) {
    state.openTabs = state.openTabs.filter(t => t.path !== filePath);
    if (state.activeFilePath === filePath) {
        state.activeFilePath = state.openTabs.length > 0 ? state.openTabs[state.openTabs.length - 1].path : null;
    }
    renderTabs();
    if (state.activeFilePath) {
        const tab = state.openTabs.find(t => t.path === state.activeFilePath);
        if (tab) loadEditorContent(tab);
    } else {
        storyEditor.innerText = "";
        currentFileName.innerText = "Chào mừng";
    }
}

function loadEditorContent(tab) {
    currentFileName.innerText = tab.name;
    storyEditor.innerText = tab.content || "";
    document.getElementById('word-count').innerText = `Chars: ${tab.content ? tab.content.length : 0}`;
}

async function saveActiveFile(silent = false) {
    if (!state.activeFilePath || !state.currentStoryPath) return;
    const content = storyEditor.innerText;

    const tab = state.openTabs.find(t => t.path === state.activeFilePath);
    if (tab) tab.content = content;

    try {
        await invoke('write_file', {
            rootPath: state.currentStoryPath,
            filePath: state.activeFilePath,
            content,
        });
        if (!silent) showStatus("Đã lưu ✓");
        document.getElementById('word-count').innerText = `Chars: ${content.length}`;
    } catch (err) {
        console.error("Save failed:", err);
        showStatus("Lưu thất bại!");
    }
}

// ═══════════════════════════════════════════════
// 5. Node Operations (File/Folder CRUD)
// ═══════════════════════════════════════════════
function showNodeModal(parentPath, type) {
    showModal(`Tên ${type === 'folder' ? 'thư mục' : 'tập tin'} mới`, "create-node", { parentPath, type });
}

async function deleteNode(nodePath) {
    if (!confirm("Bạn có chắc chắn muốn xóa? Thư mục con cũng sẽ bị xóa.")) return;
    try {
        await invoke('delete_node', {
            rootPath: state.currentStoryPath,
            nodePath,
        });
        await loadNodes(state.currentStoryPath);
        if (state.activeFilePath === nodePath) closeTab(nodePath);
    } catch (err) {
        console.error("Delete failed:", err);
    }
}

function renameNodePrompt(nodePath, oldName) {
    showModal("Đổi tên", "rename-node", { nodePath, oldName });
}

// ═══════════════════════════════════════════════
// 6. AI Features
// ═══════════════════════════════════════════════
async function sendChat() {
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

async function runAiWriting(type) {
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
    const content = storyEditor.innerText;

    showStatus("AI đang viết...");
    if (!selection) storyEditor.innerText += "\n\n";

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

async function clearChat() {
    chatMessages.innerHTML = "";
    state.chatHistory = [];
    if (state.currentStoryPath) {
        await saveChatHistory();
    }
}

async function loadChatHistory() {
    if (!state.currentStoryPath) return;
    try {
        state.chatHistory = await invoke('get_chat_history', {
            rootPath: state.currentStoryPath,
        });
        chatMessages.innerHTML = "";
        state.chatHistory.forEach(msg => {
            addChatMessage(msg.role, msg.content);
        });
    } catch (err) {
        console.error("Failed to load chat history:", err);
        state.chatHistory = [];
    }
}

async function saveChatHistory() {
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

function addChatMessage(role, text) {
    const div = document.createElement('div');
    div.className = `message ${role === 'assistant' ? 'assistant' : 'user'}`;
    div.innerText = text;
    chatMessages.appendChild(div);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    return div;
}

// ═══════════════════════════════════════════════
// 7. Utilities
// ═══════════════════════════════════════════════
function showModal(title, action, extra = {}) {
    document.getElementById('modal-title').innerText = title;
    const input = document.getElementById('modal-input');
    input.value = extra.oldName || "";

    document.getElementById('modal-confirm').onclick = async () => {
        const val = input.value.trim();
        if (!val) return;

        if (action === "create-node") {
            try {
                await invoke('create_node', {
                    rootPath: state.currentStoryPath,
                    parentPath: extra.parentPath,
                    name: val,
                    nodeType: extra.type,
                });
                await loadNodes(state.currentStoryPath);
            } catch (err) {
                alert("Lỗi: " + err);
            }
        } else if (action === "create-story") {
            try {
                const storyName = val.replace(/[<>:"/\\|?*]/g, '').trim();
                if (!storyName) {
                    alert("Tên truyện không hợp lệ!");
                    return;
                }
                const sep = extra.parentDir.includes('\\') ? '\\' : '/';
                const newStoryPath = extra.parentDir + (extra.parentDir.endsWith(sep) ? '' : sep) + storyName;
                
                showStatus("Đang khởi tạo...");
                await invoke('initialize_story_folders', { rootPath: newStoryPath });
                await openStory(newStoryPath);
                showStatus("Đã khởi tạo truyện mới!");
            } catch (err) {
                alert("Lỗi: " + err);
            }
        } else if (action === "rename-node") {
            try {
                await invoke('rename_node', {
                    rootPath: state.currentStoryPath,
                    oldPath: extra.nodePath,
                    newName: val,
                });
                await loadNodes(state.currentStoryPath);
                
                const tab = state.openTabs.find(t => t.path === extra.nodePath);
                if (tab) {
                    const parts = tab.path.split(/[/\\]/);
                    parts[parts.length - 1] = val;
                    tab.path = parts.join('/');
                    tab.name = val;
                    if (state.activeFilePath === extra.nodePath) {
                        state.activeFilePath = tab.path;
                    }
                    renderTabs();
                }
            } catch (err) {
                alert("Lỗi: " + err);
            }
        }
        modalOverlay.classList.add('hidden');
    };
    modalOverlay.classList.remove('hidden');
    input.focus();
    input.onkeydown = (e) => {
        if (e.key === 'Enter') {
            document.getElementById('modal-confirm').click();
        }
    };
}

function showStatus(text) {
    const syncEl = document.getElementById('sync-status');
    syncEl.innerHTML = `<i data-lucide="loader"></i> ${text}`;
    lucide.createIcons();
    setTimeout(() => {
        syncEl.innerHTML = `<i data-lucide="hard-drive"></i> File System`;
        lucide.createIcons();
    }, 3000);
}

function escapeAttr(str) {
    return str.replace(/'/g, "\\'").replace(/"/g, '&quot;');
}

function debounce(func, timeout = 300) {
    let timer;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => { func.apply(this, args); }, timeout);
    };
}

function showStatus(text) {
    const syncEl = document.getElementById('sync-status');
    syncEl.innerHTML = `<i data-lucide="loader"></i> ${text}`;
    lucide.createIcons();
    setTimeout(() => {
        syncEl.innerHTML = `<i data-lucide="hard-drive"></i> File System`;
        lucide.createIcons();
    }, 3000);
}

function escapeAttr(str) {
    return str.replace(/'/g, "\\'").replace(/"/g, '&quot;');
}

function debounce(func, timeout = 300) {
    let timer;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => { func.apply(this, args); }, timeout);
    };
}
