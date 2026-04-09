import { state } from './state.js';
import { debounce } from './utils.js';
import { setupAIListeners, sendChat, clearChat } from './ai.js';
import { handleCreateStory, handleOpenFolder, openStory, loadNodes, setupExplorerListeners, handleNewFile, handleNewFolder, handleRename, handleDeleteBtn } from './fileExplorer.js';
import { saveActiveFile } from './editor.js';
import { invoke, fs, path } from './services/tauri.js';

document.addEventListener('DOMContentLoaded', async () => {
    setupEventListeners();
    setupAIListeners();
    setupExplorerListeners();
    
    // Load last opened folder
    const lastPath = localStorage.getItem('last_story_path');
    if (lastPath) {
        await openStory(lastPath);
    }
    
    if (window.lucide) window.lucide.createIcons();
});

function setupEventListeners() {
    // Activity Bar
    const navItems = ['explorer', 'search', 'story-settings', 'wiki', 'profile'];
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

    const newFileBtn = document.getElementById('new-file-btn');
    if (newFileBtn) newFileBtn.onclick = handleNewFile;

    const newFolderBtn = document.getElementById('new-folder-btn');
    if (newFolderBtn) newFolderBtn.onclick = handleNewFolder;

    const renameNodeBtn = document.getElementById('rename-node-btn');
    if (renameNodeBtn) renameNodeBtn.onclick = handleRename;

    const deleteNodeBtn = document.getElementById('delete-node-btn');
    if (deleteNodeBtn) deleteNodeBtn.onclick = handleDeleteBtn;

    // Editor Actions
    // (Manual save button removed, using auto-save and Ctrl+S)
    
    // Chat Actions
    const sendChatBtn = document.getElementById('send-chat-btn');
    if (sendChatBtn) sendChatBtn.onclick = sendChat;
    
    const clearChatBtn = document.getElementById('clear-chat-btn');
    if (clearChatBtn) clearChatBtn.onclick = clearChat;

    const chatInput = document.getElementById('chat-input');
    if (chatInput) {
        chatInput.onkeydown = (e) => {
            if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendChat(); }
        };
    }

    // Modal
    const modalCancel = document.getElementById('modal-cancel');
    const modalOverlay = document.getElementById('modal-overlay');
    if (modalCancel && modalOverlay) {
        modalCancel.onclick = () => modalOverlay.classList.add('hidden');
    }

    const storyEditor = document.getElementById('story-editor');

    // Editor Auto-save on input (debounced)
    if (storyEditor) {
        storyEditor.addEventListener('input', () => {
            if (state.activeFilePath) {
                const tab = state.openTabs.find(t => t.path === state.activeFilePath);
                if (tab && !tab.dirty) {
                    tab.dirty = true;
                    import('./editor.js').then(module => module.renderTabs());
                }
            }
        });

        storyEditor.addEventListener('input', debounce(() => {
            if (state.activeFilePath) saveActiveFile(true);
        }, 2000));
    }

    // Keyboard shortcut: Ctrl+S to save
    document.addEventListener('keydown', (e) => {
        if (e.ctrlKey && e.key === 's') {
            e.preventDefault();
            saveActiveFile();
        }
    });
}

function switchActivity(activity) {
    document.querySelectorAll('.activity-icon').forEach(el => el.classList.remove('active'));
    const target = document.getElementById(`nav-${activity}`);
    if (target) target.classList.add('active');

    // Special logic for Wiki shortcut
    if (activity === 'wiki') {
        const explorerTree = document.getElementById('explorer-tree');
        if (explorerTree) {
            const wikiItem = Array.from(explorerTree.querySelectorAll('sl-tree-item')).find(item => item.dataset.name === 'wiki');
            if (wikiItem) {
                wikiItem.scrollIntoView({ behavior: 'smooth', block: 'center' });
                wikiItem.expanded = true;
                // Highlight or select it
                wikiItem.selected = true;
            }
        }
        // Ensure UI stays on explorer (since we only have one sidebar area)
        document.querySelectorAll('.activity-icon').forEach(el => el.classList.remove('active'));
        document.getElementById('nav-explorer').classList.add('active');
        document.getElementById('nav-wiki').classList.add('active');
    }
}

// Map showModal to window so fileExplorer/others can call it
window.showModal = function(title, action, extra = {}) {
    document.getElementById('modal-title').innerText = title;
    const input = document.getElementById('modal-input');
    input.value = extra.oldName || "";

    const modalOverlay = document.getElementById('modal-overlay');
    
    document.getElementById('modal-confirm').onclick = async () => {
        const val = input.value.trim();
        if (!val) return;

        if (action === "create-node") {
            try {
                let fileName = val;
                if (extra.type === 'file' && !fileName.includes('.')) {
                    fileName += '.md';
                }
                const newPath = await path.join(extra.parentPath, fileName);
                
                if (extra.type === 'file') {
                    await fs.writeTextFile(newPath, "");
                } else {
                    await fs.mkdir(newPath);
                }
                
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
                
                const syncEl = document.getElementById('sync-status');
                if (syncEl) syncEl.innerHTML = `<i data-lucide="loader"></i> Đang khởi tạo...`;
                
                await invoke('initialize_story_folders', { rootPath: newStoryPath });
                await openStory(newStoryPath);
                
                if (syncEl) syncEl.innerHTML = `<i data-lucide="hard-drive"></i> Đã khởi tạo truyện mới!`;
            } catch (err) {
                alert("Lỗi: " + err);
            }
        } else if (action === "rename-node") {
            try {
                const parentDir = await path.dirname(extra.nodePath);
                const newPath = await path.join(parentDir, val);
                await fs.rename(extra.nodePath, newPath);
                
                await loadNodes(state.currentStoryPath);
                
                const tab = state.openTabs.find(t => t.path === extra.nodePath);
                if (tab) {
                    tab.path = newPath;
                    tab.name = val;
                    if (state.activeFilePath === extra.nodePath) {
                        state.activeFilePath = newPath;
                    }
                    import('./editor.js').then(module => module.renderTabs());
                }
            } catch (err) {
                alert("Lỗi: " + err);
            }
        }
        if (modalOverlay) modalOverlay.classList.add('hidden');
    };
    
    if (modalOverlay) modalOverlay.classList.remove('hidden');
    input.focus();
    input.onkeydown = (e) => {
        if (e.key === 'Enter') {
            document.getElementById('modal-confirm').click();
        }
    };
};
