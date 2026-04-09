import { state } from './state.js';
import { invoke, fs } from './services/tauri.js';
import { escapeAttr, showStatus } from './utils.js';
import { renderExplorer } from './fileExplorer.js';

export async function openFile(node) {
    try {
        let content = await fs.readTextFile(node.path);

        // Special handling for chat history JSON
        if (node.name === 'chat_history.json') {
            try {
                const json = JSON.parse(content);
                content = JSON.stringify(json, null, 2);
            } catch (e) {
                console.warn("Failed to parse chat history JSON:", e);
            }
        }

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
        
        // Disable editing for system files
        const storyEditor = document.getElementById('story-editor');
        if (storyEditor) {
            storyEditor.contentEditable = node.name.startsWith('.') ? "false" : "true";
        }
    } catch (err) {
        console.error("Failed to open file:", err);
        alert("Không thể mở file: " + err);
    }
}

export function renderTabs() {
    const tabsList = document.getElementById('tabs-list');
    if (!tabsList) return;
    
    tabsList.innerHTML = "";
    state.openTabs.forEach(tab => {
        const tabEl = document.createElement('div');
        tabEl.className = `tab ${tab.path === state.activeFilePath ? 'active' : ''} ${tab.dirty ? 'dirty' : ''}`;
        tabEl.innerHTML = `
            <span>${tab.name}</span>
            <div class="dirty-indicator"></div>
            <i data-lucide="x" class="tab-close" onclick="event.stopPropagation(); window.closeTab('${escapeAttr(tab.path)}')"></i>
        `;
        tabEl.onclick = () => {
            state.activeFilePath = tab.path;
            renderTabs();
            loadEditorContent(tab);
        };
        tabsList.appendChild(tabEl);
    });
    if (window.lucide) window.lucide.createIcons();
}

export function closeTab(filePath) {
    state.openTabs = state.openTabs.filter(t => t.path !== filePath);
    if (state.activeFilePath === filePath) {
        state.activeFilePath = state.openTabs.length > 0 ? state.openTabs[state.openTabs.length - 1].path : null;
    }
    renderTabs();
    
    const storyEditor = document.getElementById('story-editor');
    const currentFileName = document.getElementById('current-file-name');
    
    if (state.activeFilePath) {
        const tab = state.openTabs.find(t => t.path === state.activeFilePath);
        if (tab) loadEditorContent(tab);
    } else {
        if (storyEditor) storyEditor.innerText = "";
        if (currentFileName) currentFileName.innerText = "Chào mừng";
    }
}

export function loadEditorContent(tab) {
    const currentFileName = document.getElementById('current-file-name');
    const storyEditor = document.getElementById('story-editor');
    
    if (currentFileName) currentFileName.innerText = tab.name;
    if (storyEditor) storyEditor.innerText = tab.content || "";
    
    const wordCount = document.getElementById('word-count');
    if (wordCount) wordCount.innerText = `Chars: ${tab.content ? tab.content.length : 0}`;
}

export async function saveActiveFile(silent = false) {
    if (!state.activeFilePath || !state.currentStoryPath) return;
    const storyEditor = document.getElementById('story-editor');
    const content = storyEditor ? storyEditor.innerText : "";

    const tab = state.openTabs.find(t => t.path === state.activeFilePath);
    if (tab) tab.content = content;

    try {
        await fs.writeTextFile(state.activeFilePath, content);
        
        // Clear dirty state
        if (tab) tab.dirty = false;
        renderTabs();

        if (!silent) showStatus("Đã lưu ✓");
        
        const wordCount = document.getElementById('word-count');
        if (wordCount) wordCount.innerText = `Chars: ${content.length}`;
    } catch (err) {
        console.error("Save failed:", err);
        showStatus("Lưu thất bại!");
    }
}

// Attach globals for inline HTML event handlers
window.closeTab = closeTab;
