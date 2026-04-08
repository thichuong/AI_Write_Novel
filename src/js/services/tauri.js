export const invoke = window.__TAURI__?.core?.invoke || (async (cmd, args) => {
    console.warn(`Tauri invoke('${cmd}') failed: window.__TAURI__ is undefined.`, args);
    return Promise.reject("Tauri API not available");
});

export const listen = window.__TAURI__?.event?.listen || (async (event, handler) => {
    console.warn(`Tauri listen('${event}') failed: window.__TAURI__ is undefined.`);
    return () => {}; // return dummy unlisten function
});

// Tauri Plugin FS API (v2)
export const fs = window.__TAURI__?.fs || {};

// Tauri Path API
export const path = window.__TAURI__?.path || {};

// Tauri Dialog API (v2)
export const openDialog = async (options) => {
    if (window.__TAURI__?.dialog?.open) {
        return await window.__TAURI__.dialog.open(options);
    }
    if (window.__TAURI__?.core?.invoke) {
        return await window.__TAURI__.core.invoke('plugin:dialog|open', options);
    }
    return Promise.reject("Tauri Dialog API not available");
};
