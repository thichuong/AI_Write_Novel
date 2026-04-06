let currentStoryId = null;
let currentChapterId = null;
let chapters = [];
let activeBlock = null;

// Selectors
const chapterList = document.getElementById('chapter-list');
const storyEditor = document.getElementById('story-editor');
const chatMessages = document.getElementById('chat-messages');
const chatInput = document.getElementById('chat-input');
const sendChatBtn = document.getElementById('send-chat-btn');
const floatingToolbar = document.getElementById('floating-toolbar');
const modalOverlay = document.getElementById('modal-overlay');

// Initialization
document.addEventListener('DOMContentLoaded', async () => {
    await loadStories();
    setupEventListeners();
});

function setupEventListeners() {
    document.getElementById('new-story-btn').onclick = () => showModal("Tên truyện mới", "confirm-new-story");
    document.getElementById('add-chapter-btn').onclick = addChapter;
    document.getElementById('save-btn').onclick = saveCurrentChapter;
    sendChatBtn.onclick = sendChat;
    
    document.getElementById('modal-cancel').onclick = () => modalOverlay.classList.add('hidden');
    
    document.getElementById('ai-rewrite-btn').onclick = () => runAiWriting("rewrite");
    document.getElementById('ai-continue-btn').onclick = () => runAiWriting("continue");
    document.getElementById('ai-full-write-btn').onclick = () => runAiWriting("full");

    // Hide toolbar when clicking outside
    document.addEventListener('mousedown', (e) => {
        if (!floatingToolbar.contains(e.target) && !storyEditor.contains(e.target)) {
            floatingToolbar.classList.add('hidden');
        }
    });
}

// Modal handling
function showModal(title, action) {
    document.getElementById('modal-title').innerText = title;
    document.getElementById('modal-input').value = "";
    document.getElementById('modal-confirm').onclick = async () => {
        const val = document.getElementById('modal-input').value;
        if (action === "confirm-new-story") await createStory(val);
        modalOverlay.classList.add('hidden');
    };
    modalOverlay.classList.remove('hidden');
}

// API Calls
async function loadStories() {
    const res = await fetch('/api/stories');
    const stories = await res.json();
    if (stories.length > 0) {
        await selectStory(stories[0].id);
    }
}

async function createStory(title) {
    const res = await fetch('/api/stories', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title })
    });
    const story = await res.json();
    await loadStories();
    await selectStory(story.id);
}

async function selectStory(id) {
    currentStoryId = id;
    const res = await fetch(`/api/chapters/${id}`);
    chapters = await res.json();
    renderChapterList();
    if (chapters.length > 0) {
        selectChapter(chapters[0].id);
    }
}

function renderChapterList() {
    chapterList.innerHTML = "";
    chapters.forEach((chap, index) => {
        const div = document.createElement('div');
        div.className = `chapter-item ${chap.id === currentChapterId ? 'active' : ''}`;
        div.innerText = chap.title;
        div.onclick = () => selectChapter(chap.id);
        chapterList.appendChild(div);
    });
}

function selectChapter(id) {
    currentChapterId = id;
    const chap = chapters.find(c => c.id === id);
    document.getElementById('current-chapter-title').innerText = chap.title;
    renderEditor(chap.content);
    renderChapterList();
}

async function addChapter() {
    const res = await fetch(`/api/chapters/${currentStoryId}`, { method: 'POST' });
    const newChap = await res.json();
    chapters.push(newChap);
    renderChapterList();
    selectChapter(newChap.id);
}

// Editor Logic
function renderEditor(content) {
    storyEditor.innerHTML = "";
    const pTags = content.split('\n\n');
    pTags.forEach(p => {
        if (p.trim() || pTags.length === 1) {
            addBlock(p.trim());
        }
    });
    if (storyEditor.children.length === 0) addBlock("");
}

function addBlock(text) {
    const div = document.createElement('div');
    div.className = "story-block";
    div.contentEditable = "true";
    div.innerText = text;
    
    div.onfocus = (e) => {
        activeBlock = div;
        showToolbar(div);
    };

    div.onkeydown = (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            const newBlock = addBlock("");
            div.after(newBlock);
            newBlock.focus();
        }
    };

    storyEditor.appendChild(div);
    return div;
}

function showToolbar(element) {
    const rect = element.getBoundingClientRect();
    floatingToolbar.style.top = `${window.scrollY + rect.top - 50}px`;
    floatingToolbar.style.left = `${rect.left}px`;
    floatingToolbar.classList.remove('hidden');
}

async function saveCurrentChapter() {
    const content = Array.from(storyEditor.children).map(c => c.innerText).join('\n\n');
    await fetch(`/api/chapters/${currentChapterId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content })
    });
    // Update local state
    const chap = chapters.find(c => c.id === currentChapterId);
    if (chap) chap.content = content;
    alert("Đã lưu!");
}

// AI Actions
async function sendChat() {
    const msg = chatInput.value;
    if (!msg) return;
    
    addChatMessage("user", msg);
    chatInput.value = "";
    
    const context = Array.from(storyEditor.children).map(c => c.innerText).join('\n\n');
    
    const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            story_id: currentStoryId,
            message: msg,
            story_context: context
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

function addChatMessage(role, text) {
    const div = document.createElement('div');
    div.className = `message ${role}`;
    div.innerText = text;
    chatMessages.appendChild(div);
    chatMessages.scrollTop = chatMessages.scrollHeight;
    return div;
}

async function runAiWriting(type) {
    let targetBlock = activeBlock;
    let instruction = "";
    
    if (type === "rewrite") {
        instruction = "Hãy viết lại đoạn văn này cho hay hơn, sâu sắc hơn.";
    } else if (type === "continue") {
        instruction = "Hãy viết tiếp đoạn văn này.";
    } else if (type === "full") {
        instruction = "Dựa trên tất cả các chương trước đó và ý tưởng trong chat, hãy viết tiếp chương hiện tại.";
        // Focus last block or create new if needed
        if (!storyEditor.lastElementChild) addBlock("");
        targetBlock = storyEditor.lastElementChild;
    }

    if (!targetBlock) return;
    
    const selection = (type === "rewrite") ? window.getSelection().toString() : "";
    const context = Array.from(storyEditor.children).map(c => c.innerText).join('\n\n');

    targetBlock.classList.add('ai-thinking');
    
    if (type === "continue" || type === "full") {
        const nextBlock = addBlock("");
        targetBlock.after(nextBlock);
        targetBlock = nextBlock;
    }
    
    targetBlock.innerText = ""; // Clear for streaming

    const response = await fetch('/api/write', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            story_id: currentStoryId,
            current_chapter_id: currentChapterId,
            instruction: instruction,
            story_context: context,
            selection_context: selection || null
        })
    });

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    
    while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        targetBlock.innerText += decoder.decode(value);
        storyEditor.scrollTop = storyEditor.scrollHeight;
    }
    targetBlock.classList.remove('ai-thinking');
}
