export function escapeAttr(str) {
    if (!str) return "";
    return str.replace(/'/g, "\\'").replace(/"/g, '&quot;');
}

export function debounce(func, timeout = 300) {
    let timer;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => { func.apply(this, args); }, timeout);
    };
}

export function showStatus(text) {
    const syncEl = document.getElementById('sync-status');
    if (!syncEl) return;
    syncEl.innerHTML = `<i data-lucide="loader"></i> ${text}`;
    if (window.lucide) window.lucide.createIcons();
    setTimeout(() => {
        syncEl.innerHTML = `<i data-lucide="hard-drive"></i> File System`;
        if (window.lucide) window.lucide.createIcons();
    }, 3000);
}
