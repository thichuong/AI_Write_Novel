export const state = {
    currentStoryPath: null,
    nodes: [],           // Tree of FileNode from Rust
    openTabs: [],        // Array of { name, path, content }
    activeFilePath: null, // relative path of active file
    chatHistory: [],
    isSaving: false,
};
