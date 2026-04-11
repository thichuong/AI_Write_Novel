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

export function showStatus(text, isError = false) {
    const syncEl = document.getElementById('sync-status');
    if (!syncEl) return;
    
    const icon = isError ? "alert-circle" : "loader";
    syncEl.innerHTML = `<i data-lucide="${icon}"></i> ${text}`;
    if (isError) syncEl.classList.add('error');
    else syncEl.classList.remove('error');
    
    if (window.lucide) window.lucide.createIcons();
    
    setTimeout(() => {
        syncEl.innerHTML = `<i data-lucide="hard-drive"></i> File System`;
        syncEl.classList.remove('error');
        if (window.lucide) window.lucide.createIcons();
    }, 5000); // 5s for errors/status
}
