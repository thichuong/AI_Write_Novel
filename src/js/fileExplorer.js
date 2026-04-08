import { state } from './state.js';
import { fs, path, listen, openDialog } from './services/tauri.js';
import { escapeAttr, showStatus } from './utils.js';
import { openFile, renderTabs } from './editor.js';
import { loadChatHistory } from './ai.js';

export function setupExplorerListeners() {
    listen('file-system-changed', () => {
        if (state.currentStoryPath) {
            loadNodes(state.currentStoryPath);
        }
    });

    // Listen to sl-tree events if needed
    const explorerTree = document.getElementById('explorer-tree');
    if (explorerTree) {
        explorerTree.addEventListener('sl-selection-change', (event) => {
            const selectedItem = event.detail.selection[0];
            if (selectedItem && selectedItem.dataset.type === 'file') {
                const node = {
                    path: selectedItem.dataset.path,
                    name: selectedItem.dataset.name,
                    nodeType: 'file'
                };
                openFile(node);
            }
        });
    }
}

export async function handleOpenFolder() {
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

export async function handleCreateStory() {
    try {
        const selected = await openDialog({
            directory: true,
            multiple: false,
            title: "Chọn thư mục cha để tạo truyện mới"
        });
        
        if (selected) {
            window.showModal("Tên truyện mới", "create-story", { parentDir: selected });
        }
    } catch (err) {
        console.error("Failed to open dialog:", err);
        alert("Lỗi: " + err);
    }
}

export async function openStory(storyPath) {
    const storyEditor = document.getElementById('story-editor');
    const currentFileName = document.getElementById('current-file-name');
    const statusStoryName = document.getElementById('status-story-name');

    if (state.currentStoryPath && state.currentStoryPath !== storyPath) {
        state.openTabs = [];
        state.activeFilePath = null;
        renderTabs();
        if (storyEditor) storyEditor.innerText = "";
        if (currentFileName) currentFileName.innerText = "Chọn một file để bắt đầu";
    }

    state.currentStoryPath = storyPath;
    
    // Get folder name from path safely
    let folderName = "Story";
    try {
        folderName = await path.basename(storyPath);
    } catch(e) {
        folderName = storyPath.split(/[/\\]/).pop() || storyPath;
    }
    
    if (statusStoryName) statusStoryName.innerText = folderName;
    
    const storyTitleDisplay = document.getElementById('story-title-display');
    if (storyTitleDisplay) storyTitleDisplay.innerText = folderName.toUpperCase();

    localStorage.setItem('last_story_path', storyPath);

    await window.loadNodes(storyPath);
    await loadChatHistory();
}

export async function loadNodes(rootPath) {
    if (!rootPath) return;
    try {
        // Use Tauri FS readDir instead of Rust invoke
        const entries = await fs.readDir(rootPath);
        
        // Map entries to simplified node format with proper absolute paths
        state.nodes = await Promise.all(entries.map(async (entry) => {
            const entryPath = entry.path || await path.join(rootPath, entry.name);
            return {
                name: entry.name || "unknown",
                path: entryPath,
                nodeType: entry.isDirectory ? 'folder' : 'file',
                children: []
            };
        }));
        
        state.nodes.sort((a, b) => {
            // Folders first, then alphabetically
            if (a.nodeType === 'folder' && b.nodeType !== 'folder') return -1;
            if (a.nodeType !== 'folder' && b.nodeType === 'folder') return 1;
            return a.name.localeCompare(b.name);
        });
        
        renderExplorer();
    } catch (err) {
        console.error("Failed to load nodes:", err);
        alert("Không thể tải danh sách file: " + err);
    }
}
window.loadNodes = loadNodes;

export function renderExplorer() {
    const explorerTree = document.getElementById('explorer-tree');
    if (!explorerTree) return;

    explorerTree.innerHTML = "";
    state.nodes.forEach(node => {
        explorerTree.appendChild(createNodeElement(node));
    });
}

function createNodeElement(node) {
    const treeItem = document.createElement('sl-tree-item');
    treeItem.innerText = node.name; // Label in default slot
    treeItem.dataset.path = node.path;
    treeItem.dataset.name = node.name;
    treeItem.dataset.type = node.nodeType;

    // Set icons
    const customIcon = document.createElement('sl-icon');
    customIcon.slot = 'icon';
    customIcon.name = node.nodeType === 'folder' ? 'folder' : 'file-earmark-text';
    treeItem.appendChild(customIcon);

    if (node.nodeType === 'folder') {
        treeItem.lazy = true;
        
        // Load children on expand
        treeItem.addEventListener('sl-expand', async (event) => {
            if (treeItem.children.length <= 1) { // Only icon slot exists
                try {
                    const children = await fs.readDir(node.path);
                    const childNodes = await Promise.all(children.map(async (entry) => {
                        const entryPath = entry.path || await path.join(node.path, entry.name);
                        return {
                            name: entry.name || "unknown",
                            path: entryPath,
                            nodeType: entry.isDirectory ? 'folder' : 'file'
                        };
                    }));
                    
                    childNodes.sort((a, b) => {
                        if (a.nodeType === 'folder' && b.nodeType !== 'folder') return -1;
                        if (a.nodeType !== 'folder' && b.nodeType === 'folder') return 1;
                        return a.name.localeCompare(b.name);
                    });
                    
                    childNodes.forEach(child => {
                        treeItem.appendChild(createNodeElement(child));
                    });
                } catch (err) {
                    console.error("Failed to load sub-folder:", err);
                }
            }
        });
    }

    // Add context menu / actions (simplified for now as sl-tree handles selection)
    // We can add buttons in a slot if needed but standard sl-tree-item is cleaner
    
    return treeItem;
}

export function showNodeModal(parentPath, type) {
    window.showModal(`Tên ${type === 'folder' ? 'thư mục' : 'tập tin'} mới`, "create-node", { parentPath, type });
}
window.showNodeModal = showNodeModal;

export async function deleteNode(nodePath) {
    if (!confirm("Bạn có chắc chắn muốn xóa? Thư mục con cũng sẽ bị xóa.")) return;
    try {
        await fs.remove(nodePath, { recursive: true });
        if (state.currentStoryPath) {
            await loadNodes(state.currentStoryPath);
        }
        if (state.activeFilePath === nodePath) {
            import('./editor.js').then(module => module.closeTab(nodePath));
        }
    } catch (err) {
        console.error("Delete failed:", err);
        alert("Xóa thất bại: " + err);
    }
}
window.deleteNode = deleteNode;

export function renameNodePrompt(nodePath, oldName) {
    window.showModal("Đổi tên", "rename-node", { nodePath, oldName });
}
window.renameNodePrompt = renameNodePrompt;

