/**
 * AI Novelist IDE - Core Application Logic
 */

let state = {
    currentStoryId: null,
    nodes: [], // Full tree of nodes for the current story
    openTabs: [], // Array of node objects
    activeNodeId: null,
    stories: [],
    isSaving: false
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

// 1. Initialization
document.addEventListener('DOMContentLoaded', async () => {
    await loadStories();
    setupEventListeners();
    lucide.createIcons();
});

function setupEventListeners() {
    // Activity Bar
    document.getElementById('nav-explorer').onclick = () => switchActivity('explorer');
    document.getElementById('new-story-btn').onclick = () => showModal("Tên truyện mới", "create-story");
    document.getElementById('refresh-explorer').onclick = () => loadNodes(state.currentStoryId);
    
    // Editor Actions
    document.getElementById('save-btn').onclick = () => saveActiveFile();
    document.getElementById('ai-full-write-btn').onclick = () => runAiWriting("full");
    
    // Chat Actions
    document.getElementById('send-chat-btn').onclick = sendChat;
    document.getElementById('clear-chat-btn').onclick = () => { chatMessages.innerHTML = ""; };
    chatInput.onkeydown = (e) => { if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendChat(); } };

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

    // Editor Auto-save on blur or input (debounced)
    storyEditor.addEventListener('input', debounce(() => {
        if (state.activeNodeId) saveActiveFile(true);
    }, 2000));
}

function switchActivity(activity) {
    document.querySelectorAll('.activity-icon').forEach(el => el.classList.remove('active'));
    document.getElementById(`nav-${activity}`).classList.add('active');
    // Implement other panes if needed
}

// 2. Models & API
async function loadStories() {
    const res = await fetch('/api/stories');
    state.stories = await res.json();
    renderStoriesList();
    if (state.stories.length > 0 && !state.currentStoryId) {
        await selectStory(state.stories[0]);
    }
}

function renderStoriesList() {
    const storiesList = document.getElementById('stories-list');
    storiesList.innerHTML = "";
    state.stories.forEach(story => {
        const item = document.createElement('div');
        item.className = `story-list-item ${story.id === state.currentStoryId ? 'active' : ''}`;
        item.innerHTML = `<i data-lucide="book"></i> <span>${story.title}</span>`;
        item.onclick = () => selectStory(story);
        storiesList.appendChild(item);
    });
    lucide.createIcons();
}

async function selectStory(story) {
    if (state.currentStoryId && state.currentStoryId !== story.id) {
        // Clear editor and tabs when switching stories
        state.openTabs = [];
        state.activeNodeId = null;
        renderTabs();
        storyEditor.innerText = "";
        currentFileName.innerText = "Chọn một file để bắt đầu";
    }

    state.currentStoryId = story.id;
    statusStoryName.innerText = story.title;
    document.getElementById('story-title-display').innerText = story.title.toUpperCase();
    
    renderStoriesList();
    await loadNodes(story.id);
}

async function loadNodes(storyId) {
    const res = await fetch(`/api/stories/${storyId}/nodes`);
    state.nodes = await res.json();
    renderExplorer();
}

