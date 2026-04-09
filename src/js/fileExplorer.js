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

    const explorerTree = document.getElementById('explorer-tree');
    if (explorerTree) {
        explorerTree.addEventListener('sl-selection-change', (event) => {
            const selectedItem = event.detail.selection[0];
            if (selectedItem) {
                state.selectedNodePath = selectedItem.dataset.path;
                state.selectedNodeType = selectedItem.dataset.type;
                state.selectedNodeName = selectedItem.dataset.name;
                
                if (state.selectedNodeType === 'file') {
                    const node = {
                        path: state.selectedNodePath,
                        name: state.selectedNodeName,
                        nodeType: 'file'
                    };
                    openFile(node);
                }
            } else {
                state.selectedNodePath = null;
                state.selectedNodeType = null;
                state.selectedNodeName = null;
            }
        });

        // Track expansion state
        explorerTree.addEventListener('sl-expand', (event) => {
            const path = event.target.dataset.path;
            if (path) state.expandedPaths.add(path);
        });

        explorerTree.addEventListener('sl-collapse', (event) => {
            const path = event.target.dataset.path;
            if (path) state.expandedPaths.delete(path);
        });

        // Context Menu
        explorerTree.addEventListener('contextmenu', (event) => {
            event.preventDefault();
            const treeItem = event.target.closest('sl-tree-item');
            if (treeItem) {
                // Select on right-click so header buttons work on this item
                treeItem.selected = true;
                state.selectedNodePath = treeItem.dataset.path;
                state.selectedNodeType = treeItem.dataset.type;
                state.selectedNodeName = treeItem.dataset.name;
                
                showContextMenu(event.clientX, event.clientY, treeItem.dataset.path, treeItem.dataset.type, treeItem.dataset.name);
            } else {
                // Background of tree
                showContextMenu(event.clientX, event.clientY, state.currentStoryPath, 'folder', 'root');
            }
        });
    }

    // Close context menu on click elsewhere
    document.addEventListener('click', () => {
        const menu = document.getElementById('context-menu');
        if (menu) menu.classList.add('hidden');
    });
}

function showContextMenu(x, y, path, type, name) {
    const menu = document.getElementById('context-menu');
    if (!menu) return;

    menu.style.top = `${y}px`;
    menu.style.left = `${x}px`;
    menu.classList.remove('hidden');

    const slMenu = document.getElementById('explorer-context-menu');
    slMenu.onclick = (event) => {
        const menuItem = event.target.closest('sl-menu-item');
        if (!menuItem) return;
        const action = menuItem.value;

        if (action === 'new-file') showNodeModal(path, 'file');
        else if (action === 'new-folder') showNodeModal(path, 'folder');
        else if (action === 'rename') renameNodePrompt(path, name);
        else if (action === 'delete') deleteNode(path);

        menu.classList.add('hidden');
    };
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

export async function handleNewFile() {
    if (!state.currentStoryPath) return;
    let parentPath = state.currentStoryPath;
    if (state.selectedNodePath) {
        parentPath = state.selectedNodeType === 'folder' ? state.selectedNodePath : await path.dirname(state.selectedNodePath);
    } else if (state.activeFilePath) {
        parentPath = await path.dirname(state.activeFilePath);
    }
    showNodeModal(parentPath, 'file');
}

export async function handleNewFolder() {
    if (!state.currentStoryPath) return;
    let parentPath = state.currentStoryPath;
    if (state.selectedNodePath) {
        parentPath = state.selectedNodeType === 'folder' ? state.selectedNodePath : await path.dirname(state.selectedNodePath);
    } else if (state.activeFilePath) {
        parentPath = await path.dirname(state.activeFilePath);
    }
    showNodeModal(parentPath, 'folder');
}

export function handleRename() {
    if (state.selectedNodePath) {
        renameNodePrompt(state.selectedNodePath, state.selectedNodeName);
    } else {
        alert("Vui lòng chọn một tệp hoặc thư mục để đổi tên.");
    }
}

export function handleDeleteBtn() {
    if (state.selectedNodePath) {
        deleteNode(state.selectedNodePath);
    } else {
        alert("Vui lòng chọn một tệp hoặc thư mục để xóa.");
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
    if (node.name === 'wiki' && node.nodeType === 'folder') {
        customIcon.name = 'book';
    } else if (node.name === 'chat_history.json' && node.nodeType === 'file') {
        customIcon.name = 'clock-history';
    } else {
        customIcon.name = node.nodeType === 'folder' ? 'folder' : 'file-earmark-text';
    }
    treeItem.appendChild(customIcon);

    if (node.nodeType === 'folder') {
        treeItem.lazy = true;
        
        // Restore expansion state
        if (state.expandedPaths.has(node.path)) {
            treeItem.expanded = true;
            // For lazy items, we need to trigger the load manually if expanding
            setTimeout(() => treeItem.dispatchEvent(new CustomEvent('sl-lazy-load')), 0);
        }

        // Toggle expansion on click (not just on the arrow)
        treeItem.addEventListener('click', (event) => {
            // Only toggle if the click is on this item itself, not on a nested item
            if (event.target.closest('sl-tree-item') !== treeItem) return;

            const path = event.composedPath();
            const isExpandButton = path.some(el => el instanceof HTMLElement && el.getAttribute('part') === 'expand-button');
            
            if (!isExpandButton) {
                treeItem.expanded = !treeItem.expanded;
            }
        });

        // Load children on lazy load
        treeItem.addEventListener('sl-lazy-load', async (event) => {
            if (treeItem.lazy) { 
                treeItem.loading = true; // Ensure loading state is visible if not already
                try {
                    const children = await fs.readDir(node.path);
                    
                    // Stop loading if empty
                    if (children.length === 0) {
                        treeItem.lazy = false;
                        treeItem.loading = false;
                        return;
                    }

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
                    
                    treeItem.lazy = false;
                    treeItem.expanded = true;
                } catch (err) {
                    console.error("Failed to load sub-folder:", err);
                    treeItem.lazy = false;
                } finally {
                    treeItem.loading = false;
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

