import { state } from './state.js';
import { invoke, openDialog } from './services/tauri.js';
import { escapeAttr, showStatus } from './utils.js';
import { openFile, renderTabs } from './editor.js';
import { loadChatHistory } from './ai.js';
import { listen } from './services/tauri.js';

export function setupExplorerListeners() {
    listen('file-system-changed', () => {
        if (state.currentStoryPath) {
            loadNodes(state.currentStoryPath);
        }
    });
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

export async function openStory(path) {
    const storyEditor = document.getElementById('story-editor');
    const currentFileName = document.getElementById('current-file-name');
    const statusStoryName = document.getElementById('status-story-name');

    if (state.currentStoryPath && state.currentStoryPath !== path) {
        state.openTabs = [];
        state.activeFilePath = null;
        renderTabs();
        if (storyEditor) storyEditor.innerText = "";
        if (currentFileName) currentFileName.innerText = "Chọn một file để bắt đầu";
    }

    state.currentStoryPath = path;
    const folderName = path.split(/[/\\]/).pop() || path;
    if (statusStoryName) statusStoryName.innerText = folderName;
    
    const storyTitleDisplay = document.getElementById('story-title-display');
    if (storyTitleDisplay) storyTitleDisplay.innerText = folderName.toUpperCase();

    localStorage.setItem('last_story_path', path);

    await window.loadNodes(path);
    await loadChatHistory();
}

export async function loadNodes(rootPath) {
    if (!rootPath) return;
    try {
        state.nodes = await invoke('list_nodes', { rootPath });
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
    if (window.lucide) window.lucide.createIcons();
}

export function createNodeElement(node) {
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
                <button onclick="event.stopPropagation(); window.showNodeModal('${escapeAttr(node.path)}', 'file')" title="Add File"><i data-lucide="file-plus"></i></button>
                <button onclick="event.stopPropagation(); window.showNodeModal('${escapeAttr(node.path)}', 'folder')" title="Add Folder"><i data-lucide="folder-plus"></i></button>
            ` : ''}
            <button onclick="event.stopPropagation(); window.renameNodePrompt('${escapeAttr(node.path)}', '${escapeAttr(node.name)}')" title="Rename"><i data-lucide="pencil"></i></button>
            <button onclick="event.stopPropagation(); window.deleteNode('${escapeAttr(node.path)}')" title="Delete"><i data-lucide="trash-2"></i></button>
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

    // Initial render if children exist
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

export function showNodeModal(parentPath, type) {
    window.showModal(`Tên ${type === 'folder' ? 'thư mục' : 'tập tin'} mới`, "create-node", { parentPath, type });
}
window.showNodeModal = showNodeModal;

export async function deleteNode(nodePath) {
    if (!confirm("Bạn có chắc chắn muốn xóa? Thư mục con cũng sẽ bị xóa.")) return;
    try {
        await invoke('delete_node', {
            rootPath: state.currentStoryPath,
            nodePath,
        });
        await loadNodes(state.currentStoryPath);
        if (state.activeFilePath === nodePath) {
            import('./editor.js').then(module => module.closeTab(nodePath));
        }
    } catch (err) {
        console.error("Delete failed:", err);
    }
}
window.deleteNode = deleteNode;

export function renameNodePrompt(nodePath, oldName) {
    window.showModal("Đổi tên", "rename-node", { nodePath, oldName });
}
window.renameNodePrompt = renameNodePrompt;