async function saveActiveFile(silent = false) {
    if (!state.activeNodeId) return;
    const content = storyEditor.innerText;
    
    // Update local state first
    const node = state.nodes.find(n => n.id === state.activeNodeId);
    if (node) node.content = content;

    try {
        await fetch(`/api/nodes/${state.activeNodeId}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ content })
        });
        if (!silent) showStatus("Saved successfully");
    } catch (e) {
        console.error("Save failed", e);
    }
}

// 3. UI Rendering: Explorer
function renderExplorer() {
    explorerTree.innerHTML = "";
    const rootNodes = state.nodes.filter(n => !n.parent_id);
    rootNodes.forEach(node => {
        explorerTree.appendChild(createNodeElement(node));
    });
    lucide.createIcons();
}

function createNodeElement(node) {
    const container = document.createElement('div');
    container.className = 'explorer-node';
    
    const item = document.createElement('div');
    item.className = `node-item ${node.id === state.activeNodeId ? 'active' : ''} ${node.type === 'folder' ? 'node-folder' : 'node-file'}`;
    
    const icon = node.type === 'folder' ? 'folder' : 'file-text';
    item.innerHTML = `
        <i data-lucide="${icon}" class="node-icon"></i>
        <span class="node-name">${node.name}</span>
        <div class="node-actions">
            ${node.type === 'folder' ? `
                <button onclick="event.stopPropagation(); showNodeModal(${node.id}, 'file')" title="Add File"><i data-lucide="file-plus"></i></button>
                <button onclick="event.stopPropagation(); showNodeModal(${node.id}, 'folder')" title="Add Folder"><i data-lucide="folder-plus"></i></button>
            ` : ''}
            <button onclick="event.stopPropagation(); deleteNode(${node.id})" title="Delete"><i data-lucide="trash-2"></i></button>
        </div>
    `;

    item.onclick = () => {
        if (node.type === 'folder') {
            const contents = container.querySelector('.folder-contents');
            if (contents) contents.classList.toggle('hidden');
        } else {
            openFile(node);
        }
    };

    container.appendChild(item);

    if (node.type === 'folder') {
        const contents = document.createElement('div');
        contents.className = 'folder-contents';
        const children = state.nodes.filter(n => n.parent_id === node.id);
        children.forEach(child => {
            contents.appendChild(createNodeElement(child));
        });
        container.appendChild(contents);
    }

    return container;
}

// 4. Tabs & Editor
function openFile(node) {
    if (!state.openTabs.find(t => t.id === node.id)) {
        state.openTabs.push(node);
    }
    state.activeNodeId = node.id;
    renderTabs();
    loadEditorContent(node);
    renderExplorer(); // Refresh active state
}

function renderTabs() {
    tabsList.innerHTML = "";
    state.openTabs.forEach(node => {
        const tab = document.createElement('div');
        tab.className = `tab ${node.id === state.activeNodeId ? 'active' : ''}`;
        tab.innerHTML = `
            <span>${node.name}</span>
            <i data-lucide="x" class="tab-close" onclick="event.stopPropagation(); closeTab(${node.id})"></i>
        `;
        tab.onclick = () => openFile(node);
        tabsList.appendChild(tab);
    });
    lucide.createIcons();
}

function closeTab(nodeId) {
    state.openTabs = state.openTabs.filter(t => t.id !== nodeId);
    if (state.activeNodeId === nodeId) {
        state.activeNodeId = state.openTabs.length > 0 ? state.openTabs[state.openTabs.length - 1].id : null;
    }
    renderTabs();
    if (state.activeNodeId) {
        loadEditorContent(state.nodes.find(n => n.id === state.activeNodeId));
    } else {
        storyEditor.innerText = "";
        currentFileName.innerText = "Chào mừng";
    }
}

function loadEditorContent(node) {
    currentFileName.innerText = node.name;
    storyEditor.innerText = node.content || "";
    document.getElementById('word-count').innerText = `Chars: ${node.content ? node.content.length : 0}`;
}

// 5. Node Operations
function showNodeModal(parentId, type) {
    showModal(`Tên ${type === 'folder' ? 'thư mục' : 'tập tin'} mới`, "create-node", { parentId, type });
}

async function deleteNode(id) {
    if (!confirm("Bạn có chắc chắn muốn xóa? Thư mục con cũng sẽ bị xóa.")) return;
    await fetch(`/api/nodes/${id}`, { method: 'DELETE' });
    await loadNodes(state.currentStoryId);
    if (state.activeNodeId === id) closeTab(id);
}

// 6. AI Features
async function sendChat() {
    const msg = chatInput.value;
    if (!msg) return;
    
    addChatMessage("user", msg);
    chatInput.value = "";
    
    const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            story_id: state.currentStoryId,
            message: msg,
            story_context: storyEditor.innerText
        })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let aiMsgDiv = addChatMessage("assistant", "");
    
    while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        aiMsgDiv.innerText += decoder.decode(value);
        chatMessages.scrollTop = chatMessages.scrollHeight;
    }
}

async function runAiWriting(type) {
    let instruction = "";
    if (type === "rewrite") instruction = "Hãy sửa lại đoạn văn này cho tinh tế và giàu cảm xúc hơn.";
    else if (type === "continue") instruction = "Hãy viết tiếp đoạn văn này một cách tự nhiên.";
    else if (type === "full") instruction = "Dựa trên các quy tắc và nhân vật, hãy phát triển tiếp nội dung cho chương này.";

    const selection = window.getSelection().toString();
    const context = storyEditor.innerText;

    // Show loading or visual indicator
    showStatus("AI is thinking...");
    
    const response = await fetch('/api/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            story_id: state.currentStoryId,
            current_chapter_id: state.activeNodeId || 0, // Fallback if no file open
            instruction: instruction,
            story_context: context,
            selection_context: selection || null
        })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    
    // If not a selection, append to end
    if (!selection) storyEditor.innerText += "\n\n";

    while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        const chunk = decoder.decode(value);
        
        if (selection) {
            // Complex to replace selection in real-time in contenteditable, simplified:
            // Append for now or replace once at start? Let's just append to end for 'continue' and 'full'.
        }
        storyEditor.innerText += chunk;
        storyEditor.scrollTop = storyEditor.scrollHeight;
    }
    showStatus("AI finished.");
    saveActiveFile(true);
}

function addChatMessage(role, text) {
    const div = document.createElement('div');
    div.className = `message ${role}`;
    div.innerText = text;
    chatMessages.appendChild(div);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    return div;
}

// Utilities
function showModal(title, action, extra = {}) {
    document.getElementById('modal-title').innerText = title;
    document.getElementById('modal-input').value = "";
    document.getElementById('modal-confirm').onclick = async () => {
        const val = document.getElementById('modal-input').value;
        if (action === "create-story") {
            const res = await fetch('/api/stories', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ title: val })
            });
            const story = await res.json();
            await loadStories();
            await selectStory(story);
        } else if (action === "create-node") {
            await fetch('/api/nodes', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    story_id: state.currentStoryId,
                    parent_id: extra.parentId,
                    name: val,
                    type: extra.type,
                    category: state.nodes.find(n => n.id === extra.parentId)?.category
                })
            });
            await loadNodes(state.currentStoryId);
        }
        modalOverlay.classList.add('hidden');
    };
    modalOverlay.classList.remove('hidden');
}

function showStatus(text) {
    document.getElementById('sync-status').innerHTML = `<i data-lucide="loader"></i> ${text}`;
    lucide.createIcons();
    setTimeout(() => {
        document.getElementById('sync-status').innerHTML = `<i data-lucide="cloud-check"></i> Connected`;
        lucide.createIcons();
    }, 3000);
}

function debounce(func, timeout = 300) {
    let timer;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => { func.apply(this, args); }, timeout);
    };
}
