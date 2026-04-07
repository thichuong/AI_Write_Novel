import { state } from './state.js';
import { debounce } from './utils.js';
import { setupAIListeners, sendChat, clearChat, runAiWriting } from './ai.js';
import { handleCreateStory, handleOpenFolder, openStory, loadNodes } from './fileExplorer.js';
import { saveActiveFile } from './editor.js';
import { invoke } from './services/tauri.js';

document.addEventListener('DOMContentLoaded', async () => {
    setupEventListeners();
    setupAIListeners();
    
    // Load last opened folder
    const lastPath = localStorage.getItem('last_story_path');
    if (lastPath) {
        await openStory(lastPath);
    }
    
    if (window.lucide) window.lucide.createIcons();
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
    const saveBtn = document.getElementById('save-btn');
    if (saveBtn) saveBtn.onclick = () => saveActiveFile();
    
    const aiFullWriteBtn = document.getElementById('ai-full-write-btn');
    if (aiFullWriteBtn) aiFullWriteBtn.onclick = () => runAiWriting("full");

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

    // Floating Toolbar
    const rewriteBtn = document.getElementById('ai-rewrite-btn');
    if (rewriteBtn) rewriteBtn.onclick = () => runAiWriting("rewrite");
    
    const continueBtn = document.getElementById('ai-continue-btn');
    if (continueBtn) continueBtn.onclick = () => runAiWriting("continue");

    // Click outside toolbar
    const storyEditor = document.getElementById('story-editor');
    document.addEventListener('mousedown', (e) => {
        const toolbar = document.getElementById('floating-toolbar');
        if (toolbar && storyEditor && !toolbar.contains(e.target) && !storyEditor.contains(e.target)) {
            toolbar.classList.add('hidden');
        }
    });

    // Editor Auto-save on input (debounced)
    if (storyEditor) {
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
