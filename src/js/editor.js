import { state } from './state.js';
import { invoke, fs, listen, path } from './services/tauri.js';
import { escapeAttr, showStatus } from './utils.js';

export async function openFile(input, shouldFocus = true) {
    console.log("openFile called with:", input, "shouldFocus:", shouldFocus);
    try {
        let node;
        if (typeof input === 'string') {
            // If input is a path string, normalize it
            let fullPath = input;
            if (state.currentStoryPath && !input.startsWith('/') && !input.includes(':')) {
                // Use absolute path join for reliable file reading
                try {
                    fullPath = await path.join(state.currentStoryPath, input);
                } catch (e) {
                    console.warn("Path join failed, falling back to simple join:", e);
                    const sep = state.currentStoryPath.includes('\\') ? '\\' : '/';
                    fullPath = state.currentStoryPath + (state.currentStoryPath.endsWith(sep) ? '' : sep) + input;
                }
            }
            
            const name = input.split(/[/\\]/).pop();
            node = { path: fullPath, name: name };
        } else {
            node = input;
        }

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

        // Only change active file if shouldFocus is true OR if there's no active file
        if (shouldFocus || !state.activeFilePath) {
            state.activeFilePath = node.path;
        }

        renderTabs();

        // Always update editor content if the file being opened IS the active file
        // This ensures the editor refreshes if the AI updates the file currently being viewed
        if (state.activeFilePath === node.path) {
            loadEditorContent(tab);
        }
    } catch (err) {
        console.error("Failed to open file:", err);
        showStatus("Không thể mở file", true);
    }
}

export function renderTabs() {
    const tabsList = document.getElementById('tabs-list');
    if (!tabsList) return;
    
    tabsList.innerHTML = "";
    let activeTabEl = null;

    state.openTabs.forEach(tab => {
        const tabEl = document.createElement('div');
        const isActive = tab.path === state.activeFilePath;
        tabEl.className = `tab ${isActive ? 'active' : ''} ${tab.dirty ? 'dirty' : ''}`;
        tabEl.innerHTML = `
            <span>${tab.name}</span>
            <div class="dirty-indicator"></div>
            <i data-lucide="x" class="tab-close" title="Đóng" onclick="event.stopPropagation(); window.closeTab('${escapeAttr(tab.path)}')"></i>
        `;
        tabEl.onclick = () => {
            state.activeFilePath = tab.path;
            renderTabs();
            loadEditorContent(tab);
        };
        tabsList.appendChild(tabEl);
        if (isActive) activeTabEl = tabEl;
    });

    if (window.lucide) window.lucide.createIcons();

    // Auto-scroll to active tab
    if (activeTabEl) {
        requestAnimationFrame(() => {
            activeTabEl.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'center' });
        });
    }
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
    const markdownViewer = document.getElementById('markdown-viewer');
    const toggleEditModeBtn = document.getElementById('toggle-edit-mode-btn');
    
    if (currentFileName) currentFileName.innerText = tab.name;
    
    const isMd = tab.name.toLowerCase().endsWith('.md');
    
    if (isMd) {
        // Initialize view mode if not set
        if (!state.viewModes[tab.path]) {
            state.viewModes[tab.path] = 'view';
        }
        
        const mode = state.viewModes[tab.path];
        
        if (toggleEditModeBtn) {
            toggleEditModeBtn.classList.remove('hidden');
            if (mode === 'view') {
                toggleEditModeBtn.innerHTML = `<i data-lucide="edit-2"></i><span class="btn-text">Sửa</span>`;
                toggleEditModeBtn.title = "Chuyển sang chế độ Chỉnh sửa";
            } else {
                toggleEditModeBtn.innerHTML = `<i data-lucide="eye"></i><span class="btn-text">Xem</span>`;
                toggleEditModeBtn.title = "Chuyển sang chế độ Xem Markdown";
            }
        }
        
        if (mode === 'view') {
            if (storyEditor) storyEditor.classList.add('hidden');
            if (markdownViewer) {
                markdownViewer.classList.remove('hidden');
                if (window.marked && typeof window.marked.parse === 'function') {
                    markdownViewer.innerHTML = window.marked.parse(tab.content || "");
                } else if (window.marked) {
                    markdownViewer.innerHTML = window.marked(tab.content || "");
                } else {
                    markdownViewer.innerText = tab.content || "";
                }
            }
        } else {
            if (markdownViewer) markdownViewer.classList.add('hidden');
            if (storyEditor) {
                storyEditor.classList.remove('hidden');
                storyEditor.innerText = tab.content || "";
                storyEditor.contentEditable = "true";
            }
        }
    } else {
        if (toggleEditModeBtn) toggleEditModeBtn.classList.add('hidden');
        if (markdownViewer) markdownViewer.classList.add('hidden');
        if (storyEditor) {
            storyEditor.classList.remove('hidden');
            storyEditor.innerText = tab.content || "";
            // Disable editing for system files starting with dot
            storyEditor.contentEditable = tab.name.startsWith('.') ? "false" : "true";
        }
    }
    
    const wordCount = document.getElementById('word-count');
    if (wordCount) wordCount.innerText = `Chars: ${tab.content ? tab.content.length : 0}`;
    
    if (window.lucide) window.lucide.createIcons();
}

export async function saveActiveFile(silent = false) {
    if (!state.activeFilePath || !state.currentStoryPath) return;
    
    const isMd = state.activeFilePath.toLowerCase().endsWith('.md');
    const currentMode = state.viewModes[state.activeFilePath] || 'view';
    
    let content;
    if (isMd && currentMode === 'view') {
        const tab = state.openTabs.find(t => t.path === state.activeFilePath);
        content = tab ? tab.content : "";
    } else {
        const storyEditor = document.getElementById('story-editor');
        content = storyEditor ? storyEditor.innerText : "";
    }

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
        showStatus("Lưu thất bại!", true);
    }
}

export function setupEditorListeners() {
    console.log("Setting up editor listeners...");
    listen('open-file', (event) => {
        console.log("Received open-file event:", event);
        const { path: filePath } = event.payload;
        if (filePath) {
            console.log("Triggering openFile (no-focus) for:", filePath);
            // Pass false for shouldFocus to avoid disrupting user reading
            openFile(filePath, false);
        }
    });

    // Set up toggle edit mode button listener
    const toggleBtn = document.getElementById('toggle-edit-mode-btn');
    if (toggleBtn) {
        toggleBtn.addEventListener('click', () => {
            if (!state.activeFilePath) return;
            const isMd = state.activeFilePath.toLowerCase().endsWith('.md');
            if (!isMd) return;
            
            // Toggle view mode
            const currentMode = state.viewModes[state.activeFilePath] || 'view';
            const newMode = currentMode === 'view' ? 'edit' : 'view';
            state.viewModes[state.activeFilePath] = newMode;
            
            // Reload content to update view
            const tab = state.openTabs.find(t => t.path === state.activeFilePath);
            if (tab) {
                // If switching from Edit to View, make sure we capture current editor changes
                if (newMode === 'view') {
                    const storyEditor = document.getElementById('story-editor');
                    if (storyEditor) {
                        tab.content = storyEditor.innerText;
                    }
                }
                loadEditorContent(tab);
            }
        });
    }
}

// Attach globals for inline HTML event handlers
window.closeTab = closeTab;
